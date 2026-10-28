# `pirsig.dev.git`: git as an inbound channel

## Why

For a developer, the richest record of what they think is already in git: READMEs,
design docs, commit history, the forks they keep. Today that knowledge sits outside the
brain, and the connections between repositories (this library is used by that service,
this README explains an idea another one builds on) are only in your head.

`pirsig.dev.git` makes git an **inbound** channel: it reads repositories and turns them into
resources and atomic notes, linked to each other, inside the private brain. It never
writes to the repositories.

## What it imports

| From git | Into Pirsig |
|---|---|
| a repository | a **resource** (title, URL, description, topics, language, stars, `fork_of`) |
| the README, split at headings | one **note** per section, linked to the repo resource |
| `docs/**/*.md` (opt-in) | notes, same splitting |
| a fork | a link `fork-of` to the upstream repo resource, imported too (metadata only) |
| a URL to another repo in a README | a link `mentions` between the two repo resources |
| a dependency on another of your repos (`Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`) | a link `depends-on` |
| shared GitHub topics | shared labels (see the BYOL labels issue) |

Notes keep a `source:` pointing at `repo@sha:path#heading`, so a re-import updates them in
place and a note can always be traced back to the exact README line it came from.

## Commands

```sh
psg git import https://github.com/h0ffmann/zettelkasten   # one repo (remote or local path)
psg git import gh:h0ffmann                                 # every repo of a user, forks included
psg git import gh:h0ffmann --no-forks --docs               # skip forks, also import docs/
psg git sync                                               # re-import what changed since the last sha
psg git graph gh:h0ffmann                                  # print the repo/idea graph (text, --json, --dot)
```

`gh:` sources use the GitHub API (a token in `config/*.local.yaml` or `GITHUB_TOKEN`
for private repos and rate limits); anything else is a plain `git clone --depth 1` into
a cache under `~/.cache/pirsig/git/`. Each import records the last seen commit sha per
repo, so `sync` is incremental and cheap enough for a daily cron.

## Seeing connections between ideas

The point is not a copy of your READMEs, it is the graph. After an import:

- `psg git graph` shows repos and the edges above (`fork-of`, `mentions`, `depends-on`,
  shared labels);
- README sections become notes that the rest of Pirsig links like any other note, so an
  idea described in two READMEs can be linked by hand or by the label suggester
  (BYOL issue) and shows up next to your own notes;
- the use cases in the README (separate work) walk through this on the author's own
  repositories, including a public fork.

## Privacy

Everything imported is `private`, including content from public repos: importing is
reading, not endorsing for publication. Private repos are only read with an explicit
token. Nothing is ever pushed to a source repository.

## Out of scope for v0

Commit messages and PR discussions as notes, other forges (GitLab, Gitea; the source
trait makes them a small addition), outbound git (that is `psg publish` with the `git`
target).

## Implementation steps

1. A `GitSource` trait (`list_repos`, `fetch_readme`, `fetch_paths`, `head_sha`) with a
   local/clone implementation and a GitHub API implementation.
2. Notes (the zettel entity) with `source`, and typed links between items (`fork-of`,
   `mentions`, `depends-on`). Both are prerequisites shared with the Obsidian import.
3. `psg git import` / `sync`, README splitting, link extraction.
4. `psg git graph` with text, JSON and Graphviz output.
