# zettelkasten

2nd brain from the future. `ztk` is the product: a [Loco](https://loco.rs) app with a
REST API and a terminal CLI that share one model layer.

## Use it from a private fork

Your brain is personal data; this repository is public code. Run ztk from a **private
copy** of this repo, so nothing you write ends up on the web just because it is in git:

```sh
# create an empty private repo first, e.g. github.com/<you>/brain
git clone https://github.com/h0ffmann/zettelkasten.git brain && cd brain
git remote rename origin upstream
git remote add origin git@github.com:<you>/brain.git
git push -u origin main
git pull upstream main     # later, to update ztk
```

Making something public is a separate, explicit step (planned, not built yet): `ztk publish` shows a plan of
exactly what would become public (like `terraform plan`) and only `ztk publish --apply`
does it. See [docs/design](docs/design/README.md) for the publish, pandoc and git
inbound designs.

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
