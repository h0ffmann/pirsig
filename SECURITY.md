# Security

## Reporting a vulnerability

Please do not open a public issue for a security problem. Report it privately through GitHub:
**Security → Report a vulnerability** on this repository (GitHub's private vulnerability
reporting), or write to [@h0ffmann](https://github.com/h0ffmann) on GitHub. You will get an
answer within a week.

Say what an attacker can do, the steps to reproduce it, and the version or commit.

## What is in scope

- The `psg` CLI, the REST API, the desktop app (`desktop/`) and the importers (`src/ingest/`).
- Anything that could publish what a brain keeps private: `#private` notes leaking through
  `psg fork export`, a public note linking a private one, or a token written to disk.
- The GitHub App manifest (`github-app/`) and its permissions.

## Supported versions

Pirsig is young: only the latest commit of `main` gets fixes.

## Keeping your own copy safe

- Keep your brain in a **private** repository (see the README).
- `config/production.yaml` takes every secret from the environment and refuses to boot without
  them. The default `JWT_SECRET` in `config/development.yaml` is for one machine: set your own
  before anyone else can reach the server.
- Tokens (`GH_TOKEN`, `GITHUB_TOKEN`) are read from the environment and never stored.
