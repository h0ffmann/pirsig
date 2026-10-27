# zettelkasten

2nd brain from the future. `ztk` is the product: a [Loco](https://loco.rs) app with a
REST API and a terminal CLI that share one model layer.

![How ztk works: the ztk CLI and the REST API make the same calls into one model layer, which reads and writes SQLite; ztk figure exports the PARA tree for archify, which draws a checked HTML page and PNG; ztk graph exports a graph.json for graphify, which clusters, queries and draws it.](docs/img/how-ztk-works.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** Whatever you use, terminal or HTTP, goes through the same rules, and the brain can
leave ztk as a diagram or as a knowledge graph.

**How to read.** Top to bottom, one lane per role. Arrows say what flows. The orange border marks
where you come in and the outside tools that draw the result.

**Not shown.** Auth, the background workers, and the archive (`--archived`) and done tasks
(`--all`) switches of the exports.

**Evidence.** [`docs/archify/how-ztk-works.json`](docs/archify/how-ztk-works.json), the source of
the figure; `src/cli/mod.rs`, `src/controllers/`, `src/figures/`.

</details>

## PARA

| | What | API | CLI |
|---|---|---|---|
| **P**rojects | outcomes with an end, inside an area | `/api/projects` | `ztk project` |
| **A**reas | ongoing responsibilities | `/api/areas` | `ztk area` |
| **R**esources | reference material, optionally tied to an area/project | `/api/resources` | `ztk resource` |
| **A**rchives | `archived_at` on areas, projects, resources | `POST /api/<kind>/{id}/archive` / `unarchive`, `?archived=true` | `--archive ID`, `--unarchive ID`, `--archived` |

Tasks (`/api/tasks`, `ztk task`) are next actions inside a project.

## CLI

```sh
cargo install --path . --bin ztk   # or: cargo run --bin ztk -- <cmd>

ztk area -c -n Health                       # create
ztk area -u 1 --description "body & mind"   # update (only the given fields)
ztk project -c -n "Run a marathon" -a 1 --status active
ztk task -c -t "Buy shoes" -p 1 --due 2026-11-01
ztk task --done 1
ztk resource -c -t "Couch to 5k" --url https://example.com -a 1
ztk area --archive 1
ztk area                                    # list (add --archived for the archive)
ztk para                                    # the whole tree
ztk project -d 1 --force                    # delete, including its tasks
```

Every command takes `--json`. `ztk` uses the config of `LOCO_ENV` (default
`development`) from `./config`, or from `LOCO_CONFIG_FOLDER`; `DATABASE_URL` overrides the
SQLite file. Pending migrations run automatically.

## Figures and graphs

`ztk figure` exports the PARA tree as [archify](https://github.com/tt-a1i/archify) JSON IR, and
`ztk graph` exports the brain as a [graphify](https://github.com/safishamsi/graphify) `graph.json`
(also `GET /api/figures/tree` and `GET /api/figures/graph`). Both take `--archived` and `--all`.

```sh
ztk figure -o docs/archify/para.json && just figure     # -> docs/img/para.png
just graph                                              # -> graphify-out/graph.html, GRAPH_REPORT.md
graphify query "what feeds the marathon?"
```

![The demo brain as a tree: the root "A second brain in ztk" has the areas Health, Work and Second brain and an Unfiled resources node; Health holds the project Run a marathon with its tasks and the resource Couch to 5k; Work holds Q4 roadmap; Second brain holds Ship ztk v0 with two tasks and the resource PARA method; Reading list is unfiled.](docs/img/para.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** This is `ztk para` as a picture: every area, project, open task and resource in its
place.

**How to read.** Top down, each line means "contains". Below the root are the areas, then their
projects, then tasks and resources. Small grey text is the description, status, due date or URL.

**Not shown.** Done tasks and the archive; `ztk figure --all --archived` puts them in.

**Evidence.** [`docs/archify/para.json`](docs/archify/para.json), written by `ztk figure` from the
demo brain of `just figure-para`; the drawing is archify's.

</details>

Diagrams in these docs follow the `figure` skill (`.claude/skills/figure`): sources in
`docs/archify/`, `just figure` draws them with the vendored archify (`just figure-init` once),
`just figure-check` says whether a PNG is stale.

## Server

```sh
cargo loco start      # API on http://localhost:5150
cargo test
```
