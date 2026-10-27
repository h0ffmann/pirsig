# Private by default: private fork + `ztk publish`

## Why

A second brain is personal by construction: health, finances, people, half-formed ideas.
ztk is also code-first and git-friendly, so the natural place for a brain is a git repo.
Put those together carelessly and you get a public GitHub repo full of personal data.

Two rules follow:

1. **Run ztk from a private copy of this repository.** The code is public; your brain
   is not. Nothing about using ztk should need a public repo.
2. **Publishing is a separate, explicit step**, modeled on `terraform plan` / `terraform
   apply`: ztk first shows exactly what would become public, and only an `apply` makes it
   so. Nothing is public unless you said so, note by note.

## The private fork

GitHub does not allow a private fork of a public repository, so the "private fork" is a
private repository that tracks this one as `upstream`:

```sh
# 1. create an empty PRIVATE repo, e.g. github.com/<you>/brain (no README)
git clone https://github.com/h0ffmann/zettelkasten.git brain
cd brain
git remote rename origin upstream
git remote add origin git@github.com:<you>/brain.git
git push -u origin main

# later: take new ztk versions
git pull upstream main
```

What lives where:

| Path | In the private fork | Notes |
|---|---|---|
| `src/`, `migration/`, … | yes, from upstream | never edit unless you contribute back |
| `brain/` | yes, **your data** | Markdown + YAML frontmatter, one file per item (same format as the Obsidian export) |
| `*.sqlite` | no (gitignored) | an index rebuilt from `brain/`; never the source of truth |
| `config/*.local.yaml` | no (gitignored) | machine-local settings and tokens |
| `.ztk/publish.state.json` | yes | what has been published, see below |

Contributing back stays easy: code changes go on a branch, are pushed to a public fork
of `h0ffmann/zettelkasten`, and never touch `brain/`. A pre-push hook (`ztk doctor
--hooks`) refuses to push `brain/` to any remote that is public.

## `ztk publish`: plan, then apply

### Visibility

Every item (area, project, task, resource, note) gets a `visibility`:

| Value | Meaning |
|---|---|
| `private` (default) | never leaves the brain |
| `public` | published as is |
| `redacted` | published with the body replaced by a stub (title and links stay) |

Visibility is inherited down PARA (a public area does **not** make its projects public;
inheritance only goes the other way: a private area forces its children private), set
with `--visibility` on create/update or `visibility:` in frontmatter.

### Commands

```sh
ztk publish                 # = plan: show what would change, change nothing
ztk publish -o next.plan    # save the plan to apply exactly that later
ztk publish --apply         # plan again, show it, ask "yes" before doing anything
ztk publish --apply next.plan   # apply a saved plan; fails if the brain changed since
ztk publish --destroy       # plan + apply removal of everything ztk published
```

Output looks like terraform's, on purpose:

```
ztk will publish to target "site" (git: github.com/<you>/notes, branch gh-pages)

  + note    "Atomic notes beat folders"            (new, public)
  ~ project "Run a marathon"                       (title changed)
  ~ note    "Zettelkasten in Rust"                 (redacted body)
  - note    "Old draft"                            (no longer public)

  ! note "Zettelkasten in Rust" links to private note "Salary 2026":
    the link will be published as plain text, without the target.
  ! note "Atomic notes beat folders": looks like an email address on line 12.

Plan: 1 to add, 2 to change, 1 to destroy. 2 warnings.
```

### How it works

1. **Desired state**: every item with `visibility != private`, rendered by the target's
   renderer (Markdown, HTML, JSON).
2. **Current state**: `.ztk/publish.state.json`, one entry per published item: id,
   target path, content hash, published-at. Like terraform state, it is how ztk knows
   what to update or remove without trusting the remote.
3. **Diff** desired vs. current gives the `+ ~ -` lines.
4. **Leak checks** run on the desired state, and warnings are part of the plan:
   - links from public to private items (published as plain text, never as a dangling
     URL that reveals the private title, unless the link text itself is the title: then
     it is replaced with `[private]`);
   - secret and PII patterns (tokens, emails, phone numbers, IBANs), reusing the same
     detectors as `ztk doctor`;
   - labels starting with `private/` or `@` (smart labels) are stripped.
   `--strict` turns warnings into errors.
5. **Apply** writes the rendered files to the target, then updates the state file. A
   saved plan stores the brain's content hash; applying it after the brain changed
   fails, exactly like a stale terraform plan.

### Targets

Configured in `config/<env>.yaml` under `ztk.publish.targets`:

```yaml
ztk:
  publish:
    targets:
      site:
        kind: git              # commit + push the rendered files
        remote: git@github.com:<you>/notes.git
        branch: gh-pages
        format: markdown
      local:
        kind: dir              # just write files; useful for previews and CI
        path: ./public
```

v0 ships `dir` and `git`. Later: gist, a hosted ztk instance, RSS/ActivityPub.

## Out of scope

- Encryption of the private repo (use GitHub private repos, or git-crypt if you need it).
- Per-person sharing (publish is public-or-not; sharing with someone is a later design).
- Publishing papers: those go through [pandoc](pandoc.md) and stay local unless the
  resulting file is itself marked public.

## Implementation steps

1. `visibility` column on areas, projects, tasks, resources (migration, default `private`),
   `--visibility` on the CLI, `?visibility=` on the API.
2. `brain/` Markdown store as the source of truth (shared with the Obsidian export).
3. `ztk publish` plan/apply with the `dir` target and the state file.
4. Leak checks, `--strict`.
5. `git` target, `--destroy`, saved plans.
6. `ztk doctor --hooks`: pre-push guard for `brain/`.
