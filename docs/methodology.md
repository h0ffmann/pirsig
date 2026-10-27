# Methodology readiness

ztk runs two methods side by side, plus three channels that feed them. PARA (Tiago Forte)
answers *where the work is*. The Zettelkasten (Niklas Luhmann, as written down by Sönke
Ahrens) answers *what you think*. This page says what each method requires, which of
those rules ztk enforces and which it only reports, and where each rule is tested.

Two commands answer "how ready is my brain?":

```sh
ztk ready            # every check, with what to run to fix it
ztk ready --json     # the same, for scripts (also GET /api/readiness)
```

A check is **ready**, **partial** (it works, but there is a backlog) or **missing** (not
set up yet). Rules the database can hold are *enforced*: breaking one is an error (HTTP
400, or `field: reason` in the CLI). Rules that need judgement are *reported* by
`ztk ready`, because only the owner can fix them.

## Readiness matrix

| Method / channel | Status | Enforced | Reported by `ztk ready` | Tests |
|---|---|---|---|---|
| **PARA** | ready | non-blank names; area names unique; project status `active`/`waiting`/`someday`/`done`; nothing new in an archived area or project; archiving an area archives its projects and resources; deleting a parent with children needs `force`; a resource's area and project agree | deadlines, next actions, done-but-not-archived, overdue tasks, unfiled resources | `tests/requests/para.rs` |
| **Zettelkasten** | ready | Folgezettel ids; literature notes cite a source or a resource; links survive renames; links and tags in code do not count; a project pulls notes in by link and never owns them | inbox older than 14 days, orphan permanent notes, notes over 500 words or with several sections, hub notes, literature without a resource, projects linking no notes | `tests/requests/notes.rs` |
| **Obsidian in/out** (`ztk.pkm.obsidian`) | ready | export replaces only files ztk wrote; links rewritten to file stems and back; import matches notes by zid (else title) and PARA by name; `--dry-run` rolls back | `[[links]]` with no note behind them | `tests/requests/vault.rs` |
| **RSS in** (`ztk.web.rss`) | ready | feed URLs are http(s) and unique; entries matched by guid, so pulls never duplicate; arXiv entries share the paper's identity | feeds not pulled in 7 days, feeds filed nowhere | `tests/requests/ingest.rs` |
| **RSS out** | ready | permanent notes by default; fleeting notes (the inbox) never go out | whether anything is publishable | `tests/requests/ingest.rs` |
| **arXiv in** (`ztk.sci.arxiv`) | ready | any id shape; one resource per paper (a new version updates it); unique BibTeX citekeys; filed into the one area its categories point to, else reported | papers not filed, papers with no literature note | `tests/requests/ingest.rs`, `src/ingest/arxiv.rs` |

Not covered yet: Obsidian's key/value properties as ztk *tags* (see
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

| Kind | Rule in ztk |
|---|---|
| `fleeting` | the default and the inbox; never published |
| `literature` | must name a `source` (book, URL, `@citekey`) or a `resource`, whose citation becomes the source |
| `permanent` | one idea in your own words; `ztk ready` flags orphans and notes that look like several ideas |
| `hub` | a structure note; `structure`, `moc` and `index` from other slip-boxes mean the same |

- **Ids** are Luhmann-style zids (`1`, `1a`, `1a1`) and never change. Titles can change.
  Renaming a note rewrites `[[Old title]]` in the notes that cite it, so their links
  survive their next save.
- **Links** are `[[zid]]` or `[[Title]]` inside sentences. Obsidian's `[[note#heading]]`
  and `[[note^block]]` resolve to the note. Links and `#tags` inside code do not count.
  A hand-made link (`ztk link A B -w why`) carries its reason.
- **PARA and the slipbox meet by link, never by folder.** A literature note points at its
  resource. A project pulls notes in (`ztk project --pull ID --note ZID -w why`). Archiving
  or deleting the project drops the link and never the note.

## Channels

```sh
# Obsidian: PARA as folders, notes flat in Zettelkasten/
ztk obsidian export ./vault --archived
ztk obsidian import ./vault --dry-run          # then without --dry-run

# RSS in: entries become resources under the feed's area or project
ztk rss add https://export.arxiv.org/rss/physics.ao-ph -a 3
ztk rss pull                                   # all feeds; daily from cron
# RSS out: your permanent notes as a feed
ztk rss out -o feed.xml                        # or GET /api/slipbox/feed.xml

# arXiv: papers in, cited and filed
ztk arxiv search 'ti:"rogue waves" AND cat:physics.ao-ph' -n 5
ztk arxiv add 2212.12794 https://arxiv.org/abs/1706.03762 --note
```

`ztk arxiv add` files a paper by its arXiv categories when they point at exactly one of
your areas (`cs.GT` → Game Theory; `math.NA`, `math.AP` or `physics.class-ph` → ZAMM;
`physics.ao-ph` → Weather or Oceanography; `physics.flu-dyn` → Oceanography or ZAMM; ...). When more than one area fits, it leaves the
paper unfiled and names the candidates. `--note` puts a fleeting `Read: <title>` note,
citing the paper, in the inbox. Turning it into a literature note in your own words is
the method's next step, and `ztk ready` counts the papers still waiting for one.

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
- The `zk-method` skill (`.claude/skills/zk-method/SKILL.md`): the checks `ztk ready`
  reports.
