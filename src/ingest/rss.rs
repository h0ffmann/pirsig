//! `pirsig.web.rss`: feeds in (each entry becomes a resource) and the slipbox out
//! (an RSS 2.0 feed of notes).
#![allow(clippy::missing_errors_doc)]
use std::fmt::Write;

use loco_rs::{model::ModelResult, Error, Result};
use sea_orm::{ConnectionTrait, DatabaseConnection, QuerySelect, TransactionTrait};
use serde::Serialize;

use super::{fetch, plain_text, xml_escape};
use crate::{
    dtos::{common::Patch, resources::CreateResource, resources::UpdateResource},
    models::{feeds, notes, resources},
};

/// One feed entry, ready to become a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// `rss:<entry id>`: what a later pull matches on. An entry that links to
    /// an arXiv paper is `arxiv:<id>` instead, the same resource `psg arxiv add`
    /// makes, so a paper met in a feed and added by hand is filed once.
    pub external_id: String,
    /// `paper` for arXiv entries, `article` for the rest.
    pub kind: &'static str,
    pub title: String,
    pub url: Option<String>,
    pub summary: Option<String>,
}

/// Parse RSS 0.9x/1.0/2.0, Atom or JSON Feed: the feed's title and its entries.
pub fn parse(bytes: &[u8]) -> Result<(Option<String>, Vec<Item>)> {
    let feed =
        feed_rs::parser::parse(bytes).map_err(|e| Error::Message(format!("not a feed: {e}")))?;
    let items = feed
        .entries
        .into_iter()
        .map(|e| {
            let url = e
                .links
                .iter()
                .find(|l| l.rel.as_deref().is_none_or(|r| r == "alternate"))
                .or_else(|| e.links.first())
                .map(|l| l.href.clone())
                .filter(|u| u.starts_with("http://") || u.starts_with("https://"));
            let summary = e
                .summary
                .map(|t| t.content)
                .or_else(|| e.content.and_then(|c| c.body))
                .map(|s| plain_text(&s))
                .filter(|s| !s.is_empty());
            let title = e
                .title
                .map(|t| plain_text(&t.content))
                .filter(|t| !t.is_empty())
                .or_else(|| url.clone())
                .unwrap_or_else(|| "(untitled)".to_string());
            let arxiv = url
                .as_deref()
                .filter(|u| u.contains("arxiv.org/abs/"))
                .and_then(super::arxiv::parse_id);
            Item {
                external_id: arxiv
                    .as_ref()
                    .map_or_else(|| format!("rss:{}", e.id), |id| format!("arxiv:{id}")),
                kind: if arxiv.is_some() { "paper" } else { "article" },
                title,
                url,
                summary,
            }
        })
        .collect();
    Ok((feed.title.map(|t| plain_text(&t.content)), items))
}

#[derive(Debug, Default, Serialize)]
pub struct PullReport {
    pub feed_id: i64,
    pub title: Option<String>,
    pub new: usize,
    pub updated: usize,
}

/// File `items` as resources under the feed's area and project: new entries
/// are created, known ones (same `external_id`) get their title and link
/// refreshed, keeping anything written in their body.
pub async fn store<C: ConnectionTrait + TransactionTrait>(
    db: &C,
    feed: &feeds::Model,
    items: Vec<Item>,
) -> ModelResult<(usize, usize)> {
    let (mut new, mut updated) = (0, 0);
    // the project decides the area, so the two never disagree
    let area_id = match feed.project_id {
        Some(pid) => Some(crate::models::projects::Model::find(db, pid).await?.area_id),
        None => feed.area_id,
    };
    for item in items {
        if let Some(r) = resources::Entity::by_external_id(db, &item.external_id).await? {
            let params = UpdateResource {
                title: Some(item.title),
                url: item.url.into(),
                body: if r.body.is_none() {
                    item.summary.into()
                } else {
                    Patch::Keep
                },
                ..Default::default()
            };
            r.update_with(db, params).await?;
            updated += 1;
        } else {
            let params = CreateResource {
                title: item.title,
                url: item.url,
                body: item.summary,
                area_id,
                project_id: feed.project_id,
                kind: Some(item.kind.into()),
                citekey: None,
                external_id: Some(item.external_id),
            };
            resources::ActiveModel::create(db, params).await?;
            new += 1;
        }
    }
    Ok((new, updated))
}

/// Fetch the feed, file its entries, and note the pull.
pub async fn pull(db: &DatabaseConnection, feed: feeds::Model) -> Result<PullReport> {
    let bytes = fetch(&feed.url).await?;
    let (title, items) = parse(&bytes)?;
    let (new, updated) = store(db, &feed, items).await.map_err(crate::cli::explain)?;
    let feed = feed.fetched(db, title).await?;
    Ok(PullReport {
        feed_id: feed.id,
        title: feed.title,
        new,
        updated,
    })
}

/// The slipbox out: notes as an RSS 2.0 feed, newest first. `link` is where
/// the brain is served (`http://localhost:5150`); each item links to its
/// Markdown there.
#[must_use]
pub fn render(title: &str, link: &str, items: &[(notes::Model, Vec<String>)]) -> String {
    let link = link.trim_end_matches('/');
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rss version=\"2.0\">\n<channel>\n");
    let _ = writeln!(xml, "<title>{}</title>", xml_escape(title));
    let _ = writeln!(xml, "<link>{}</link>", xml_escape(link));
    xml.push_str("<description>Notes from a Pirsig slipbox</description>\n");
    xml.push_str("<generator>Pirsig</generator>\n");
    if let Some((newest, _)) = items.first() {
        let _ = writeln!(
            xml,
            "<lastBuildDate>{}</lastBuildDate>",
            newest.updated_at.to_rfc2822()
        );
    }
    for (n, tags) in items {
        xml.push_str("<item>\n");
        let _ = writeln!(xml, "<title>{}</title>", xml_escape(&n.title));
        let _ = writeln!(
            xml,
            "<link>{}</link>",
            xml_escape(&format!("{link}/api/notes/{}/markdown", n.zid))
        );
        // the old name stays in the guid: a feed reader would see every note as new
        let _ = writeln!(xml, "<guid isPermaLink=\"false\">ztk:{}</guid>", n.zid);
        let _ = writeln!(xml, "<description>{}</description>", xml_escape(&n.body));
        for t in tags {
            let _ = writeln!(xml, "<category>{}</category>", xml_escape(t));
        }
        let _ = writeln!(xml, "<pubDate>{}</pubDate>", n.created_at.to_rfc2822());
        xml.push_str("</item>\n");
    }
    xml.push_str("</channel>\n</rss>\n");
    xml
}

/// Notes for the feed: one kind (default `permanent`, your own thinking; the
/// inbox never goes out), optionally one tag, newest first.
pub async fn feed_notes<C: ConnectionTrait>(
    db: &C,
    kind: Option<&str>,
    tag: Option<&str>,
    limit: u64,
) -> ModelResult<Vec<(notes::Model, Vec<String>)>> {
    let kind = kind.unwrap_or("permanent");
    if kind == "fleeting" {
        return Err(crate::models::rules::invalid(
            "kind",
            "fleeting notes are the inbox; they never go out",
        ));
    }
    let found = notes::Entity::listing(Some(kind), tag, None)
        .limit(limit)
        .all(db)
        .await?;
    let ids: Vec<i64> = found.iter().map(|n| n.id).collect();
    let mut tags = notes::Entity::tags_of(db, &ids).await?;
    Ok(found
        .into_iter()
        .map(|n| {
            let t = tags.remove(&n.id).unwrap_or_default();
            (n, t)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS: &str = r#"<?xml version="1.0"?>
<rss version="2.0"><channel><title>Ocean &amp; Air</title>
<item><title>Rogue waves, again</title><link>https://example.org/rogue</link>
<guid>https://example.org/rogue</guid><description>&lt;p&gt;Big &lt;b&gt;waves&lt;/b&gt;&lt;/p&gt;</description></item>
<item><link>https://example.org/untitled</link><guid>u2</guid></item>
</channel></rss>"#;

    #[test]
    fn parses_rss_entries() {
        let (title, items) = parse(RSS.as_bytes()).unwrap();
        assert_eq!(title.as_deref(), Some("Ocean & Air"));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Rogue waves, again");
        assert_eq!(items[0].url.as_deref(), Some("https://example.org/rogue"));
        assert_eq!(items[0].summary.as_deref(), Some("Big waves"));
        assert_eq!(items[0].external_id, "rss:https://example.org/rogue");
        assert_eq!(items[1].title, "https://example.org/untitled");
        assert_eq!(items[0].kind, "article");

        let arxiv = r#"<rss version="2.0"><channel><title>physics.ao-ph</title>
<item><title>GraphCast</title><link>https://arxiv.org/abs/2212.12794</link><guid>oai:arXiv.org:2212.12794v2</guid></item>
</channel></rss>"#;
        let (_, items) = parse(arxiv.as_bytes()).unwrap();
        assert_eq!(items[0].external_id, "arxiv:2212.12794");
        assert_eq!(items[0].kind, "paper");
        assert!(parse(b"<html>nope</html>").is_err());
    }
}
