<div align="center">

<table>
<tr>
<td align="center"><img alt="ztk: a head in one line whose brain is a zettelkasten, by @h0ffmann" src="docs/drawings/ztk.svg" width="320"/></td>
<td align="center"><a href="https://www.desmos.com/calculator/4vxpt3qrjp"><img alt="Brain, a Desmos graph" src="https://saved-work.desmos.com/calc_thumbs/production/4vxpt3qrjp.png" width="320"/></a></td>
</tr>
<tr>
<td align="center"><sub><i>ztk</i>, Matheus Hoffmann (<a href="https://github.com/h0ffmann">@h0ffmann</a>), 2026</sub></td>
<td align="center"><sub><a href="https://www.desmos.com/calculator/4vxpt3qrjp"><i>Brain</i></a>, a Desmos graph; author not shown ⚠</sub></td>
</tr>
<tr>
<td align="center"><sub>Desmos source: <a href="docs/drawings/ztk.desmos.json"><code>ztk.desmos.json</code></a></sub></td>
<td align="center"><sub>Desmos source: <a href="https://www.desmos.com/calculator/4vxpt3qrjp"><code>desmos.com/calculator/4vxpt3qrjp</code></a></sub></td>
</tr>
</table>

</div>

# zettelkasten

2nd brain from the future. `ztk` is the product: a [Loco](https://loco.rs) app with a
REST API and a terminal CLI that share one model layer.

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

## Server

```sh
cargo loco start      # API on http://localhost:5150
cargo test
```

## Drawing

The drawing at the top is rendered from its Desmos expressions by `cargo run -p ztk-drawings --bin
ztk-art`: see [`docs/drawings/README.md`](docs/drawings/README.md). The graph beside it loads its
picture from Desmos; this repository does not copy it.
