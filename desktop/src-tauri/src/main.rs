//! Pirsig as a desktop app. The app runs Pirsig's own REST API (the Loco app, on a
//! free port of 127.0.0.1, with the same brain as the `psg` CLI) and opens a
//! window on it; the API also serves the page (`desktop/dist`), so the window
//! and the API share an origin. See `desktop/README.md`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, net::TcpListener, path::PathBuf, time::Duration};

use loco_rs::{
    app::Hooks,
    boot::{self, ServeParams, StartMode},
    environment::{resolve_from_env, Environment},
};
use tauri::{WebviewUrl, WebviewWindowBuilder};
use zettelkasten::app::App;

/// The folder Pirsig runs in: `PIRSIG_HOME`, else the current one when it holds
/// Pirsig's `config/`, else the checkout this app was built from. The brain (the
/// database in the config) and the page are found from there.
fn home() -> PathBuf {
    if let Some(home) = std::env::var_os("PIRSIG_HOME") {
        return home.into();
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    if cwd.join("config").is_dir() {
        return cwd;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A port nobody is listening on.
fn free_port() -> std::io::Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

/// Boot the API on `port` and serve it in the background; returns once it
/// answers, with the task that ends when the API stops.
async fn serve(port: u16) -> Result<tokio::task::JoinHandle<()>, Box<dyn Error>> {
    let environment: Environment = resolve_from_env().into();
    let mut config = App::load_config(&environment).await?;
    config.server.port = port.into();
    config.server.binding = "127.0.0.1".into();
    config.server.host = "http://127.0.0.1".into();
    let booted = App::boot(StartMode::ServerOnly, &environment, config).await?;
    let params = ServeParams {
        port: port.into(),
        binding: "127.0.0.1".into(),
    };
    let server = tokio::spawn(async move {
        if let Err(e) = boot::start::<App>(booted, params, true).await {
            eprintln!("Pirsig API stopped: {e}");
        }
    });
    for _ in 0..200 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Ok(server);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    Err("the Pirsig API did not start".into())
}

fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_current_dir(home())?;
    // the API lives on this runtime for as long as the app runs
    let runtime = tokio::runtime::Runtime::new()?;
    let port = free_port()?;
    let server = runtime.block_on(serve(port))?;
    let url = format!("http://127.0.0.1:{port}/").parse()?;
    let handle = runtime.handle().clone();
    tauri::Builder::default()
        .setup(move |app| {
            // the API stops on a shutdown signal (Ctrl-C, SIGTERM) or an
            // error; a window with nothing behind it is no use, so quit too
            let app_handle = app.handle().clone();
            handle.spawn(async move {
                let _ = server.await;
                app_handle.exit(0);
            });
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title("Pirsig")
                .inner_size(1240.0, 820.0)
                .min_inner_size(720.0, 480.0)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    drop(runtime);
    Ok(())
}
