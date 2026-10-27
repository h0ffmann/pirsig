//! `ztk-gh-app`: writes the GitHub App manifest, or a page that registers it.
//!
//! ```console
//! cargo run -q -p ztk-github --bin ztk-gh-app > github-app/manifest.json
//! cargo run -q -p ztk-github --bin ztk-gh-app register [ORG] > register.html
//! ```
//!
//! `register` prints an HTML form that posts the manifest to GitHub's
//! "new app" page (for your account, or for ORG). Open it in a browser
//! signed in to GitHub; see github-app/README.md for the rest.

use ztk_github::manifest::{manifest, HOMEPAGE};

fn main() -> Result<(), String> {
    let name = std::env::var("ZTK_APP_NAME").unwrap_or_else(|_| "ztk bootstrap".to_string());
    let m = manifest(&name, HOMEPAGE);
    let mut args = std::env::args().skip(1);
    let out = match args.next().as_deref() {
        None => serde_json::to_string_pretty(&m).map_err(|e| e.to_string())?,
        Some("register") => register_page(&m.to_string(), args.next().as_deref()),
        Some(other) => {
            return Err(format!(
                "unknown argument {other}: try `register` or nothing"
            ))
        }
    };
    println!("{out}");
    Ok(())
}

fn register_page(manifest: &str, org: Option<&str>) -> String {
    let action = org.map_or_else(
        || "https://github.com/settings/apps/new".to_string(),
        |o| format!("https://github.com/organizations/{o}/settings/apps/new"),
    );
    let escaped = manifest
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;");
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>Register ztk bootstrap</title>
<form action="{action}" method="post">
  <input type="hidden" name="manifest" value="{escaped}">
  <p>Creates the read-only GitHub App described in github-app/manifest.json.</p>
  <button type="submit">Create the GitHub App</button>
</form>"#
    )
}
