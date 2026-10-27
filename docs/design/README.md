# Design notes

How ztk is meant to grow. Each note is a proposal: it says what we build, why, and what
stays out. Code that implements a note links back to it.

| Note | Channel | Summary |
|---|---|---|
| [Private by default: private fork + `ztk publish`](private-fork-and-publish.md) | outbound | Keep the brain in a private copy of this repo; make things public only through a terraform-style plan/apply. |
| [Pandoc: papers and citations](pandoc.md) | outbound | Turn notes and resources into paper-quality documents with real citations, without the draft ever leaving your machine. |
| [`ztk.dev.git`: git as an inbound channel](git-inbound.md) | inbound | Pull repositories and their READMEs into the brain as resources and atomic notes, linked to each other. |
| [`ztk bootstrap gh`: your GitHub FOSS work](github-bootstrap.md) | inbound | Sign in with the ztk GitHub App and index your repos, public forks and org contributions (private ones metadata-only) as the brain's first layer. |

**Inbound** channels bring things into the brain (git, web clipper, Obsidian import).
**Outbound** channels take things out of it (publish, pandoc, Obsidian export). Inbound
never needs permission: it only adds private material. Outbound always goes through an
explicit, reviewable step, because that is where personal data can leak.

Integrations are named `ztk.<domain>.<tool>` (`ztk.dev.git`, `ztk.pub.pandoc`,
`ztk.pkm.obsidian`), so the CLI, config keys and docs share one namespace.
