//! `pirsig.sci.arxiv`: papers in, smartly.
//!
//! Any way of naming a paper works (`1706.03762`, `arXiv:1706.03762v5`, an
//! `abs/` or `pdf/` URL, old ids like `hep-th/9901001`). Each paper becomes one
//! `paper` resource: deduplicated by its arXiv id (a new version updates it in
//! place), given a BibTeX citekey (`vaswani2017attention`) so literature notes
//! can cite it, and filed into the one area its arXiv categories point to when
//! that is unambiguous. `--note` also drops a fleeting "Read:" note in the
//! inbox, where processing it into a literature note is the method's next step.
#![allow(clippy::missing_errors_doc)]
use loco_rs::{model::ModelResult, Error, Result};
use sea_orm::{
    prelude::Date, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    TransactionTrait,
};
use serde::Serialize;

use super::{fetch, plain_text};
use crate::{
    dtos::{
        common::Patch,
        notes::CreateNote,
        resources::{CreateResource, UpdateResource},
    },
    models::{areas, notes, projects, resources},
};

const API: &str = "https://export.arxiv.org/api/query";

/// The arXiv id in `input`, without its version: `1706.03762`, `hep-th/9901001`.
#[must_use]
pub fn parse_id(input: &str) -> Option<String> {
    let s = input.trim();
    let s = s
        .strip_prefix("arXiv:")
        .or_else(|| s.strip_prefix("arxiv:"))
        .unwrap_or(s);
    let s = ["/abs/", "/pdf/", "/html/"]
        .iter()
        .find_map(|m| s.find(m).map(|i| &s[i + m.len()..]))
        .unwrap_or(s);
    let s = s.split(['?', '#']).next().unwrap_or(s);
    let s = s.trim_end_matches('/').trim_end_matches(".pdf");
    let base = strip_version(s);
    let new_style = base.split_once('.').is_some_and(|(yymm, num)| {
        yymm.len() == 4
            && yymm.chars().all(|c| c.is_ascii_digit())
            && (4..=5).contains(&num.len())
            && num.chars().all(|c| c.is_ascii_digit())
    });
    let old_style = base.split_once('/').is_some_and(|(archive, num)| {
        !archive.is_empty()
            && archive
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-' || c == '.' || c.is_ascii_uppercase())
            && num.len() == 7
            && num.chars().all(|c| c.is_ascii_digit())
    });
    (new_style || old_style).then(|| base.to_string())
}

/// `1706.03762v7` → `1706.03762`.
fn strip_version(id: &str) -> &str {
    match id.rfind('v') {
        Some(i) if i > 0 && id[i + 1..].chars().all(|c| c.is_ascii_digit()) && i + 1 < id.len() => {
            &id[..i]
        }
        _ => id,
    }
}

/// One paper, as the arXiv API describes it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    #[serde(rename = "abstract")]
    pub summary: String,
    pub published: Option<Date>,
    /// Primary category first (`physics.ao-ph`, `cs.GT`, ...).
    pub categories: Vec<String>,
}

impl Paper {
    #[must_use]
    pub fn url(&self) -> String {
        format!("https://arxiv.org/abs/{}", self.id)
    }

    #[must_use]
    pub fn pdf(&self) -> String {
        format!("https://arxiv.org/pdf/{}", self.id)
    }

    /// `vaswani2017attention`: first author's family name, year, first word
    /// of the title that is not a stopword — the usual BibTeX shape.
    #[must_use]
    pub fn citekey(&self) -> String {
        const STOP: [&str; 18] = [
            "a", "an", "the", "on", "of", "in", "for", "to", "and", "with", "is", "are", "from",
            "towards", "toward", "via", "by", "at",
        ];
        let family = self
            .authors
            .first()
            .and_then(|a| a.split_whitespace().last())
            .map(ascii_word)
            .filter(|w| !w.is_empty())
            .unwrap_or_else(|| "anon".into());
        let year = self
            .published
            .map(|d| d.format("%Y").to_string())
            .unwrap_or_default();
        let word = self
            .title
            .split(|c: char| !c.is_alphanumeric())
            .map(ascii_word)
            .find(|w| !w.is_empty() && !STOP.contains(&w.as_str()))
            .unwrap_or_default();
        format!("{family}{year}{word}")
    }

    /// The resource body: who, when, where it sits in arXiv, and the abstract.
    fn body(&self) -> String {
        let date = self.published.map(|d| d.to_string()).unwrap_or_default();
        format!(
            "**Authors:** {}  \n**Published:** {date} · **Categories:** {}  \n**PDF:** <{}>\n\n> {}\n",
            self.authors.join(", "),
            self.categories.join(", "),
            self.pdf(),
            self.summary
        )
    }
}

/// Lowercase ASCII letters and digits only, accents folded (`Gödel` → `godel`).
fn ascii_word(w: &str) -> String {
    w.chars()
        .filter_map(|c| {
            let c = c.to_lowercase().next().unwrap_or(c);
            let folded = match c {
                'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' => 'i',
                'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
                'ú' | 'ù' | 'û' | 'ü' => 'u',
                'ç' => 'c',
                'ñ' => 'n',
                'ß' => 's',
                c => c,
            };
            folded.is_ascii_alphanumeric().then_some(folded)
        })
        .collect()
}

/// Parse an arXiv API (Atom) response. arXiv reports a bad query as an entry
/// whose id is an error URL; that becomes an error here.
pub fn parse(bytes: &[u8]) -> Result<Vec<Paper>> {
    let feed = feed_rs::parser::parse(bytes)
        .map_err(|e| Error::Message(format!("arXiv answered something that is not Atom: {e}")))?;
    let mut papers = Vec::new();
    for e in feed.entries {
        if e.id.contains("/api/errors") {
            let why = e.summary.map(|s| s.content).unwrap_or_default();
            return Err(Error::Message(format!("arXiv: {}", why.trim())));
        }
        let Some(id) = parse_id(&e.id) else {
            continue;
        };
        papers.push(Paper {
            id,
            title: e.title.map(|t| plain_text(&t.content)).unwrap_or_default(),
            authors: e
                .authors
                .into_iter()
                .map(|a| a.name.trim().to_string())
                .collect(),
            summary: e
                .summary
                .map(|s| plain_text(&s.content))
                .unwrap_or_default(),
            published: e.published.map(|d| d.date_naive()),
            categories: e.categories.into_iter().map(|c| c.term).collect(),
        });
    }
    Ok(papers)
}

/// Look papers up by id (any form `parse_id` accepts), in one request.
pub async fn fetch_ids(inputs: &[String]) -> Result<Vec<Paper>> {
    let ids: Vec<String> = inputs
        .iter()
        .map(|i| parse_id(i).ok_or_else(|| Error::Message(format!("not an arXiv id: {i}"))))
        .collect::<Result<_>>()?;
    let url = format!("{API}?id_list={}&max_results={}", ids.join(","), ids.len());
    let papers = parse(&fetch(&url).await?)?;
    if papers.len() < ids.len() {
        let missing: Vec<&String> = ids
            .iter()
            .filter(|id| !papers.iter().any(|p| &p.id == *id))
            .collect();
        return Err(Error::Message(format!(
            "arXiv has no paper {}",
            missing
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    Ok(papers)
}

/// Search arXiv (its query syntax: `ti:wave AND cat:physics.ao-ph`, or plain words).
pub async fn search(query: &str, max: u32) -> Result<Vec<Paper>> {
    let q: String = url::form_urlencoded::byte_serialize(query.trim().as_bytes()).collect();
    let q = if query.contains(':') {
        q
    } else {
        format!("all:{q}")
    };
    let url = format!("{API}?search_query={q}&max_results={max}&sortBy=relevance");
    parse(&fetch(&url).await?)
}

/// arXiv categories and the area names they belong to. An area is suggested
/// when its name is one of these words (case-insensitive).
const AREAS_BY_CATEGORY: &[(&str, &[&str])] = &[
    (
        "physics.ao-ph",
        &["weather", "oceanography", "climate", "atmosphere"],
    ),
    (
        "physics.flu-dyn",
        &["oceanography", "fluid dynamics", "zamm", "mechanics"],
    ),
    ("physics.geo-ph", &["oceanography", "geophysics"]),
    ("physics.class-ph", &["zamm", "mechanics"]),
    ("math.AP", &["zamm"]),
    ("math.NA", &["zamm", "numerical analysis"]),
    ("cs.NA", &["zamm", "numerical analysis"]),
    ("cs.GT", &["game theory"]),
    ("econ.TH", &["game theory", "economics"]),
    ("cs.MA", &["game theory"]),
    ("cs.LG", &["machine learning"]),
    ("stat.ML", &["machine learning"]),
    ("cs.CL", &["machine learning", "nlp"]),
];

/// Areas (id, name) that `categories` point to, among the active ones.
pub async fn suggest_areas<C: ConnectionTrait>(
    db: &C,
    categories: &[String],
) -> ModelResult<Vec<(i64, String)>> {
    let active = areas::Entity::listing(false).all(db).await?;
    let mut found: Vec<(i64, String)> = Vec::new();
    for cat in categories {
        let words = AREAS_BY_CATEGORY
            .iter()
            .find(|(c, _)| c == cat)
            .map_or(&[][..], |(_, w)| *w);
        for a in &active {
            if words.contains(&a.name.to_lowercase().as_str())
                && !found.iter().any(|(id, _)| *id == a.id)
            {
                found.push((a.id, a.name.clone()));
            }
        }
    }
    Ok(found)
}

/// Where ingested papers go.
#[derive(Debug, Default, Clone)]
pub struct Filing {
    pub area_id: Option<i64>,
    pub project_id: Option<i64>,
    /// Also put a fleeting "Read:" note in the inbox.
    pub note: bool,
}

#[derive(Debug, Serialize)]
pub struct Ingested {
    pub id: String,
    pub resource_id: i64,
    pub title: String,
    pub citekey: Option<String>,
    pub created: bool,
    /// The area it was filed in, when filed.
    pub area: Option<String>,
    /// Areas it could go in, when the categories pointed at more than one.
    pub could_go_in: Vec<String>,
    /// zid of the inbox note, when one was made.
    pub note: Option<String>,
}

/// A citekey no other resource holds: `key`, else `keyb`, `keyc`, ...
async fn free_citekey<C: ConnectionTrait>(db: &C, key: &str) -> ModelResult<String> {
    for suffix in std::iter::once(String::new()).chain(('b'..='z').map(String::from)) {
        let candidate = format!("{key}{suffix}");
        let taken = resources::Entity::find()
            .filter(resources::Column::Citekey.eq(&candidate))
            .count(db)
            .await?;
        if taken == 0 {
            return Ok(candidate);
        }
    }
    Ok(format!("{key}-{}", chrono::Utc::now().timestamp()))
}

/// File `papers` as resources (see the module docs for what "smart" means).
pub async fn store<C: ConnectionTrait + TransactionTrait>(
    db: &C,
    papers: Vec<Paper>,
    filing: &Filing,
) -> ModelResult<Vec<Ingested>> {
    let mut out = Vec::new();
    for paper in papers {
        let external_id = format!("arxiv:{}", paper.id);
        let mut area_id = filing.area_id;
        let mut could_go_in = Vec::new();
        if let Some(pid) = filing.project_id {
            area_id = Some(projects::Model::find(db, pid).await?.area_id);
        } else if area_id.is_none() {
            let found = suggest_areas(db, &paper.categories).await?;
            if let [(id, _)] = found.as_slice() {
                area_id = Some(*id);
            } else {
                could_go_in = found.into_iter().map(|(_, name)| name).collect();
            }
        }
        let existing = resources::Entity::by_external_id(db, &external_id).await?;
        let created = existing.is_none();
        let resource = if let Some(r) = existing {
            // a new version: refresh what arXiv says, keep where it was filed
            let params = UpdateResource {
                title: Some(paper.title.clone()),
                url: Some(paper.url()).into(),
                body: Some(paper.body()).into(),
                kind: Some("paper".to_string()).into(),
                area_id: if r.area_id.is_none() && r.project_id.is_none() {
                    area_id.into()
                } else {
                    Patch::Keep
                },
                project_id: if r.project_id.is_none() {
                    filing.project_id.into()
                } else {
                    Patch::Keep
                },
                citekey: Patch::Keep,
            };
            r.update_with(db, params).await?
        } else {
            let citekey = free_citekey(db, &paper.citekey()).await?;
            let params = CreateResource {
                title: paper.title.clone(),
                url: Some(paper.url()),
                body: Some(paper.body()),
                area_id,
                project_id: filing.project_id,
                kind: Some("paper".into()),
                citekey: Some(citekey),
                external_id: Some(external_id),
            };
            resources::ActiveModel::create(db, params).await?
        };
        let note = if filing.note && resource.notes(db).await?.is_empty() {
            let params = CreateNote {
                title: format!("Read: {}", paper.title),
                resource_id: Some(resource.id),
                ..Default::default()
            };
            Some(notes::ActiveModel::create(db, params).await?.zid)
        } else {
            None
        };
        let area = match resource.area_id {
            Some(id) => Some(areas::Model::find(db, id).await?.name),
            None => None,
        };
        out.push(Ingested {
            id: paper.id,
            resource_id: resource.id,
            title: resource.title.clone(),
            citekey: resource.citekey.clone(),
            created,
            area,
            could_go_in,
            note,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ATOM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
  <title>arXiv Query</title>
  <id>http://arxiv.org/api/x</id>
  <updated>2026-10-27T00:00:00-04:00</updated>
  <entry>
    <id>http://arxiv.org/abs/1706.03762v7</id>
    <updated>2023-08-02T00:41:18Z</updated>
    <published>2017-06-12T17:57:34Z</published>
    <title>Attention Is All
  You Need</title>
    <summary>  The dominant sequence transduction models are based on complex recurrent networks.
</summary>
    <author><name>Ashish Vaswani</name></author>
    <author><name>Noam Shazeer</name></author>
    <link href="http://arxiv.org/abs/1706.03762v7" rel="alternate" type="text/html"/>
    <arxiv:primary_category term="cs.CL"/>
    <category term="cs.CL" scheme="http://arxiv.org/schemas/atom"/>
    <category term="cs.LG" scheme="http://arxiv.org/schemas/atom"/>
  </entry>
</feed>"#;

    #[test]
    fn ids_in_every_shape() {
        for input in [
            "1706.03762",
            "1706.03762v7",
            "arXiv:1706.03762",
            "https://arxiv.org/abs/1706.03762v5",
            "https://arxiv.org/pdf/1706.03762v5.pdf",
            "http://export.arxiv.org/abs/1706.03762?context=cs",
        ] {
            assert_eq!(parse_id(input).as_deref(), Some("1706.03762"), "{input}");
        }
        assert_eq!(
            parse_id("hep-th/9901001v2").as_deref(),
            Some("hep-th/9901001")
        );
        assert_eq!(parse_id("2410.12345").as_deref(), Some("2410.12345"));
        assert_eq!(parse_id("not an id"), None);
        assert_eq!(parse_id("17.06"), None);
    }

    #[test]
    fn parses_papers_and_makes_citekeys() {
        let papers = parse(ATOM.as_bytes()).unwrap();
        assert_eq!(papers.len(), 1);
        let p = &papers[0];
        assert_eq!(p.id, "1706.03762");
        assert_eq!(p.title, "Attention Is All You Need");
        assert_eq!(p.authors, ["Ashish Vaswani", "Noam Shazeer"]);
        assert_eq!(p.categories, ["cs.CL", "cs.LG"]);
        assert_eq!(p.published.unwrap().to_string(), "2017-06-12");
        assert_eq!(p.citekey(), "vaswani2017attention");
        let godel = Paper {
            authors: vec!["Kurt Gödel".into()],
            title: "On the Theory".into(),
            ..p.clone()
        };
        assert_eq!(godel.citekey(), "godel2017theory");

        let err = r#"<feed xmlns="http://www.w3.org/2005/Atom"><title>x</title><id>x</id><updated>2026-10-27T00:00:00Z</updated>
<entry><id>http://arxiv.org/api/errors#incorrect_id_format_for_1</id><title>Error</title><summary>incorrect id format for 1</summary><updated>2026-10-27T00:00:00Z</updated></entry></feed>"#;
        assert!(parse(err.as_bytes()).is_err());
    }
}
