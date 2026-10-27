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

## Slipbox (notes, smart notes, Zettelkasten)

Notes live in a slipbox with Luhmann-style ids (`zid`): `1` starts a train of thought,
`1a` continues it, `1a1` branches off `1a`, `2` starts the next one. Everything is
addressed by zid.

- **Kinds** (after *How to Take Smart Notes*): `fleeting` (default, the inbox),
  `literature` (with a `source`), `permanent` (your own words), `hub` (structure note).
- **Links**: `[[1a]]` or `[[Some title]]` in a body becomes a link on save, including
  links written before the target existed. Hand-made links (`ztk link`) carry a reason
  and survive edits.
- **Tags**: `#hashtags` in the body plus explicit tags.
- **Views**: backlinks, inbox, orphans (unconnected notes), random (serendipity), tree
  (Folgezettel order), graph (JSON nodes/edges), stats, Markdown export with front matter.

| | API | CLI |
|---|---|---|
| notes | `GET/POST /api/notes` (`?q=&kind=&tag=`), `GET/PUT/PATCH/DELETE /api/notes/{zid}` | `ztk note` |
| Markdown | `GET /api/notes/{zid}/markdown` | `ztk note -s ZID --md` |
| links | `POST /api/notes/{zid}/links` `{target, context}`, `DELETE /api/notes/{zid}/links/{target}` | `ztk link FROM TO -w why`, `-d` |
| views | `GET /api/slipbox/{inbox,orphans,random,tree,graph,stats}` | `ztk box [inbox\|orphans\|random\|tree\|graph\|tags\|stats]` |
| export | | `ztk box export ./vault` |

```sh
ztk note -c "Atomic notes"                      # quick capture (fleeting)
ztk note -c -t "Links beat folders" -a 1 -k permanent -e   # continue 1 as 1a, write in $EDITOR
echo "See [[1]] #pkm" | ztk note -c "Piped"     # body from stdin
ztk note -s 1a                                  # note + links + backlinks
ztk note -u 1a -e                               # edit body in $EDITOR
ztk note rust                                   # search; also -k KIND, --tag TAG
ztk link 1a 2 -w "contradicts"
ztk box tree
ztk box inbox
```

## justfile

[`just`](https://github.com/casey/just) keeps the commands short: `just n "An idea"`,
`just after 1 "Next"`, `just show 1a`, `just edit 1a`, `just find rust`,
`just box inbox`, `just export`, `just dev`, `just check` (fmt + clippy + tests, as CI).
Run `just` to list them all.

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
