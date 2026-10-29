# Mobile screens

A first pass at Pirsig on a phone: the desktop app's views, reshaped for one hand. The
sidebar becomes a tab bar (Today, Notes, Slip-box, PARA, More) and quick capture becomes a
sheet. Colours, kinds and words come from the desktop app (`desktop/dist/app.css` and the
views in `desktop/dist/js/views/`), so the two read as one app.

The screens were drawn in [OpenPencil](https://github.com/open-pencil/open-pencil) through
its MCP server, at 390×844 and exported at 2×. They are mock-ups with sample data, not
screenshots of a build.

| Today | Notes | Note |
|---|---|---|
| <img alt="Today: quick capture, counts for notes, links, inbox, orphans and readiness, the inbox to process and active projects" src="../img/mobile/today.png" width="260"/> | <img alt="Notes: search, chips to filter by kind, and notes with their ids, kinds and hashtags" src="../img/mobile/notes.png" width="260"/> | <img alt="A permanent note: its body with a wikilink to 1a, a quote, tags, then its links and backlinks" src="../img/mobile/note.png" width="260"/> |

| Slip-box | Project | Capture |
|---|---|---|
| <img alt="Slip-box: Inbox, Orphans, Tree and Tags; the tree shows notes in Folgezettel order, continuations indented under the note they continue" src="../img/mobile/slipbox.png" width="260"/> | <img alt="The project Run a 10k in the Health area: due date, task progress, tasks and the notes filed under it" src="../img/mobile/project.png" width="260"/> | <img alt="The capture sheet over Today: title, body, a kind picker and pickers for project and Hippo" src="../img/mobile/capture.png" width="260"/> |

What the screens map to:

| Screen | Desktop view | API |
|---|---|---|
| Today | `#/` (`today.js`) | `/api/slipbox/stats`, `/api/readiness`, `/api/slipbox/inbox`, `/api/projects`, `/api/tasks` |
| Notes | `#/notes` | `/api/notes` |
| Note | `#/notes/:zid` | `/api/notes/:zid` |
| Slip-box | `#/slipbox/:tab` | `/api/slipbox/{inbox,orphans,tree}`, `/api/slipbox/stats` |
| Project | `#/projects/:id` | `/api/projects/:id`, `/api/projects/:id/notes`, `/api/tasks` |
| Capture | the sidebar's capture form | `POST /api/notes` |

Not drawn yet: Hippos, Areas, Resources, Feeds and arXiv, Plasticity, Readiness, the note
editor, and dark mode.
