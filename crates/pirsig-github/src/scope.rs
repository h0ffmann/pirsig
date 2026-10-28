//! What to index, as written by the user in `config/*.local.yaml`:
//!
//! ```yaml
//! github:
//!   user: h0ffmann
//!   forks: true            # public forks you own, linked `fork-of` their upstream
//!   own_private: skip      # your private repos: skip | minimal
//!   orgs:
//!     - name: ouroboric-dev
//!       private: minimal   # private repos there you contributed to: skip | minimal
//!     - name: picasso-labs
//! ```

use serde::{Deserialize, Serialize};

/// How to treat private repositories in a part of the scope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Private {
    /// Leave them out entirely (the default).
    #[default]
    Skip,
    /// Index name, URL, language and your contribution counts; nothing else.
    Minimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Org {
    pub name: String,
    #[serde(default)]
    pub private: Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    /// The GitHub login whose work is indexed.
    pub user: String,
    /// Include public forks owned by `user`.
    #[serde(default = "yes")]
    pub forks: bool,
    /// Private repositories owned by `user`.
    #[serde(default)]
    pub own_private: Private,
    /// Organisations whose repositories `user` contributed to.
    #[serde(default)]
    pub orgs: Vec<Org>,
}

const fn yes() -> bool {
    true
}

impl Scope {
    /// The defaults for a user: own public repos and public forks, no orgs.
    #[must_use]
    pub fn for_user(user: &str) -> Self {
        Self {
            user: user.to_string(),
            forks: true,
            own_private: Private::Skip,
            orgs: Vec::new(),
        }
    }

    /// The org entry for `owner`, if the user listed it (case-insensitive,
    /// like GitHub logins).
    #[must_use]
    pub fn org(&self, owner: &str) -> Option<&Org> {
        self.orgs
            .iter()
            .find(|o| o.name.eq_ignore_ascii_case(owner))
    }

    #[must_use]
    pub fn is_user(&self, owner: &str) -> bool {
        self.user.eq_ignore_ascii_case(owner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_config_block_with_defaults() {
        let scope: Scope = serde_json::from_value(serde_json::json!({
            "user": "h0ffmann",
            "orgs": [{"name": "ouroboric-dev", "private": "minimal"}, {"name": "picasso-labs"}]
        }))
        .unwrap();
        assert!(scope.forks);
        assert_eq!(scope.own_private, Private::Skip);
        assert_eq!(
            scope.org("Ouroboric-Dev").unwrap().private,
            Private::Minimal
        );
        assert_eq!(scope.org("picasso-labs").unwrap().private, Private::Skip);
        assert!(scope.org("elsewhere").is_none());
    }
}
