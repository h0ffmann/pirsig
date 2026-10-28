use clap::Parser;
use zettelkasten::cli::{self, Cli};

#[tokio::main]
async fn main() {
    if let Err(err) = cli::run(Cli::parse()).await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
