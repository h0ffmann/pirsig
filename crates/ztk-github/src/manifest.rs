//! The GitHub App manifest for "ztk bootstrap".
//!
//! Registered once by the maintainer through GitHub's manifest flow
//! (`github-app/README.md`); every ztk user then signs in to that app with
//! the device flow, so no one has to create their own app or paste a token.
//!
//! Permissions are read-only and as few as the indexer needs:
//!
//! | permission | why |
//! |---|---|
//! | `metadata: read` | repo name, visibility, fork parent, language, topics (mandatory for every app) |
//! | `contents: read` | READMEs of **public** repos, for the opt-in `ztk git import`; never used for minimal records |
//! | `pull_requests: read` | count the user's PRs in org repositories, private ones included |
//!
//! No webhooks: the index is refreshed by `ztk bootstrap gh --sync` (cron), so
//! the app needs no server.

use serde_json::{json, Value};

/// Where the app points people: the project home.
pub const HOMEPAGE: &str = "https://github.com/h0ffmann/zettelkasten";

/// Read-only permissions the app requests, as `(name, level)`.
pub const PERMISSIONS: [(&str, &str); 3] = [
    ("metadata", "read"),
    ("contents", "read"),
    ("pull_requests", "read"),
];

/// The manifest as GitHub's manifest flow expects it.
#[must_use]
pub fn manifest(name: &str, homepage: &str) -> Value {
    let permissions: serde_json::Map<String, Value> = PERMISSIONS
        .iter()
        .map(|(k, v)| ((*k).to_string(), json!(v)))
        .collect();
    json!({
        "name": name,
        "url": homepage,
        "description": "Seed a ztk second brain from your GitHub FOSS work: your repos, public forks and contributions to the orgs you choose. Read-only; private repositories are indexed as metadata only and never published.",
        "public": true,
        // After creation GitHub redirects here with ?code=...; the code is
        // exchanged once for the app credentials (see github-app/README.md).
        "redirect_url": homepage,
        // Loopback for a future browser sign-in; GitHub accepts any port on it.
        "callback_urls": ["http://127.0.0.1/callback"],
        "request_oauth_on_install": false,
        "setup_on_update": false,
        "hook_attributes": { "url": homepage, "active": false },
        "default_events": [],
        "default_permissions": permissions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_committed_manifest_is_the_output() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../github-app/manifest.json"
        );
        let committed: Value =
            serde_json::from_str(&std::fs::read_to_string(path).expect("manifest.json")).unwrap();
        assert_eq!(
            committed,
            manifest("ztk bootstrap", HOMEPAGE),
            "regenerate: cargo run -q -p ztk-github --bin ztk-gh-app > github-app/manifest.json"
        );
    }

    #[test]
    fn asks_for_read_only_and_no_events() {
        let m = manifest("ztk bootstrap", HOMEPAGE);
        let perms = m["default_permissions"].as_object().unwrap();
        assert!(perms.values().all(|v| v == "read"));
        assert!(m["default_events"].as_array().unwrap().is_empty());
        assert_eq!(m["hook_attributes"]["active"], false);
    }
}
