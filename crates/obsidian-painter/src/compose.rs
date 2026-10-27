//! HTML composition: one page from a note and its figures.
//!
//! Pandoc turns the note into HTML with a `<div data-painter="ID"></div>` where
//! each figure goes. Each figure then sits in a declarative shadow root
//! (`<template shadowrootmode="open">`), so every archify page keeps its own
//! CSS and the page stays one self-contained file.
use std::{collections::HashMap, fmt::Write};

use serde::Deserialize;

use crate::archify::Fragment;

#[derive(Debug, Deserialize)]
pub struct Page {
    pub title: String,
    /// The note as HTML (pandoc's `-t html5` output).
    pub body: String,
    /// Figure id → its fragment.
    pub figures: HashMap<String, FragmentIn>,
}

#[derive(Debug, Deserialize)]
pub struct FragmentIn {
    #[serde(default)]
    pub css: String,
    pub svg: String,
}

impl From<FragmentIn> for Fragment {
    fn from(f: FragmentIn) -> Self {
        Self {
            css: f.css,
            svg: f.svg,
        }
    }
}

const LOOK: &str = "body{max-width:52rem;margin:3rem auto;padding:0 1.25rem;font:17px/1.6 Georgia,'Times New Roman',serif;color:#1d1d1f}\
h1,h2,h3{font-family:Inter,system-ui,sans-serif;line-height:1.25}\
figure.painter{margin:2rem 0}figure.painter>div{width:100%}\
pre,code{font-family:ui-monospace,Menlo,monospace;font-size:.9em}pre{overflow-x:auto}\
@media (prefers-color-scheme:dark){body{background:#111;color:#e8e8e8}}";

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Swap every `<div data-painter="ID">…</div>` for `fig`, whatever pandoc put
/// between the two tags.
fn fill(body: &str, id: &str, fig: &str) -> String {
    let open = format!(r#"<div data-painter="{id}">"#);
    let mut out = String::with_capacity(body.len() + fig.len());
    let mut rest = body;
    while let Some(a) = rest.find(&open) {
        let Some(b) = rest[a..].find("</div>") else {
            break;
        };
        out.push_str(&rest[..a]);
        out.push_str(fig);
        rest = &rest[a + b + "</div>".len()..];
    }
    out.push_str(rest);
    out
}

/// The whole page.
#[must_use]
pub fn page(p: Page) -> String {
    let mut body = p.body;
    for (id, f) in p.figures {
        let mut fig = String::new();
        let _ = write!(
            fig,
            r#"<figure class="painter" id="fig-{id}"><div><template shadowrootmode="open"><style>:host{{display:block}}svg{{max-width:100%;height:auto}}{}</style>{}</template></div></figure>"#,
            f.css.replace("</style", "<\\/style"),
            f.svg
        );
        body = fill(&body, &id, &fig);
    }
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>{LOOK}</style></head>\n<body>\n{body}\n</body></html>\n",
        escape(&p.title)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_go_in_their_own_shadow_root() {
        let html = page(Page {
            title: "A <note>".into(),
            body: "<p>before</p><div data-painter=\"ab12\">\n</div><p>after</p>".into(),
            figures: [(
                "ab12".to_string(),
                FragmentIn {
                    css: ".n{}".into(),
                    svg: "<svg></svg>".into(),
                },
            )]
            .into(),
        });
        assert!(html.contains("<title>A &lt;note&gt;</title>"));
        assert!(html.contains(
            r#"<figure class="painter" id="fig-ab12"><div><template shadowrootmode="open"><style>"#
        ));
        assert!(html.contains(".n{}</style><svg></svg></template>"));
        assert!(!html.contains("data-painter"));
    }
}
