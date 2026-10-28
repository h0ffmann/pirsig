# Methodology readiness

Pirsig runs two methods side by side, plus three channels that feed them. PARA (Tiago Forte)
answers *where the work is*. The Zettelkasten (Niklas Luhmann, as written down by Sönke
Ahrens) answers *what you think*. This page says what each method requires, which of
those rules Pirsig enforces and which it only reports, and where each rule is tested.

Two commands answer "how ready is my brain?":

```sh
psg ready            # every check, with what to run to fix it
psg ready --json     # the same, for scripts (also GET /api/readiness)
```

A check is **ready**, **partial** (it works, but there is a backlog) or **missing** (not
set up yet). Rules the database can hold are *enforced*: breaking one is an error (HTTP
400, or `field: reason` in the CLI). Rules that need judgement are *reported* by
`psg ready`, because only the owner can fix them.

## Readiness matrix

| Method / channel | Status | Enforced | Reported by `psg ready` | Tests |
|---|---|---|---|---|
| **PARA** | ready | non-blank names; area names unique; project status `active`/`waiting`/`someday`/`done`; nothing new in an archived area or project; archiving an area archives its projects and resources; deleting a parent with children needs `force`; a resource's area and project agree | deadlines, next actions, done-but-not-archived, overdue tasks, unfiled resources | `tests/requests/para.rs` |
| **Zettelkasten** | ready | Folgezettel ids; literature notes cite a source or a resource; links survive renames; links and tags in code do not count; a project pulls notes in by link and never owns them | inbox older than 14 days, orphan permanent notes, notes over 500 words or with several sections, hub notes, literature without a resource, projects linking no notes | `tests/requests/notes.rs` |
| **Obsidian in/out** (`pirsig.pkm.obsidian`) | ready | export replaces only files Pirsig wrote; links rewritten to file stems and back; import matches notes by zid (else title) and PARA by name; `--dry-run` rolls back | `[[links]]` with no note behind them | `tests/requests/vault.rs` |
| **RSS in** (`pirsig.web.rss`) | ready | feed URLs are http(s) and unique; entries matched by guid, so pulls never duplicate; arXiv entries share the paper's identity | feeds not pulled in 7 days, feeds filed nowhere | `tests/requests/ingest.rs` |
| **RSS out** | ready | permanent notes by default; fleeting notes (the inbox) never go out | whether anything is publishable | `tests/requests/ingest.rs` |
| **Hippo rhizome** | ready | each note is woven in on save (two links, each with a reason); no `--after`; a deleted note's neighbours are relinked | rhizomes in pieces, fixations with no notes | `tests/requests/hippos.rs` |
| **Plasticity** | ready | the plasts are Pirsig's six; levels run 1 to 5; seeds below 2^53; every attempt is recorded | whether any plast was practised this week | `tests/requests/plasticity.rs`, `src/plasticity/` |
| **GitHub in** (`psg gh`) | ready | a resource and a literature note per featured repo (own, pushed in the last `--cold-after` months), keyed `gh:owner/name` (a hand-made one with the same URL is claimed) and linked from its category hub; public forks and cold repos as lines of one index note each; private repos skipped, or minimal (no description, topics or README) with `--private minimal`, cold or forked ones in a `#private` index; text from GitHub never links or tags | through PARA and the Zettelkasten: unfiled repo resources, dangling links | `tests/requests/ingest.rs`, `src/ingest/github.rs`, `crates/pirsig-github` |
| **arXiv in** (`pirsig.sci.arxiv`) | ready | any id shape; one resource per paper (a new version updates it); unique BibTeX citekeys; filed into the one area its categories point to, else reported | papers not filed, papers with no literature note | `tests/requests/ingest.rs`, `src/ingest/arxiv.rs` |

Not covered yet: Obsidian's key/value properties as Pirsig *tags* (see
[labels and tags](design/labels-and-tags.md)), and a review status on notes
(`unverified` → `verified` → `approved`), which [the use cases](use-cases.md) ask for.

## PARA

| Item | Is | Rules |
|---|---|---|
| **Area** | an ongoing responsibility, with no end date | unique name; archiving it archives its active projects and resources; restoring it restores only the area |
| **Project** | an outcome with an end, inside one area | `status` defaults to `active`; `--due` is the end that makes it a project; a project that is `done` should be archived |
| **Task** | the next action inside a project | only in an open project; `--no-due` clears the date |
| **Resource** | reference material: the PARA home of a source | `kind` is `article`, `book`, `paper`, `repo`, `video`, `web` or `other`; `url` is http(s); `citekey` is unique and is how literature notes cite it |
| **Archive** | `archived_at` on areas, projects and resources | nothing active lives in it; to bring a project back, bring back its area first |

Optional fields can be cleared: `null` in JSON, `""` for text flags, and `--no-due`,
`--no-area` or `--no-project` in the CLI. Deleting an area or project that still has
children is refused unless forced (`--force`, or `?force=true`). Archiving is almost
always the better move.

## Zettelkasten

| Kind | Rule in Pirsig |
|---|---|
| `fleeting` | the default and the inbox; never published |
| `literature` | must name a `source` (book, URL, `@citekey`) or a `resource`, whose citation becomes the source |
| `permanent` | one idea in your own words; `psg ready` flags orphans and notes that look like several ideas |
| `hub` | a structure note; `structure`, `moc` and `index` from other slip-boxes mean the same |

- **Ids** are Luhmann-style zids (`1`, `1a`, `1a1`) and never change. Titles can change.
  Renaming a note rewrites `[[Old title]]` in the notes that cite it, so their links
  survive their next save.
- **Links** are `[[zid]]` or `[[Title]]` inside sentences. Obsidian's `[[note#heading]]`
  and `[[note^block]]` resolve to the note. Links and `#tags` inside code do not count.
  A hand-made link (`psg link A B -w why`) carries its reason.
- **PARA and the slipbox meet by link, never by folder.** A literature note points at its
  resource. A project pulls notes in (`psg project --pull ID --note ZID -w why`). Archiving
  or deleting the project drops the link and never the note.

## Hippo rhizome

A Hippo is a name, a description and the fixations its notes gather around. Its notes
follow the rhizome principles of Deleuze and Guattari (*A Thousand Plateaus*, 1980), as far as
software can hold them:

| Principle | In Pirsig |
|---|---|
| connection: any point connects to any other | each note is linked on save to the notes it shares a fixation (strongest), a tag or a title word with, else to the last note captured, until it has two links |
| no root, no trunk | `--after` is refused; the map has no top, only entry points |
| asignifying rupture: broken anywhere, it grows back | deleting a note links its neighbours in a chain; `psg hippo --weave` bridges any pieces |
| multiplicity, plateaus | each fixation is a plateau; `psg hippo --map` lists its notes, and where you left off |

The automatic links carry a `rhizome:` reason, so they can be told from the person's own,
which always win.

## Channels

```sh
# Obsidian: PARA as folders, notes flat in Zettelkasten/
psg obsidian export ./vault --archived
psg obsidian import ./vault --dry-run          # then without --dry-run

# RSS in: entries become resources under the feed's area or project
psg rss add https://export.arxiv.org/rss/physics.ao-ph -a 3
psg rss pull                                   # all feeds; daily from cron
# RSS out: your permanent notes as a feed
psg rss out -o feed.xml                        # or GET /api/slipbox/feed.xml

# GitHub: every repo a resource and a note, indexed by category
GH_TOKEN=$(gh auth token) psg bootstrap gh --dry-run
GH_TOKEN=$(gh auth token) psg bootstrap gh --private minimal --readme -a 6

# arXiv: papers in, cited and filed
psg arxiv search 'ti:"rogue waves" AND cat:physics.ao-ph' -n 5
psg arxiv add 2212.12794 https://arxiv.org/abs/1706.03762 --note
```

`psg arxiv add` files a paper by its arXiv categories when they point at exactly one of
your areas (`cs.GT` → Game Theory; `math.NA`, `math.AP` or `physics.class-ph` → ZAMM;
`physics.ao-ph` → Weather or Oceanography; `physics.flu-dyn` → Oceanography or ZAMM; ...). When more than one area fits, it leaves the
paper unfiled and names the candidates. `--note` puts a fleeting `Read: <title>` note,
citing the paper, in the inbox. Turning it into a literature note in your own words is
the method's next step, and `psg ready` counts the papers still waiting for one.

| | API |
|---|---|
| readiness | `GET /api/readiness` |
| project notes | `GET/POST /api/projects/{id}/notes` `{zid, context}`, `DELETE /api/projects/{id}/notes/{zid}` |
| feeds | `GET/POST /api/feeds` `{url, area_id, project_id}`, `POST /api/feeds/{id}/pull`, `DELETE /api/feeds/{id}` |
| feed out | `GET /api/slipbox/feed.xml?kind=&tag=&limit=` |
| arXiv | `POST /api/ingest/arxiv` `{ids, area_id, project_id, note}`, `GET /api/ingest/arxiv/search?q=&max=` |

## Sources

- Tiago Forte, *Building a Second Brain* (2022): PARA.
- Sönke Ahrens, *How to Take Smart Notes* (2017): fleeting, literature and permanent notes.
- Niklas Luhmann, "Kommunikation mit Zettelkästen" (1981): fixed ids, branching, links over
  hierarchy.
- The `zk-method` skill (`.claude/skills/zk-method/SKILL.md`): the checks `psg ready`
  reports.
