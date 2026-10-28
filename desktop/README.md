# Pirsig desktop (Tauri)

`just desktop` opens Pirsig as a desktop app built with [Tauri 2](https://v2.tauri.app). The
whole brain is managed from it:

| Section | What you can do |
|---|---|
| Today | stats, readiness score, the inbox, active projects, dated tasks; quick capture from the sidebar on every page |
| Notes | search by text, kind and tag; write and edit with a live preview (`[[links]]`, `#tags`, ⌘S); link with a reason, unlink, continue a note (Folgezettel), pull into a project, see the Markdown Obsidian gets, delete |
| Slip-box | inbox, orphans, the Folgezettel tree, tags, a random note |
| Hippos | create, edit, delete; the rhizome map (plateaus, pieces, where you left off); grow a note in it; weave |
| Projects | create, edit, archive, delete (with their tasks, after a second confirmation); tasks; the notes a project pulls in, with why; its resources |
| Areas | create, edit, archive, delete (with their projects, after a second confirmation); an area's projects and resources |
| Resources | create, edit, filter, archive, delete; write a literature note from one |
| Tasks | open and done, across projects; tick, edit, delete |
| Feeds and arXiv | subscribe, pull, unsubscribe; search arXiv and ingest papers as resources |
| Plasticity, Readiness | practise exercises; the methodology checks with their fixes |

## How it fits together

```
pirsig-desktop (Tauri)                        the same in a browser
  ├─ Pirsig's REST API (Loco), 127.0.0.1:<free port>     `just dev` → localhost:5150
  │    └─ serves desktop/dist (the UI) at /
  └─ a window on http://127.0.0.1:<port>/
```

The app starts the Loco app from the `zettelkasten` crate in-process, on a free port of
`127.0.0.1`, and opens its window on it. The API also serves the page (`config/development.yaml`,
`server.middlewares.static`), so the page and the API share an origin: no CORS, no Tauri
commands, and the same UI in a browser while `just dev` runs. The page is plain HTML, CSS
and ES modules (`dist/`), with no build step and no dependencies; it builds the DOM from text,
so nothing a note holds is ever parsed as HTML.

The brain is the one in `config/development.yaml`: the app runs in `PIRSIG_HOME` when set, else
in the current folder when it holds Pirsig's `config/`, else in the checkout it was built from.
Any process on this machine can reach the API's port while the app runs, as with `just dev`.

| Path | What |
|---|---|
| `src-tauri/src/main.rs` | starts the API, then the window |
| `src-tauri/tauri.conf.json` | app id, CSP, `bundle.active: false` (no installers yet) |
| `dist/index.html`, `dist/app.css` | the page and its styles (light and dark) |
| `dist/js/api.js`, `ui.js`, `md.js` | the API client, a small DOM and dialog toolkit, the Markdown renderer |
| `dist/js/views/*.js` | one module per section |
| `e2e/brain.spec.mjs` | the end-to-end story, every section, on a fresh brain |

## Tests

`just desktop-e2e` runs `e2e/` with Playwright in WebKit (the engine of the macOS webview):
it serves the API and UI on a throwaway brain (`target/e2e-brain.sqlite`) and goes through
every section, creating, editing, archiving and deleting. CI (`.github/workflows/desktop.yml`)
runs it on every change to `desktop/` and checks that the app compiles on macOS. On Linux,
install `libwebkit2gtk-4.1-dev` and its friends first
([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).

## Next

1. Installers: `bundle.active: true`, icons for every platform, signing, a release workflow,
   and the brain in the platform's app-data folder for an installed app.
2. The Figma designs, through the Figma MCP server this repository configures.
