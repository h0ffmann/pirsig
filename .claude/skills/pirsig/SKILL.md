---
name: pirsig
description: Drive Pirsig, the PARA and Zettelkasten second brain - capture and link notes, file areas, projects, tasks and resources, ingest arXiv or RSS, move to or from Obsidian, run `psg ready`. Use when the person asks to save, find, link or organise something in their brain, or says "Pirsig" or "psg".
---

# Pirsig: working in the second brain

Pirsig keeps two things apart. **PARA** (areas, projects, tasks, resources, archive) is *where the
work is*. The **slip-box** (notes with ids like `1`, `1a`, `1a1`) is *what the person thinks*.
They meet by link, never by folder. Run `psg` from the shell; inside the zettelkasten repository,
`cargo run -q --bin psg --` works too. Add `--json` to any command when you need to read its
output.

**The person's brain is theirs.** Write only what they said, in their words. Never invent a
note's content, a source or a link reason. Ask before any delete (`-d`); offer archiving
first. Nothing leaves the brain unless they ask: `psg rss out` and `psg obsidian export` are
explicit steps.

## Recipes

| They want | Run |
|---|---|
| to jot something down | `psg note -c "the thought"` (a fleeting note, in the inbox) |
| a note in their own words | `psg note -c -t "A claim, as a sentence" -k permanent -b "..."` |
| what a source says | `psg note -c -t "..." -k literature -r RESOURCE_ID` (or `--source "@citekey"`) |
| to continue a train of thought | `psg note -c -t "..." -a 21` (files it as `21a`) |
| to connect two notes | `psg link 21a 3 -w "why they belong together"` |
| to find something | `psg note QUERY`, `psg note -s ZID` (with links and backlinks) |
| to work the inbox | `psg box inbox`; turn each note into literature or permanent, or delete it with their OK |
| an area / project / task | `psg area -c -n NAME`; `psg project -c -n NAME -a AREA --due DATE`; `psg task -c -t TITLE -p PROJECT` |
| a project to use a note | `psg project --pull ID --note ZID -w why` |
| a paper | `psg arxiv add 2212.12794 --note`; `psg arxiv search "query"` |
| a feed | `psg rss add URL -a AREA`, then `psg rss pull` |
| their GitHub repos | `GH_TOKEN=$(gh auth token) psg bootstrap gh --dry-run`, then without `--dry-run`; `--private minimal` only if they ask for private repos |
| Obsidian | `psg obsidian export ./vault`, `psg obsidian import ./vault --dry-run` |
| to know what needs attention | `psg ready` |
| a mind that works as a rhizome | the `hippo` skill (`psg hippo`) |

## How to work

1. Read before you write: `psg note -s ZID`, `psg para` or `psg ready` shows what is there.
2. One idea per note. When the person gives you three ideas, propose three notes and wait.
3. Every link carries a reason, said in a sentence ("contradicts", "an instance of", "the
   mechanism behind").
4. `psg` answers a broken rule with `field: reason`. Read the reason to the person and fix the
   input. Never work around a rule: they keep the brain sound (a literature note needs a
   source, nothing new goes into the archive, a delete with children needs `--force`).
5. After a batch of changes, run `psg ready` and report what changed in one or two lines.

For a review of whether notes follow the method, use the `zk-method` skill.
