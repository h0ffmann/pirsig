//! `ztk bootstrap gh`: a user's GitHub repositories in, one note each.
//!
//! What is indexed, and how deeply, is decided by `ztk_github::index::plan`
//! (design: `docs/design/github-bootstrap.md`): own public repos and public
//! forks with their metadata; own private repos only when asked, and then at
//! minimal depth (name, URL, language, last push; never the description,
//! topics or README). This module plugs the HTTP side and the brain onto it.
//!
//! Every planned repository becomes one `repo` resource keyed by the record's
//! `gh:owner/name` (a repo added by hand is matched by its URL and claimed),
//! and one short literature note that cites it, written only from the record
//! and, with `--readme`, the README's first paragraph. The same words give
//! the repo a category: its `repo/<category>` tag and its area. One hub note
//! per category indexes its repos (own ones, then forks), and a root hub,
//! "GitHub repositories", indexes the categories. Re-runs add only what is new.
#![allow(clippy::missing_errors_doc)]
use std::{fmt::Write, time::Duration};

use loco_rs::{model::ModelResult, Error, Result};
use regex::Regex;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use ztk_github::index::{Depth, Record, Relation, RepoFacts};

use super::{plain_text, USER_AGENT};
use crate::{
    dtos::{
        notes::{CreateNote, UpdateNote},
        resources::CreateResource,
    },
    models::{areas, notes, resources},
};

const API: &str = "https://api.github.com";
/// Title of the hub that indexes the category hubs.
pub const ROOT_HUB: &str = "GitHub repositories";
/// How many repositories are looked up at once (upstream, README).
const PARALLEL: usize = 8;

/// Where a repository goes: its tag, its hub, its area.
#[derive(Debug, PartialEq, Eq)]
pub struct Category {
    /// The `repo/<tag>` tag on its notes.
    pub tag: &'static str,
    /// The hub note is `Repos: <title>`.
    pub title: &'static str,
    /// Area names it is filed under (case-insensitive); the first active one wins.
    pub areas: &'static [&'static str],
    /// Words or phrases in the repo's text that put it here.
    words: &'static [&'static str],
}

impl Category {
    #[must_use]
    pub fn hub(&self) -> String {
        format!("Repos: {}", self.title)
    }
}

const SOFTWARE: &[&str] = &["software", "code"];

/// Scored by [`Repo::category`]; on a tie the earlier one wins.
pub const CATEGORIES: &[Category] = &[
    Category {
        tag: "marola",
        title: "marola",
        areas: &["oceanography"],
        words: &["marola"],
    },
    Category {
        tag: "ouroboric",
        title: "ouroboric",
        areas: SOFTWARE,
        words: &["ouroboric"],
    },
    Category {
        tag: "game-theory",
        title: "game theory",
        areas: &["game theory"],
        words: &["game theory", "nash", "nashkell"],
    },
    Category {
        tag: "essays",
        title: "essays and philosophy",
        areas: &["the essays", "essays"],
        words: &[
            "philosophy",
            "philosophical",
            "essay",
            "essays",
            "modernity",
            "license research",
        ],
    },
    Category {
        tag: "weather",
        title: "weather and hazards",
        areas: &["weather"],
        words: &[
            "weather",
            "disaster",
            "earthquake",
            "tsunami",
            "typhoon",
            "storm",
            "hazard",
        ],
    },
    Category {
        tag: "ocean-waves",
        title: "ocean and wave modelling",
        areas: &["oceanography"],
        words: &[
            "ocean",
            "oceans",
            "wave",
            "waves",
            "wavewatch",
            "ww3",
            "ww4",
            "metocean",
            "xbeach",
            "coastal",
            "hydrodynamic",
            "sea",
            "bathymetry",
        ],
    },
    Category {
        tag: "gpu",
        title: "GPU computing",
        areas: SOFTWARE,
        words: &["gpu", "cuda", "triton", "shader", "shaders"],
    },
    Category {
        tag: "ai-agents",
        title: "AI agents and coding harnesses",
        areas: SOFTWARE,
        words: &[
            "agent",
            "agents",
            "agentic",
            "aider",
            "claude",
            "codex",
            "harness",
            "mcp",
            "chat",
            "chatbot",
            "assistant",
            "assistants",
            "ai coding",
        ],
    },
    Category {
        tag: "ml-llm",
        title: "machine learning and LLMs",
        areas: SOFTWARE,
        words: &[
            "llm",
            "llms",
            "gpt",
            "transformer",
            "language model",
            "language models",
            "machine learning",
            "ml",
            "mlops",
            "neural",
            "deep learning",
            "voice",
            "tts",
            "speech",
            "forecast",
            "forecasting",
            "classification",
            "churn",
            "inference",
            "jupyterlab",
            "text to sound",
            "i18n",
        ],
    },
    Category {
        tag: "languages",
        title: "programming languages and compilers",
        areas: SOFTWARE,
        words: &[
            "compiler",
            "compilers",
            "interpreter",
            "programming language",
            "language",
            "wasm",
            "webassembly",
            "framework",
            "toolkit",
        ],
    },
    Category {
        tag: "applied-math",
        title: "applied mathematics",
        areas: &["zamm"],
        words: &["mathematics", "math", "circuit"],
    },
    Category {
        tag: "finance",
        title: "finance and markets data",
        areas: SOFTWARE,
        words: &[
            "bank",
            "banking",
            "bancos",
            "finance",
            "financial",
            "itaú",
            "itau",
            "wallet",
            "stock",
            "market",
            "markets",
            "cvm",
            "megaleiloes",
            "auction",
            "risk",
            "extratos",
            "diário oficial",
            "profit",
        ],
    },
    Category {
        tag: "scala-fp",
        title: "Scala, Haskell and functional programming",
        areas: SOFTWARE,
        words: &[
            "scala",
            "haskell",
            "functional",
            "akka",
            "zio",
            "fp",
            "monad",
        ],
    },
    Category {
        tag: "games",
        title: "games and retro computing",
        areas: SOFTWARE,
        words: &["game", "games", "guitar", "decompilation", "recompilation"],
    },
    Category {
        tag: "knowledge",
        title: "notes, writing and knowledge tools",
        areas: SOFTWARE,
        words: &[
            "obsidian",
            "notes",
            "zettelkasten",
            "brain",
            "blog",
            "writing",
            "portuguese",
            "português",
            "readme",
            "garden",
            "cheat sheet",
            "book",
            "personal",
        ],
    },
    Category {
        tag: "learning",
        title: "study and career",
        areas: SOFTWARE,
        words: &[
            "course",
            "study",
            "tutorial",
            "learn",
            "interview",
            "job",
            "thesis",
            "undergraduate",
            "exercises",
            "japanese",
        ],
    },
];

/// For a repo no word places: Scala and Haskell code is functional programming…
const FP: &Category = &CATEGORIES[12];
/// …and the rest is apps and tooling.
pub const APPS: &Category = &Category {
    tag: "apps",
    title: "apps, web and tooling",
    areas: SOFTWARE,
    words: &[],
};

/// `text` as lowercase words: split on anything but letters and digits.
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

/// How many times the category's words and phrases occur in `text`.
fn hits(category: &Category, text: &[String]) -> usize {
    category
        .words
        .iter()
        .map(|w| {
            let phrase = words(w);
            text.windows(phrase.len())
                .filter(|win| *win == phrase)
                .count()
        })
        .sum()
}

/// One planned repository, and its README's first paragraph when it was read.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Repo {
    #[serde(flatten)]
    pub record: Record,
    pub readme: Option<String>,
}

impl From<Record> for Repo {
    fn from(record: Record) -> Self {
        Self {
            record,
            readme: None,
        }
    }
}

impl Repo {
    /// The repository's name, without its owner.
    #[must_use]
    pub fn name(&self) -> &str {
        self.record
            .title
            .rsplit_once('/')
            .map_or(self.record.title.as_str(), |(_, n)| n)
    }

    #[must_use]
    pub fn fork(&self) -> bool {
        self.record.relation == Relation::Fork
    }

    #[must_use]
    pub const fn private(&self) -> bool {
        !self.record.publishable
    }

    /// The category whose words occur most: twice for each hit in the repo's
    /// own words (name, description, topics, upstream), once in the README.
    /// Ties go to the earlier category; no hits at all, to the language.
    #[must_use]
    pub fn category(&self) -> &'static Category {
        let r = &self.record;
        let own = words(
            &[
                r.title.as_str(),
                r.description.as_deref().unwrap_or_default(),
                &r.topics.join(" "),
                r.fork_of.as_deref().unwrap_or_default(),
            ]
            .join(" "),
        );
        let readme = words(self.readme.as_deref().unwrap_or_default());
        let mut best: Option<(&'static Category, usize)> = None;
        for c in CATEGORIES {
            let score = 2 * hits(c, &own) + hits(c, &readme);
            if score > best.map_or(0, |(_, s)| s) {
                best = Some((c, score));
            }
        }
        best.map_or(
            match r.language.as_deref() {
                Some("Scala" | "Haskell") => FP,
                _ => APPS,
            },
            |(c, _)| c,
        )
    }

    /// The note body: what the repo is, its facts, how its README opens.
    #[must_use]
    pub fn body(&self) -> String {
        let r = &self.record;
        let mut facts = vec![
            r.language.as_ref().map_or_else(
                || "no primary language".to_string(),
                |l| format!("language {l}"),
            ),
            if self.private() { "private" } else { "public" }.to_string(),
        ];
        facts.push(match (&r.fork_of, self.fork()) {
            (Some(u), _) => format!("fork of [{u}](https://github.com/{u})"),
            (None, true) => "fork".into(),
            (None, false) => "own repository".into(),
        });
        if let Some(p) = r.pushed_at.as_deref().and_then(|p| p.get(..10)) {
            facts.push(format!("last push {p}"));
        }
        if let Some(s) = r.stars.filter(|s| *s > 0) {
            facts.push(format!("{s}★"));
        }
        let what = match (&r.description, r.depth) {
            (Some(d), _) => format!(": {}", inert(d)),
            (None, Depth::Minimal) => " (private, indexed at minimal depth).".into(),
            (None, Depth::Metadata) => " (no description on GitHub).".into(),
        };
        let mut body = format!("**{}**{what}\n\n{}.\n", r.title, facts.join("; "));
        if let Some(lead) = self
            .readme
            .as_ref()
            .filter(|l| Some(*l) != r.description.as_ref())
        {
            let _ = write!(body, "\nREADME opens with: “{}”\n", inert(lead));
        }
        let _ = write!(body, "\nCategory: {}.\n", self.category().title);
        body
    }
}

/// Quoted text with its `[[links]]` and `#tags` in code spans, where they do
/// not count: what GitHub says must not link or tag the note.
fn inert(text: &str) -> String {
    let Ok(token) =
        Regex::new(r"\[\[[^\]\n]*\]\]|(?:^|[\s(])#[\p{L}\p{N}_/-]*\p{L}[\p{L}\p{N}_/-]*")
    else {
        return text.to_string();
    };
    let text = text.replace('`', "");
    token
        .replace_all(&text, |c: &regex::Captures<'_>| {
            let m = &c[0];
            let start = m.find(['[', '#']).unwrap_or(0);
            format!("{}`{}`", &m[..start], &m[start..])
        })
        .into_owned()
}

#[derive(Deserialize)]
struct ApiRepo {
    name: String,
    owner: ApiOwner,
    html_url: String,
    description: Option<String>,
    language: Option<String>,
    #[serde(default)]
    topics: Vec<String>,
    fork: bool,
    private: bool,
    pushed_at: Option<String>,
    #[serde(default)]
    stargazers_count: u32,
    parent: Option<ApiParent>,
}

#[derive(Deserialize)]
struct ApiOwner {
    login: String,
}

#[derive(Deserialize)]
struct ApiParent {
    full_name: String,
}

impl From<ApiRepo> for RepoFacts {
    fn from(r: ApiRepo) -> Self {
        Self {
            owner: r.owner.login,
            name: r.name,
            url: r.html_url,
            private: r.private,
            fork: r.fork,
            parent: r.parent.map(|p| p.full_name),
            description: r
                .description
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty()),
            topics: r.topics,
            language: r.language,
            stars: r.stargazers_count,
            prs: 0,
            merged_prs: 0,
            pushed_at: r.pushed_at,
        }
    }
}

/// Parse a page of GitHub's repository listing.
pub fn parse_repos(bytes: &[u8]) -> Result<Vec<RepoFacts>> {
    let list: Vec<ApiRepo> = serde_json::from_slice(bytes).map_err(|e| {
        Error::Message(format!(
            "GitHub answered something that is not a repo list: {e}"
        ))
    })?;
    Ok(list.into_iter().map(RepoFacts::from).collect())
}

/// The upstream (`owner/name`) in one repository's JSON, when it is a fork.
pub fn parse_upstream(bytes: &[u8]) -> Result<Option<String>> {
    let repo: ApiRepo = serde_json::from_slice(bytes).map_err(|e| {
        Error::Message(format!("GitHub answered something that is not a repo: {e}"))
    })?;
    Ok(repo.parent.map(|p| p.full_name))
}

/// The first real paragraph of a README, as one line of at most 280 characters:
/// headings, tables, code, quotes, rules and badges are skipped.
#[must_use]
pub fn readme_lead(markdown: &str) -> Option<String> {
    let paragraphs = Regex::new(r"\n\s*\n").ok()?;
    let image = Regex::new(r"!\[[^\]]*\]\([^)]*\)").ok()?;
    let link = Regex::new(r"\[([^\]]*)\]\([^)]*\)").ok()?;
    let markdown = markdown.replace("\r\n", "\n");
    let lead = paragraphs.split(&markdown).find_map(|p| {
        let p = p.trim();
        if ["#", "|", "```", "[!", "---", ">", "="]
            .iter()
            .any(|m| p.starts_with(m))
        {
            return None;
        }
        let p = image.replace_all(p, "");
        let p = link.replace_all(&p, "$1");
        let p = plain_text(&p).replace("**", "").replace('`', "");
        (p.chars().count() > 40).then(|| shorten(&p, 280))
    });
    lead
}

fn shorten(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max - 1).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{cut}…")
}

/// GitHub's REST API, with a token when there is one.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    token: Option<String>,
}

impl Client {
    pub fn new(token: Option<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Message(format!("http client: {e}")))?;
        Ok(Self { http, token })
    }

    /// A client with `GH_TOKEN`, else `GITHUB_TOKEN`, else none.
    pub fn from_env() -> Result<Self> {
        let token = ["GH_TOKEN", "GITHUB_TOKEN"]
            .iter()
            .find_map(|k| std::env::var(k).ok().filter(|t| !t.trim().is_empty()));
        Self::new(token)
    }

    /// GET an API path; `None` when GitHub says 404.
    async fn get(&self, path: &str, accept: &str) -> Result<Option<Vec<u8>>> {
        let url = format!("{API}{path}");
        let mut req = self
            .http
            .get(&url)
            .header("Accept", accept)
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Some(t) = &self.token {
            req = req.bearer_auth(t);
        }
        let res = req
            .send()
            .await
            .map_err(|e| Error::Message(format!("{url}: {e}")))?;
        let status = res.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            let hint = if self.token.is_none() && status.as_u16() == 403 {
                " (rate limited: set GH_TOKEN, e.g. GH_TOKEN=$(gh auth token))"
            } else {
                ""
            };
            return Err(Error::Message(format!("{url}: HTTP {status}{hint}")));
        }
        let bytes = res
            .bytes()
            .await
            .map_err(|e| Error::Message(format!("{url}: {e}")))?;
        Ok(Some(bytes.to_vec()))
    }

    async fn json(&self, path: &str) -> Result<Option<Vec<u8>>> {
        self.get(path, "application/vnd.github+json").await
    }

    /// The login the token belongs to.
    async fn login(&self) -> Result<Option<String>> {
        #[derive(Deserialize)]
        struct User {
            login: String,
        }
        if self.token.is_none() {
            return Ok(None);
        }
        let Some(bytes) = self.json("/user").await? else {
            return Ok(None);
        };
        let user: User = serde_json::from_slice(&bytes)
            .map_err(|e| Error::Message(format!("GitHub /user: {e}")))?;
        Ok(Some(user.login))
    }

    /// The user (`user`, else the token's) and the repositories they own:
    /// private ones too when the token is theirs, public forks with their
    /// upstream. These are the facts `plan` decides on.
    pub async fn facts(&self, user: Option<&str>) -> Result<(String, Vec<RepoFacts>)> {
        let login = self.login().await?;
        let (user, own) = match (user, login) {
            (None, Some(l)) => (l, true),
            (Some(u), Some(l)) => (u.to_string(), u.eq_ignore_ascii_case(&l)),
            (Some(u), None) => (u.to_string(), false),
            (None, None) => {
                return Err(Error::Message(
                    "name a GitHub user, or set GH_TOKEN (e.g. GH_TOKEN=$(gh auth token))".into(),
                ))
            }
        };
        let mut facts = Vec::new();
        for page in 1.. {
            let path = if own {
                format!("/user/repos?affiliation=owner&per_page=100&page={page}")
            } else {
                format!("/users/{user}/repos?type=owner&per_page=100&page={page}")
            };
            let Some(bytes) = self.json(&path).await? else {
                return Err(Error::Message(format!("GitHub has no user {user}")));
            };
            let found = parse_repos(&bytes)?;
            let last = found.len() < 100;
            facts.extend(found);
            if last {
                break;
            }
        }
        // the listing leaves out a fork's upstream; private forks never need it
        let forks: Vec<usize> = (0..facts.len())
            .filter(|&i| facts[i].fork && !facts[i].private)
            .collect();
        for chunk in forks.chunks(PARALLEL) {
            let mut set = tokio::task::JoinSet::new();
            for &i in chunk {
                let (client, path) = (
                    self.clone(),
                    format!("/repos/{}/{}", facts[i].owner, facts[i].name),
                );
                set.spawn(async move { (i, client.json(&path).await) });
            }
            for (i, res) in set.join_all().await {
                if let Some(bytes) = res? {
                    facts[i].parent = parse_upstream(&bytes)?;
                }
            }
        }
        Ok((user, facts))
    }

    /// Fill in each README's first paragraph, for the records whose contents
    /// may be read (never a minimal one). A README that cannot be fetched is
    /// left out, not fatal; the failures are returned as `repo: error`.
    pub async fn readmes(&self, repos: &mut [Repo]) -> Vec<String> {
        let mut failed = Vec::new();
        let readable: Vec<usize> = (0..repos.len())
            .filter(|&i| repos[i].record.may_read_contents())
            .collect();
        for chunk in readable.chunks(PARALLEL) {
            let mut set = tokio::task::JoinSet::new();
            for &i in chunk {
                let (client, path) = (
                    self.clone(),
                    format!("/repos/{}/readme", repos[i].record.title),
                );
                set.spawn(
                    async move { (i, client.get(&path, "application/vnd.github.raw").await) },
                );
            }
            for (i, res) in set.join_all().await {
                match res {
                    Ok(Some(bytes)) => {
                        repos[i].readme = readme_lead(&String::from_utf8_lossy(&bytes));
                    }
                    Ok(None) => {}
                    Err(e) => failed.push(format!("{}: {e}", repos[i].record.title)),
                }
            }
        }
        failed
    }
}

/// Where indexed repositories go.
#[derive(Debug, Default, Clone)]
pub struct Filing {
    /// The area for repos whose category has no area in the brain.
    pub area_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Indexed {
    pub repo: String,
    pub resource_id: i64,
    pub created: bool,
    pub category: &'static str,
    pub depth: Depth,
    /// The area it was filed in, when filed.
    pub area: Option<String>,
    /// zid of its note, when this run wrote one.
    pub note: Option<String>,
    pub fork: bool,
    pub private: bool,
}

/// `line` into a hub body: own repos go at the end of `## Own repositories`,
/// forks at the end of `## Forks`, which stays the last section.
fn add_to_hub(body: &str, line: &str, fork: bool) -> String {
    const OWN: &str = "## Own repositories";
    const FORKS: &str = "## Forks";
    let body = body.trim_end();
    if fork {
        let sep = if body.contains(FORKS) {
            "\n"
        } else {
            "\n\n## Forks\n\n"
        };
        return format!("{body}{sep}{line}\n");
    }
    let (head, tail) = body.find(FORKS).map_or((body, ""), |i| body.split_at(i));
    let head = head.trim_end();
    let sep = if head.contains(OWN) {
        "\n"
    } else {
        "\n\n## Own repositories\n\n"
    };
    if tail.is_empty() {
        format!("{head}{sep}{line}\n")
    } else {
        format!("{head}{sep}{line}\n\n{tail}\n")
    }
}

/// The hub note titled `title`, made with `intro` when there is none yet.
async fn hub<C: ConnectionTrait + TransactionTrait>(
    db: &C,
    title: &str,
    intro: &str,
    tags: Vec<String>,
) -> ModelResult<(notes::Model, bool)> {
    let found = notes::Entity::find()
        .filter(notes::Column::Kind.eq("hub"))
        .filter(notes::Column::Title.eq(title))
        .one(db)
        .await?;
    if let Some(n) = found {
        return Ok((n, false));
    }
    let params = CreateNote {
        title: title.to_string(),
        body: format!("{intro}\n"),
        kind: Some("hub".into()),
        tags: Some(tags),
        ..Default::default()
    };
    Ok((notes::ActiveModel::create(db, params).await?, true))
}

async fn rewrite<C: ConnectionTrait + TransactionTrait>(
    db: &C,
    note: notes::Model,
    body: String,
) -> ModelResult<notes::Model> {
    let params = UpdateNote {
        body: Some(body),
        ..Default::default()
    };
    note.update_with(db, params).await
}

/// The repo's resource: the one keyed by its record, else the one with its
/// URL (claimed, and filed if it was not), else a new one.
async fn resource<C: ConnectionTrait>(
    db: &C,
    repo: &Repo,
    area: Option<i64>,
) -> ModelResult<(resources::Model, bool)> {
    let r = &repo.record;
    let existing = match resources::Entity::by_external_id(db, &r.key).await? {
        Some(found) => Some(found),
        None => {
            resources::Entity::find()
                .filter(resources::Column::Url.eq(&r.url))
                .one(db)
                .await?
        }
    };
    let Some(found) = existing else {
        let params = CreateResource {
            title: r.title.clone(),
            url: Some(r.url.clone()),
            body: r.description.clone().or_else(|| repo.readme.clone()),
            area_id: area,
            project_id: None,
            kind: Some("repo".into()),
            citekey: None,
            external_id: Some(r.key.clone()),
        };
        return Ok((resources::ActiveModel::create(db, params).await?, true));
    };
    let claim = found.external_id.is_none();
    let file = found.area_id.is_none() && found.project_id.is_none() && area.is_some();
    if !(claim || file) {
        return Ok((found, false));
    }
    let mut active: resources::ActiveModel = found.into();
    if claim {
        active.external_id = Set(Some(r.key.clone()));
    }
    if file {
        active.area_id = Set(area);
    }
    Ok((active.update(db).await?, false))
}

/// File `repos` as resources with a note each, and index them in the hubs
/// (see the module docs). Running it again only adds what is new.
pub async fn store<C: ConnectionTrait + TransactionTrait>(
    db: &C,
    repos: Vec<Repo>,
    filing: &Filing,
) -> ModelResult<Vec<Indexed>> {
    let active = areas::Entity::listing(false).all(db).await?;
    let mut out = Vec::new();
    for repo in repos {
        let category = repo.category();
        let area = category
            .areas
            .iter()
            .find_map(|name| active.iter().find(|a| a.name.to_lowercase() == *name))
            .map(|a| a.id)
            .or(filing.area_id);
        let (resource, created) = resource(db, &repo, area).await?;

        let note = if resource.notes(db).await?.is_empty() {
            let mut tags = vec![
                "github".to_string(),
                format!("repo/{}", category.tag),
                if repo.fork() { "fork" } else { "own" }.to_string(),
            ];
            if repo.private() {
                tags.push("private".into());
            }
            let params = CreateNote {
                title: format!("Repo: {}", repo.name()),
                body: repo.body(),
                kind: Some("literature".into()),
                resource_id: Some(resource.id),
                tags: Some(tags),
                ..Default::default()
            };
            let note = notes::ActiveModel::create(db, params).await?;

            let (cat_hub, new_hub) = hub(
                db,
                &category.hub(),
                "Every GitHub repository in this category.",
                vec!["github".into(), format!("repo/{}", category.tag)],
            )
            .await?;
            let private = if repo.private() { " (private)" } else { "" };
            let line = format!("- [[{}]] {}{private}", note.zid, repo.name());
            let body = add_to_hub(&cat_hub.body, &line, repo.fork());
            let cat_hub = rewrite(db, cat_hub, body).await?;
            if new_hub {
                let (root, _) = hub(
                    db,
                    ROOT_HUB,
                    "Entry point to the GitHub repositories, one hub per category.",
                    vec!["github".into()],
                )
                .await?;
                let body = format!(
                    "{}\n- [[{}]] {}\n",
                    root.body.trim_end(),
                    cat_hub.zid,
                    cat_hub.title
                );
                rewrite(db, root, body).await?;
            }
            Some(note.zid)
        } else {
            None
        };
        let area = match resource.area_id {
            Some(id) => Some(areas::Model::find(db, id).await?.name),
            None => None,
        };
        out.push(Indexed {
            repo: repo.record.title.clone(),
            resource_id: resource.id,
            created,
            category: category.tag,
            depth: repo.record.depth,
            area,
            note,
            fork: repo.fork(),
            private: repo.private(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use ztk_github::{
        index::plan,
        scope::{Private, Scope},
    };

    use super::*;

    fn facts(name: &str, description: Option<&str>, language: Option<&str>) -> RepoFacts {
        RepoFacts {
            owner: "h0ffmann".into(),
            name: name.into(),
            url: format!("https://github.com/h0ffmann/{name}"),
            private: false,
            fork: false,
            parent: None,
            description: description.map(Into::into),
            topics: vec![],
            language: language.map(Into::into),
            stars: 0,
            prs: 0,
            merged_prs: 0,
            pushed_at: Some("2026-09-01T00:00:00Z".into()),
        }
    }

    fn repo(f: &RepoFacts) -> Repo {
        let scope = Scope {
            own_private: Private::Minimal,
            ..Scope::for_user("h0ffmann")
        };
        plan(&scope, std::slice::from_ref(f)).remove(0).into()
    }

    #[test]
    fn categories_come_from_the_repos_own_words() {
        let tag = |name, desc, lang| repo(&facts(name, desc, lang)).category().tag;
        assert_eq!(
            tag("WW3", Some("WAVEWATCH III"), Some("Fortran")),
            "ocean-waves"
        );
        assert_eq!(
            tag("ww3-dia-gpu", None, Some("Cuda")),
            "ocean-waves",
            "tie: ww3 is earlier"
        );
        assert_eq!(tag("rust-cuda", Some("GPU code in Rust"), None), "gpu");
        assert_eq!(tag("marola-app", Some("the agents"), None), "marola");
        assert_eq!(
            tag("nashkell", Some("game theory + haskell"), None),
            "game-theory"
        );
        assert_eq!(
            tag("research-kit", None, Some("Python")),
            "apps",
            "research is not sea"
        );
        assert_eq!(tag("cvm-x", None, Some("Scala")), "finance");
        assert_eq!(
            tag("happy-path", None, Some("Scala")),
            "scala-fp",
            "by language"
        );
        assert_eq!(
            tag(
                "astrbot_plugin_disaster_warning",
                Some("基于 AstrBot"),
                None
            ),
            "weather"
        );
        let mut writer = facts("ptbr-writer", Some("portuguese on rails"), None);
        writer.topics = vec![
            "claude-code".into(),
            "writing".into(),
            "writing-tool".into(),
        ];
        assert_eq!(repo(&writer).category().tag, "knowledge", "most hits win");
        let mut fork = facts("thing", None, None);
        fork.fork = true;
        fork.parent = Some("ouroboric-dev/thing".into());
        assert_eq!(
            repo(&fork).category().tag,
            "ouroboric",
            "the upstream counts"
        );
    }

    #[test]
    fn a_private_repo_keeps_its_words_out_of_the_note() {
        let mut secret = facts("bank-secrets", Some("our trading strategy"), Some("Scala"));
        secret.private = true;
        secret.topics = vec!["finance".into()];
        let r = repo(&secret);
        assert_eq!(r.record.depth, Depth::Minimal);
        assert!(!r.record.may_read_contents(), "its README is never read");
        let body = r.body();
        assert!(!body.contains("trading"), "{body}");
        assert!(body.starts_with("**h0ffmann/bank-secrets** (private, indexed at minimal depth)."));
        assert_eq!(r.category().tag, "finance", "from the name alone");
    }

    #[test]
    fn readme_lead_skips_badges_and_headings() {
        let md = "# Title\r\n\r\n[![ci](https://x/badge.svg)](https://x)\n\n> a quote that is long enough to count as one\n\n\
                  WAVEWATCH III&reg; is a **community** wave modeling [framework](https://x) that   includes\nthe latest.\n\nmore";
        assert_eq!(
            readme_lead(md).as_deref(),
            Some("WAVEWATCH III® is a community wave modeling framework that includes the latest.")
        );
        assert_eq!(readme_lead("# only a heading\n\nshort"), None);
        let long = "word ".repeat(100);
        let lead = readme_lead(&long).unwrap();
        assert!(lead.chars().count() <= 280 && lead.ends_with("word…"));
    }

    #[test]
    fn parses_the_listing_and_the_upstream() {
        let page = r#"[{"name":"WW3","owner":{"login":"h0ffmann"},"html_url":"https://github.com/h0ffmann/WW3",
            "description":" WAVEWATCH III ","language":"Fortran","topics":["waves"],"fork":true,"private":false,
            "pushed_at":"2026-09-01T00:00:00Z","stargazers_count":2},
            {"name":"x","owner":{"login":"h0ffmann"},"html_url":"https://github.com/h0ffmann/x","description":"",
            "language":null,"fork":false,"private":true,"pushed_at":null}]"#;
        let found = parse_repos(page.as_bytes()).unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].description.as_deref(), Some("WAVEWATCH III"));
        assert_eq!(found[1].description, None, "blank is none");

        let one = r#"{"name":"WW3","owner":{"login":"h0ffmann"},"html_url":"u","fork":true,"private":false,
            "parent":{"full_name":"NOAA-EMC/WW3"}}"#;
        assert_eq!(
            parse_upstream(one.as_bytes()).unwrap().as_deref(),
            Some("NOAA-EMC/WW3")
        );

        let mut ww3 = found[0].clone();
        ww3.parent = Some("NOAA-EMC/WW3".into());
        let mut r = repo(&ww3);
        assert_eq!(r.record.key, "gh:h0ffmann/WW3");
        r.readme = Some("A community wave model.".into());
        assert_eq!(
            r.body(),
            "**h0ffmann/WW3**: WAVEWATCH III\n\nlanguage Fortran; public; fork of \
             [NOAA-EMC/WW3](https://github.com/NOAA-EMC/WW3); last push 2026-09-01; 2★.\n\n\
             README opens with: “A community wave model.”\n\nCategory: ocean and wave modelling.\n"
        );
    }

    #[test]
    fn quoted_text_neither_links_nor_tags() {
        assert_eq!(
            inert("Edit [[Note]] visually, #1 tool for #tags (#obsidian) in C# `x`"),
            "Edit `[[Note]]` visually, #1 tool for `#tags` (`#obsidian`) in C# x"
        );
        let mut r = repo(&facts("x", Some("see [[Home]] #wiki"), None));
        r.readme = Some("A README that links [[Elsewhere]] and tags #things here.".into());
        let body = r.body();
        assert!(crate::models::notes::wikilinks(&body).is_empty(), "{body}");
        assert!(crate::models::notes::hashtags(&body).is_empty(), "{body}");
    }

    #[test]
    fn hubs_keep_own_repos_before_forks() {
        let hub = "Every GitHub repository in this category.\n";
        let hub = add_to_hub(hub, "- [[2]] fork", true);
        assert_eq!(
            hub,
            "Every GitHub repository in this category.\n\n## Forks\n\n- [[2]] fork\n"
        );
        let hub = add_to_hub(&hub, "- [[3]] mine", false);
        let hub = add_to_hub(&hub, "- [[4]] fork2", true);
        let hub = add_to_hub(&hub, "- [[5]] mine2", false);
        assert_eq!(
            hub,
            "Every GitHub repository in this category.\n\n## Own repositories\n\n- [[3]] mine\n- [[5]] mine2\n\n\
             ## Forks\n\n- [[2]] fork\n- [[4]] fork2\n"
        );
    }
}
