# Changelog

What changes in Pirsig, newest first. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [0.1.0] - unreleased

The first version meant to be copied, as a GitHub template.

### Added

- PARA (areas, projects, tasks, resources, the archive) and a Zettelkasten slip-box (zids,
  kinds, links with reasons, backlinks, the Folgezettel tree), the same through the `psg` CLI,
  the REST API and the desktop app.
- The desktop app (Tauri): the whole brain from one window, with the REST API inside it.
- Hippos (rhizomes of notes grown around fixations) and Plasticity (quick exercises).
- Channels: GitHub (`psg gh`, with depth control), arXiv, RSS in and out, Obsidian in and out.
- The fork state: `brain/`, `psg fork export|import`, `psg gh --overwrite`.
- `psg ready`: the brain checked against its methods.
- Template support: `just adopt`, `just template-check` and the readiness survey in
  `docs/template-readiness.md`.

### Changed

- The app is Pirsig and its CLI `psg` (formerly `ztk`); old vaults with `ztk:` keys still import.
- Licensed under the Mozilla Public License 2.0.
