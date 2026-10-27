//! From repository facts to index records.
//!
//! The API layer lists candidates (`GET /user/repos`, `GET /users/{u}/repos`,
//! and `search/issues?q=type:pr author:{u} org:{org}` for contributions) and
//! hands them here as [`RepoFacts`]. [`plan`] decides what is indexed and at
//! which [`Depth`]; [`Record::from_facts`] drops every field the depth does
//! not allow, so a private repository can never carry its description, topics
//! or README into the brain.

use serde::{Deserialize, Serialize};

use crate::scope::{Private, Scope};

/// How the user relates to a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    /// Owned by the user, not a fork.
    Owner,
    /// A fork owned by the user.
    Fork,
    /// Owned by someone else; the user has authored pull requests there.
    Contributor,
}

/// What the API knows about one repository, before any filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoFacts {
    pub owner: String,
    pub name: String,
    pub url: String,
    pub private: bool,
    pub fork: bool,
    /// `owner/name` of the upstream, for forks.
    pub parent: Option<String>,
    pub description: Option<String>,
    pub topics: Vec<String>,
    pub language: Option<String>,
    pub stars: u32,
    /// Pull requests authored by the user (all states).
    pub prs: u32,
    /// Of those, merged.
    pub merged_prs: u32,
    /// Last push to the repository, RFC 3339.
    pub pushed_at: Option<String>,
}

/// How much of a repository goes into the brain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Depth {
    /// Private repositories: name, URL, language, contribution counts, dates.
    Minimal,
    /// Public repositories: everything in [`Record`]. README sections are a
    /// separate, opt-in step (`ztk git import`, see git-inbound.md).
    Metadata,
}

/// One indexed repository: becomes a ztk resource keyed by `key`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// `gh:owner/name`, stable across re-imports.
    pub key: String,
    pub title: String,
    pub url: String,
    pub relation: Relation,
    pub depth: Depth,
    /// Only public repositories may ever be published (`ztk publish`).
    /// Everything starts `private` in the brain regardless.
    pub publishable: bool,
    pub fork_of: Option<String>,
    pub description: Option<String>,
    pub topics: Vec<String>,
    pub language: Option<String>,
    pub stars: Option<u32>,
    pub prs: u32,
    pub merged_prs: u32,
    pub pushed_at: Option<String>,
}

impl Record {
    /// Build a record, keeping only what `depth` allows.
    #[must_use]
    pub fn from_facts(facts: &RepoFacts, relation: Relation, depth: Depth) -> Self {
        let full = depth == Depth::Metadata && !facts.private;
        Self {
            key: format!("gh:{}/{}", facts.owner, facts.name),
            title: format!("{}/{}", facts.owner, facts.name),
            url: facts.url.clone(),
            relation,
            depth: if full {
                Depth::Metadata
            } else {
                Depth::Minimal
            },
            publishable: !facts.private,
            fork_of: if full { facts.parent.clone() } else { None },
            description: if full {
                facts.description.clone()
            } else {
                None
            },
            topics: if full {
                facts.topics.clone()
            } else {
                Vec::new()
            },
            language: facts.language.clone(),
            stars: full.then_some(facts.stars),
            prs: facts.prs,
            merged_prs: facts.merged_prs,
            pushed_at: facts.pushed_at.clone(),
        }
    }

    /// Whether the importer may read file contents (README, docs) for this
    /// record. Never for minimal records.
    #[must_use]
    pub fn may_read_contents(&self) -> bool {
        self.depth == Depth::Metadata
    }
}

/// Decide, for each candidate, whether it is in scope and at which depth.
/// Output is sorted by key and has one record per repository.
#[must_use]
pub fn plan(scope: &Scope, candidates: &[RepoFacts]) -> Vec<Record> {
    let mut out: Vec<Record> = candidates
        .iter()
        .filter_map(|facts| {
            let (relation, private_rule) = if scope.is_user(&facts.owner) {
                let relation = if facts.fork {
                    Relation::Fork
                } else {
                    Relation::Owner
                };
                if facts.fork && !scope.forks {
                    return None;
                }
                (relation, scope.own_private)
            } else {
                // Someone else's repository: only listed orgs, only with a PR.
                let org = scope.org(&facts.owner)?;
                if facts.prs == 0 {
                    return None;
                }
                (Relation::Contributor, org.private)
            };
            let depth = match (facts.private, private_rule) {
                (false, _) => Depth::Metadata,
                (true, Private::Minimal) => Depth::Minimal,
                (true, Private::Skip) => return None,
            };
            Some(Record::from_facts(facts, relation, depth))
        })
        .collect();
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out.dedup_by(|a, b| a.key == b.key);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scope::Org;

    fn repo(owner: &str, name: &str) -> RepoFacts {
        RepoFacts {
            owner: owner.into(),
            name: name.into(),
            url: format!("https://github.com/{owner}/{name}"),
            private: false,
            fork: false,
            parent: None,
            description: Some("what it is".into()),
            topics: vec!["rust".into()],
            language: Some("Rust".into()),
            stars: 3,
            prs: 0,
            merged_prs: 0,
            pushed_at: Some("2026-10-27T03:00:00Z".into()),
        }
    }

    fn scope() -> Scope {
        Scope {
            orgs: vec![
                Org {
                    name: "ouroboric-dev".into(),
                    private: Private::Minimal,
                },
                Org {
                    name: "picasso-labs".into(),
                    private: Private::Skip,
                },
            ],
            ..Scope::for_user("h0ffmann")
        }
    }

    #[test]
    fn own_public_repos_and_forks_are_indexed_with_metadata() {
        let fork = RepoFacts {
            fork: true,
            parent: Some("loco-rs/loco".into()),
            ..repo("h0ffmann", "loco")
        };
        let records = plan(&scope(), &[repo("h0ffmann", "zettelkasten"), fork]);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].relation, Relation::Fork);
        assert_eq!(records[0].fork_of.as_deref(), Some("loco-rs/loco"));
        assert_eq!(records[1].depth, Depth::Metadata);
        assert!(records
            .iter()
            .all(|r| r.publishable && r.may_read_contents()));
    }

    #[test]
    fn forks_can_be_turned_off() {
        let fork = RepoFacts {
            fork: true,
            ..repo("h0ffmann", "loco")
        };
        let scope = Scope {
            forks: false,
            ..scope()
        };
        assert!(plan(&scope, &[fork]).is_empty());
    }

    #[test]
    fn own_private_repos_are_skipped_by_default() {
        let secret = RepoFacts {
            private: true,
            ..repo("h0ffmann", "brain")
        };
        assert!(plan(&scope(), &[secret]).is_empty());
    }

    #[test]
    fn private_org_contributions_are_minimal_and_never_publishable() {
        let private = RepoFacts {
            private: true,
            prs: 5,
            merged_prs: 4,
            ..repo("ouroboric-dev", "core")
        };
        let records = plan(&scope(), &[private]);
        let r = &records[0];
        assert_eq!(
            (r.relation, r.depth),
            (Relation::Contributor, Depth::Minimal)
        );
        assert!(!r.publishable && !r.may_read_contents());
        assert_eq!(
            (r.description.as_ref(), r.topics.len(), r.stars),
            (None, 0, None)
        );
        assert_eq!((r.prs, r.merged_prs), (5, 4));
        assert_eq!(r.language.as_deref(), Some("Rust"));
    }

    #[test]
    fn org_rules_apply_per_org_and_need_a_pr() {
        let skipped = RepoFacts {
            private: true,
            prs: 2,
            ..repo("picasso-labs", "lab")
        };
        let public = RepoFacts {
            prs: 1,
            ..repo("picasso-labs", "toolchains")
        };
        let untouched = repo("ouroboric-dev", "other");
        let unlisted = RepoFacts {
            prs: 9,
            ..repo("rust-lang", "rust")
        };
        let records = plan(&scope(), &[skipped, public, untouched, unlisted]);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].key, "gh:picasso-labs/toolchains");
        assert_eq!(records[0].depth, Depth::Metadata);
    }

    #[test]
    fn a_private_repo_cannot_be_forced_to_metadata() {
        let private = RepoFacts {
            private: true,
            ..repo("h0ffmann", "brain")
        };
        let r = Record::from_facts(&private, Relation::Owner, Depth::Metadata);
        assert_eq!(r.depth, Depth::Minimal);
        assert!(r.description.is_none());
    }
}
