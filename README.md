<div align="center">

<table>
<tr>
<td align="center"><img alt="The Pirsig logo: a head in one line whose brain is a zettelkasten, by @h0ffmann" src="docs/drawings/pirsig.svg" width="320"/></td>
<td align="center"><a href="https://www.desmos.com/calculator/4vxpt3qrjp"><img alt="Brain, a Desmos graph" src="https://saved-work.desmos.com/calc_thumbs/production/4vxpt3qrjp.png" width="320"/></a></td>
</tr>
<tr>
<td align="center"><sub><i>The Pirsig logo</i>, Matheus Hoffmann (<a href="https://github.com/h0ffmann">@h0ffmann</a>), 2026</sub></td>
<td align="center"><sub><a href="https://www.desmos.com/calculator/4vxpt3qrjp"><i>Brain</i></a>, a Desmos graph; author not shown ⚠</sub></td>
</tr>
<tr>
<td align="center"><sub>Desmos source: <a href="docs/drawings/pirsig.desmos.json"><code>pirsig.desmos.json</code></a></sub></td>
<td align="center"><sub>Desmos source: <a href="https://www.desmos.com/calculator/4vxpt3qrjp"><code>desmos.com/calculator/4vxpt3qrjp</code></a></sub></td>
</tr>
</table>

</div>

# Pirsig

2nd brain from the future. Pirsig is a [Loco](https://loco.rs) app with a REST API, a
desktop app and a terminal CLI, `psg`, that share one model layer.

> Pirsig is named for **Robert M. Pirsig** (1928–2017), the author of
> [*Zen and the Art of Motorcycle Maintenance*](https://en.wikipedia.org/wiki/Zen_and_the_Art_of_Motorcycle_Maintenance) (1974) and *Lila* (1991).
> He is my favorite author and a great inspiration to all of my work. On 24 April 2027 it
> will be ten years without him.

## Ways to run Pirsig

| | How | Status |
|---|---|---|
| **Self-hosted, live** | The desktop app (Tauri, Rust): `just desktop`. Or the server alone, `just dev`, with the same UI in a browser on http://localhost:5150. Your brain stays on your machine, in SQLite. | available |
| **Mobile app** | Pirsig on your phone. | coming |
| **Mobile cloud** | Pirsig hosted for you at [piabrota.art/pirsig](https://piabrota.art/pirsig). | coming |

## Start from my brain, or from yours

A copy of this repository carries my brain's public notes in [`brain/`](brain/): my GitHub
repositories and public forks, one note each, indexed by category under the areas I keep. It
is a Markdown vault (the Obsidian export format), so it reads well on GitHub too.

```sh
just fork-state        # start from brain/: my note structure, in your brain
just brain-reset       # or replace the whole brain with your own GitHub (psg gh --overwrite)
just fork-export       # write your brain to brain/, for whoever copies yours
```

Both `fork-state` and `brain-reset` back the brain up next to its database file before they
replace it. `psg fork export` leaves out every note tagged `#private`, the resources only those
notes cite, and any link or list line that would name them, so private repositories never
reach `brain/`.

![How Pirsig works: the Pirsig CLI and the REST API make the same calls into one model layer, which reads and writes SQLite; psg figure exports the PARA tree for archify, which draws a checked HTML page and PNG; psg graph exports a graph.json for graphify, which clusters, queries and draws it.](docs/img/how-pirsig-works.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** Whatever you use, terminal or HTTP, goes through the same rules, and the brain can
leave Pirsig as a diagram or as a knowledge graph.

**How to read.** Top to bottom, one lane per role. Arrows say what flows. The orange border marks
where you come in and the outside tools that draw the result.

**Not shown.** Auth, the background workers, and the archive (`--archived`) and done tasks
(`--all`) switches of the exports.

**Evidence.** [`docs/archify/how-pirsig-works.json`](docs/archify/how-pirsig-works.json), the source of
the figure; `src/cli/mod.rs`, `src/controllers/`, `src/figures/`.

</details>

## Use it from a private fork

Your brain is personal data; this repository is public code. Run Pirsig from a **private
copy** of this repo, so nothing you write ends up on the web just because it is in git:

```sh
# create an empty private repo first, e.g. github.com/<you>/brain
git clone https://github.com/ouroboric-dev/zettelkasten.git brain && cd brain
git remote rename origin upstream
git remote add origin git@github.com:<you>/brain.git
git push -u origin main
git pull upstream main     # later, to update Pirsig
```

Making something public is a separate, explicit step (planned, not built yet): `psg publish` shows a plan of
exactly what would become public (like `terraform plan`) and only `psg publish --apply`
does it. See [docs/design](docs/design/README.md) for the publish, pandoc and git
inbound designs.

## PARA

| | What | API | CLI |
|---|---|---|---|
| **P**rojects | outcomes with an end, inside an area | `/api/projects` | `psg project` |
| **A**reas | ongoing responsibilities | `/api/areas` | `psg area` |
| **R**esources | reference material, optionally tied to an area/project | `/api/resources` | `psg resource` |
| **A**rchives | `archived_at` on areas, projects, resources | `POST /api/<kind>/{id}/archive` / `unarchive`, `?archived=true` | `--archive ID`, `--unarchive ID`, `--archived` |

Tasks (`/api/tasks`, `psg task`) are next actions inside a project. The rules are the
same through the CLI and the API:

- Names are never blank, and area names are unique. A project's `status` is `active`
  (the default), `waiting`, `someday` or `done`, and `--due` gives it its end date.
- Nothing active lives in the archive. Archiving an area archives its projects and
  resources, and nothing new goes into an archived area or project.
- Deleting an area or project that still has children needs `--force` (`?force=true`).
- A partial update clears an optional field with `null` in JSON, `""` for a text flag,
  or `--no-due`, `--no-area` or `--no-project` in the CLI.
- A resource can carry a `kind` (`paper`, `article`, `book`, ...) and a unique `citekey`
  that literature notes cite.
- A project pulls notes in by link, without owning them:
  `psg project --pull 2 --note 1a -w "the training plan"`
  (`/api/projects/{id}/notes`).

## Slipbox (notes, smart notes, Zettelkasten)

Notes live in a slipbox with Luhmann-style ids (`zid`): `1` starts a train of thought,
`1a` continues it, `1a1` branches off `1a`, `2` starts the next one. Everything is
addressed by zid.

- **Kinds** (after *How to Take Smart Notes*): `fleeting` (default, the inbox),
  `literature` (must cite a `source`, or the `--resource` it was written from),
  `permanent` (your own words), `hub` (structure note).
- **Links**: `[[1a]]` or `[[Some title]]` in a body becomes a link on save, including
  links written before the target existed. Obsidian's `[[note#heading]]` works, links
  inside code do not count, and renaming a note rewrites `[[Old title]]` in the notes
  that cite it. Hand-made links (`psg link`) carry a reason and survive edits.
- **Labels and tags**: a *label* is a value (`#pkm`), a *tag* is a key/value pair
  (`status=active`); see [labels and tags](docs/design/labels-and-tags.md). Notes take
  labels today, as `#hashtags` in the body plus `--tag` (still named tag; renamed to
  `--label` when key/value tags land).
- **Views**: backlinks, inbox, orphans (unconnected notes), random (serendipity), tree
  (Folgezettel order), graph (JSON nodes/edges), stats, Markdown export with front matter.

| | API | CLI |
|---|---|---|
| notes | `GET/POST /api/notes` (`?q=&kind=&tag=`), `GET/PUT/PATCH/DELETE /api/notes/{zid}` | `psg note` |
| Markdown | `GET /api/notes/{zid}/markdown` | `psg note -s ZID --md` |
| links | `POST /api/notes/{zid}/links` `{target, context}`, `DELETE /api/notes/{zid}/links/{target}` | `psg link FROM TO -w why`, `-d` |
| views | `GET /api/slipbox/{inbox,orphans,random,tree,graph,stats}` | `psg box [inbox\|orphans\|random\|tree\|graph\|tags\|stats]` |
| export | | `psg box export ./vault` (an Obsidian vault, see below) |

```sh
psg note -c "Atomic notes"                      # quick capture (fleeting)
psg note -c -t "Links beat folders" -a 1 -k permanent -e   # continue 1 as 1a, write in $EDITOR
echo "See [[1]] #pkm" | psg note -c "Piped"     # body from stdin
psg note -s 1a                                  # note + links + backlinks
psg note -u 1a -e                               # edit body in $EDITOR
psg note rust                                   # search; also -k KIND, --tag TAG
psg link 1a 2 -w "contradicts"
psg box tree
psg box inbox
```

## Hippo: a rhizome for ADHD, autistic, neurotic and obsessive minds

A **Hippo** is a self-described identity: a name, traits (`adhd`, `autistic`, `neurotic`,
`obsessive`, any mix) and *fixations*, the obsessions and special interests its notes gather
around. It is the person's own word, never a diagnosis, and it stays in the private brain.

A Hippo's notes form a **rhizome**, not a Folgezettel tree. Pirsig keeps the rhizome whole
without any filing work:

- **Every note is woven in when it is saved.** It is linked to the notes it shares a
  fixation, a tag or a title word with, or else to the last note captured. Every link says
  why (`rhizome: both circle rogue waves`).
- **There is no trunk:** `--after` is refused.
- **A deleted note's neighbours are linked to each other,** so the rhizome never falls
  apart.

```sh
psg hippo -c -n h0 --trait adhd --trait autistic --fixation "rogue waves" --fixation chess
psg note -c "the Draupner wave was linear focusing?" -H 1   # woven in on save
psg hippo --map 1      # where you left off, each fixation's notes, any pieces
psg hippo --weave 1    # join the pieces
```

The API is `/api/hippos` (CRUD, `GET /{id}/map`, `POST /{id}/weave`), plus `hippo_id` on
notes. `psg ready` checks that each rhizome is in one piece.

## Plasticity: quick exercises

A **plast** is one kind of quick exercise that trains the brain itself. Pirsig ships six, one
per domain, each with five levels:

| Plast | Domain | Exercise |
|---|---|---|
| `mental-arithmetic` | math | sums, products, percentages, squares, in your head |
| `sequences` | abstraction | find the rule, give the next term |
| `argument-forms` | logic | valid, or a classic fallacy? |
| `intervals` | music | semitones, interval names, the note above or below |
| `first-lines` | literature | famous lines of the classics: who wrote it, which work |
| `latin-words` | languages | everyday Latin, both ways |

```sh
psg plast go            # five exercises, the most due first (or: just plast)
psg plast go -d music   # one domain
psg plast               # level, streak and when each is due
```

Every answer is checked and recorded. A right answer pushes the plast out (1 h, 2 h, … up
to 64 h), and three in a row raise its level. A wrong answer brings it back now, and two in
a row lower the level. The API is `/api/plasticity` (`GET /next`, `POST /answer`). For now
the plasts are the ones Pirsig ships; your own plasts come later.

## obsidian-painter

An Obsidian plugin in [`obsidian-painter/`](obsidian-painter/README.md), written mostly in Rust
(`crates/obsidian-painter`, compiled to WebAssembly). It has three features:

- **Desmos:** ```` ```desmos ```` blocks draw offline, from a graph state or from LaTeX lines.
- **Archify:** ```` ```archify ```` blocks show archify figures in the note. "Compose note as
  HTML page" joins the note and its figures into one page.
- **PDF:** "Export note to PDF" writes a paper-quality PDF through pandoc.

`just painter` builds and tests it. A tag `obsidian-painter-v<version>` releases it.

## Desktop app (Tauri)

`just desktop` opens Pirsig as a desktop app ([`desktop/`](desktop/README.md)), on the same brain
as the `psg` CLI. Everything is managed there: PARA (projects with their tasks and pulled-in
notes, areas, resources, the archive), notes with their links and backlinks, the slip-box
views, Hippos, feeds and arXiv, Plasticity and readiness. The app runs Pirsig's REST API inside
itself, and the API serves the UI, so `just dev` shows the same UI in a browser on
http://localhost:5150. `just desktop-e2e` drives it end to end in WebKit.

## Claude Code plugin

The `psg` plugin brings three skills to Claude Code: `psg` (work in the brain), `zk-method`
(check notes against the method) and `hippo` (grow a Hippo's rhizome, adapting to the traits
the person gave).

```sh
claude plugin marketplace add h0ffmann/zettelkasten
claude plugin install pirsig@zettelkasten
```

`scripts/plugin-smoke.sh` installs the plugin from a checkout into an empty config, the way a
third party would. CI runs it on every change to the plugin. The evals in `plugins/pirsig/evals/`
run only by hand (Actions → plugin), because they cost API credits. The layout follows
[h0ffmann/ptbr-writer](https://github.com/h0ffmann/ptbr-writer).

## Integrations and methodology readiness

| Channel | CLI | API |
|---|---|---|
| Obsidian vault out and in | `psg obsidian export ./vault [--archived]`, `psg obsidian import ./vault [--dry-run]` | |
| RSS/Atom feeds in, as resources | `psg rss add URL -a AREA`, `psg rss pull`, `psg rss` | `/api/feeds`, `POST /api/feeds/{id}/pull` |
| The slipbox out, as RSS | `psg rss out -o feed.xml` | `GET /api/slipbox/feed.xml` |
| GitHub repos in: a note each, indexed by category | `psg bootstrap gh [--private minimal] [--readme] [--dry-run]` | |
| arXiv papers in: cited, filed, deduplicated | `psg arxiv add ID... [--note]`, `psg arxiv search QUERY` | `POST /api/ingest/arxiv`, `GET /api/ingest/arxiv/search` |
| Methodology readiness | `psg ready` | `GET /api/readiness` |

`psg ready` checks the brain against PARA, the Zettelkasten method and the three
channels, and says what to run for each gap. [docs/methodology.md](docs/methodology.md)
has the readiness matrix: what is enforced, what is only reported, and where each rule is
tested.

## justfile

[`just`](https://github.com/casey/just) keeps the commands short: `just n "An idea"`,
`just after 1 "Next"`, `just show 1a`, `just edit 1a`, `just find rust`,
`just box inbox`, `just export`, `just ready`, `just dev`, `just check` (fmt + clippy + tests, as CI).
Run `just` to list them all.

## CLI

```sh
cargo install --path . --bin psg   # or: cargo run --bin psg -- <cmd>

psg area -c -n Health                       # create
psg area -u 1 --description "body & mind"   # update (only the given fields)
psg project -c -n "Run a marathon" -a 1 --due 2027-04-18
psg task -c -t "Buy shoes" -p 1 --due 2026-11-01
psg task --done 1
psg resource -c -t "Couch to 5k" --url https://example.com -a 1
psg area --archive 1
psg area                                    # list (add --archived for the archive)
psg para                                    # the whole tree
psg project -d 1 --force                    # delete, including its tasks
```

Every command takes `--json`. `psg` uses the config of `LOCO_ENV` (default
`development`) from `./config`, or from `LOCO_CONFIG_FOLDER`; `DATABASE_URL` overrides the
SQLite file. Pending migrations run automatically.

## Figures and graphs

`psg figure` exports the PARA tree as [archify](https://github.com/tt-a1i/archify) JSON IR, and
`psg graph` exports the brain as a [graphify](https://github.com/safishamsi/graphify) `graph.json`
(also `GET /api/figures/tree` and `GET /api/figures/graph`). Both take `--archived` and `--all`.

```sh
psg figure -o docs/archify/para.json && just figure     # -> docs/img/para.png
just graph                                              # -> graphify-out/graph.html, GRAPH_REPORT.md
graphify query "what feeds the marathon?"
```

![The demo brain as a tree: the root "A second brain in Pirsig" has the areas Health, Work and Second brain and an Unfiled resources node; Health holds the project Run a marathon with its tasks and the resource Couch to 5k; Work holds Q4 roadmap; Second brain holds Ship Pirsig v0 with two tasks and the resource PARA method; Reading list is unfiled.](docs/img/para.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** This is `psg para` as a picture: every area, project, open task and resource in its
place.

**How to read.** Top down, each line means "contains". Below the root are the areas, then their
projects, then tasks and resources. Small grey text is the description, status, due date or URL.

**Not shown.** Done tasks and the archive; `psg figure --all --archived` puts them in.

**Evidence.** [`docs/archify/para.json`](docs/archify/para.json), written by `psg figure` from the
demo brain of `just figure-para`; the drawing is archify's.

</details>

Diagrams in these docs follow the `figure` skill (`.claude/skills/figure`): sources in
`docs/archify/`, `just figure` draws them with the vendored archify (`just figure-init` once),
`just figure-check` says whether a PNG is stale.

## Server

Needs Rust 1.94 or newer. `rust-toolchain.toml` pins the version, so with
[rustup](https://rustup.rs) the first `cargo` or `just` command installs it; with an older
standalone toolchain run `rustup update` first.

```sh
cargo loco start      # API on http://localhost:5150
cargo test
```

## Drawing

The drawing at the top is rendered from its Desmos expressions by `cargo run -p pirsig-drawings --bin
pirsig-art`: see [`docs/drawings/README.md`](docs/drawings/README.md). The graph beside it loads its
picture from Desmos; this repository does not copy it.

## Use cases

[`docs/use-cases.md`](docs/use-cases.md) builds a brain out of one person's GitHub repositories:
a PARA seed script and a sample slip box in [`examples/slipbox/`](examples/slipbox/).
