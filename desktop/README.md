# ztk desktop (Tauri)

**Bootstrap only.** ztk will ship as a desktop app built with [Tauri 2](https://v2.tauri.app).
What is here is enough to build and open a window that talks to Rust (`just desktop`):

| Path | What |
|---|---|
| `src-tauri/Cargo.toml` | the `ztk-desktop` crate; its own workspace, because Tauri needs a system webview to build and `just check` should not |
| `src-tauri/src/main.rs` | the shell and one command, `version` |
| `src-tauri/tauri.conf.json` | window, CSP, `bundle.active: false` (no installers yet) |
| `src-tauri/capabilities/default.json` | what the window may call: `core:default` |
| `dist/index.html` | the static page the window loads |

CI (`.github/workflows/desktop.yml`) runs `cargo check` on macOS, which ships the webview. On
Linux, install `libwebkit2gtk-4.1-dev` and its friends first
([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).

## Next

1. Start ztk's REST API inside the app (the Loco app, on a local port, with the brain's SQLite
   file) and point the window at it, or call the model layer from Tauri commands.
2. A real frontend (the Figma designs, through the Figma MCP server this repository configures).
3. `bundle.active: true`, icons for every platform, signing, and a release workflow.
