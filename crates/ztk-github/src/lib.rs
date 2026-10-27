//! `ztk bootstrap gh`: seed a brain from a user's GitHub FOSS work.
//!
//! Design: `docs/design/github-bootstrap.md`. This crate holds the parts that
//! must not drift and need no network:
//!
//! - [`manifest`]: the GitHub App manifest (permissions, events, URLs), the
//!   single source of `github-app/manifest.json`;
//! - [`scope`]: what the user asked to index (own repos, forks, orgs);
//! - [`index`]: the planner that turns repository facts from the API into
//!   index records, and the minimisation rule that keeps private repositories
//!   down to metadata.
//!
//! The HTTP side (device flow, listing repos, search) plugs in on top and only
//! has to produce [`index::RepoFacts`].

pub mod index;
pub mod manifest;
pub mod scope;
