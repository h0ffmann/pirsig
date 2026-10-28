# `ztk bootstrap gh`: your GitHub FOSS work as the first layer of the brain

## Why

A new brain is empty, and an empty brain is not worth opening. For a developer the
fastest honest seed is the work already on GitHub: the repos you own, the forks you
keep, the orgs you contribute to. The author's own brain is seeded this way, and the
point is that **anyone can replicate it by signing in with their own GitHub account**:
no token to create, no config to write before the first useful screen.

This builds on [git-inbound](git-inbound.md). Bootstrap is the *minimal, everything*
pass (one resource per repo, metadata only); `ztk git import` is the *deep, selected*
pass (README sections as notes) and stays opt-in.

## What is indexed

| Source | Included when | Depth |
|---|---|---|
| your public repos | always | metadata |
| your public forks | `forks: true` (default) | metadata + `fork-of` upstream |
| your private repos | `own_private: minimal` (default `skip`) | minimal |
| public repos in a listed org | you authored at least one PR there | metadata |
| private repos in a listed org | the org's `private: minimal` and the app is installed there | minimal |
| anything else (unlisted orgs, repos you only starred) | never | |

- **metadata**: name, URL, description, topics, language, stars, `fork-of`, your PR and
  merged-PR counts, last push. May be published later.
- **minimal**: name, URL, language, your PR counts, last push. No description, topics,
  stars, README or any file content; `publishable: false` forever.

The rules live in `crates/ztk-github/src/index.rs` (`plan`, `Record::from_facts`) and
are tested there, including that a private repo cannot be forced to metadata depth.

```yaml
# config/development.local.yaml
github:
  user: h0ffmann
  forks: true
  own_private: skip
  orgs:
    - name: ouroboric-dev
      private: minimal
    - name: picasso-labs
```

Each record becomes a ztk **resource** keyed `gh:owner/name` (re-runs update in place),
labelled `gh`, `gh:owner`, `relation:owner|fork|contributor` and the repo's topics.

### What is built: one note per repo, indexed by category

`ztk bootstrap gh` (`src/ingest/github.rs`, `src/cli/bootstrap.rs`) runs `plan` on what
the token can see and writes, for each record:

- a `repo` **resource** keyed by the record's `gh:owner/name`. A resource added by hand
  with the same URL is claimed, not duplicated, and a re-run files one left unfiled;
- one short **literature note**, `Repo: <name>`, citing the resource. It holds only the
  record's fields: description, language, public or private, `fork-of`, last push,
  stars. `--readme` adds the README's first paragraph, read only when
  `may_read_contents()` allows, so never for a minimal record. Text from GitHub is quoted
  in code spans where it would otherwise link (`[[x]]`) or tag (`#x`) the note;
- a **category**, from the words in the name, description, topics and upstream (twice)
  and the README (once): the note gets `#repo/<category>` and the resource the area the
  category names (`Oceanography`, `Weather`, `Game Theory`, `The Essays`, `ZAMM`, else
  `Software` or `Code` when the brain has one, else `-a AREA_ID`);
- a line in the category's **hub note** (`Repos: <category>`, own repos then forks), and
  the hub in the root hub **GitHub repositories**.

```sh
GH_TOKEN=$(gh auth token) ztk bootstrap gh --dry-run      # what would be indexed
GH_TOKEN=$(gh auth token) ztk bootstrap gh --private minimal --readme
```

Re-runs add only what is new. Notes are tagged `github`, `own` or `fork`, `private` and
`repo/<category>` for now; the `gh:owner` and `relation:` labels wait for
[labels and tags](labels-and-tags.md). Org contributions (the PR search) are not built.

## Credentials: a GitHub App with the device flow

Why an App and not a personal access token or an OAuth App:

- **Fine-grained and read-only**: `metadata`, `contents` and `pull_requests`, all
  `read` (see `github-app/manifest.json`). An OAuth App can only ask for `repo`, which
  is full write access to every private repo.
- **The user chooses repos and orgs** at install time, and org owners approve access to
  their private repos. ztk never sees more than both the user and the installation
  allow.
- **Device flow** suits a CLI: `ztk bootstrap gh` prints a code, you enter it at
  `github.com/login/device`, done. It needs only the public client id, so nothing
  secret ships in the binary.
- **Short-lived tokens**: user-to-server tokens expire after 8 hours and are refreshed;
  revocable from GitHub settings.

`GITHUB_TOKEN` (any PAT) still works as an override, for CI and for people who prefer
it; the same `plan` rules apply to whatever it can see.

### API calls

1. `GET /user/installations` and `/user/installations/{id}/repositories`: what the app
   can see.
2. `GET /users/{user}/repos?type=owner` for public repos and forks; `GET /repos/{o}/{r}`
   for a fork's `parent`.
3. Per listed org: `GET /search/issues?q=type:pr author:{user} org:{org}` grouped by
   repository for PR counts (merged via `is:merged`).
4. Turn results into `RepoFacts`, run `plan`, upsert resources. No file contents are
   read in bootstrap; `contents: read` is for the later, opt-in `ztk git import` of
   public repos.

All of it is conditional requests with `ETag`, so `ztk bootstrap gh --sync` from a
daily cron costs almost no rate limit. No webhooks: the app has no server.

## Privacy

- Everything lands `private` in the brain, like every inbound channel.
- Minimal records carry nothing a private org would consider content; they cannot be
  published (`ztk publish` refuses records with `publishable: false`, not just warns).
- The token is stored in the OS keychain, never in the brain or in git.

## Sharing it: "expose my GitHub knowledge"

Once indexed, `ztk publish` can emit the public part (metadata records only) as a page
or a JSON graph: repos, forks, orgs, `fork-of` and `depends-on` edges. That page is the
author's public map; anyone else gets their own by running the same bootstrap.

## Registering the app

Done once by the maintainer with GitHub's manifest flow; steps in
[`github-app/README.md`](../../github-app/README.md).

## Implementation steps

1. ✓ `crates/ztk-github`: manifest (source of `github-app/manifest.json`), `Scope`,
   `plan` and minimisation, with tests.
2. Register the app (maintainer, manual) and compile its client id in.
3. Device flow + token storage (`keyring`), HTTP client with ETag cache. Until then
   `GH_TOKEN` or `GITHUB_TOKEN` is the only credential.
4. ✓ `ztk bootstrap gh [--private minimal] [--readme] [--dry-run]`: list, plan, upsert
   resources, one note per repo, hub notes per category (see above). Still to do: org
   contributions, `--sync` with ETags.
5. `publishable` enforced in `ztk publish`.
