//! Notes to pandoc, for a PDF that reads like a paper.
//!
//! [`blocks`] finds the figures to render (`archify` and `desmos` fences).
//! [`prepare`] rewrites the note for pandoc: each figure becomes an image (PDF)
//! or a slot for [`crate::compose`] (HTML), and Obsidian's own syntax becomes
//! plain Markdown. [`pdf_args`] is the pandoc command line.
use std::{collections::HashMap, fmt::Write};

use serde::{Deserialize, Serialize};

pub const LANGS: [&str; 2] = ["archify", "desmos"];

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Block {
    /// Stable for the same content: a cache key and a file name.
    pub id: String,
    pub lang: String,
    pub source: String,
}

/// FNV-1a over the language and the trimmed body: small, stable, and the same
/// whether the body comes from the file or from Obsidian's code block
/// processor (which drops the last newline).
#[must_use]
pub fn id(lang: &str, source: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in lang.bytes().chain([0]).chain(source.trim_end().bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// A fenced block: where it starts and ends in the text, its language, its body.
struct Fence<'a> {
    start: usize,
    end: usize,
    lang: &'a str,
    body: &'a str,
}

fn fences(md: &str) -> Vec<Fence<'_>> {
    let mut out = Vec::new();
    let mut pos = 0;
    let mut open: Option<(usize, &str, &str, usize)> = None; // start, marker, lang, body start
    for line in md.split_inclusive('\n') {
        let t = line.trim_start();
        match open {
            None => {
                if let Some(m) = ["```", "~~~"].into_iter().find(|m| t.starts_with(m)) {
                    let lang = t[m.len()..].trim();
                    open = Some((pos, m, lang, pos + line.len()));
                }
            }
            Some((start, m, lang, body)) => {
                if t.trim_end() == m
                    || (t.starts_with(m)
                        && t[m.len()..]
                            .trim()
                            .chars()
                            .all(|c| c == m.chars().next().unwrap_or('`')))
                {
                    out.push(Fence {
                        start,
                        end: pos + line.len(),
                        lang,
                        body: &md[body..pos],
                    });
                    open = None;
                }
            }
        }
        pos += line.len();
    }
    out
}

/// The figures in a note.
#[must_use]
pub fn blocks(md: &str) -> Vec<Block> {
    fences(md)
        .into_iter()
        .filter(|f| LANGS.contains(&f.lang))
        .map(|f| Block {
            id: id(f.lang, f.body),
            lang: f.lang.to_string(),
            source: f.body.to_string(),
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct Prepare {
    pub markdown: String,
    /// `pdf`: figures become images; `html`: slots for composition.
    pub target: String,
    /// Figure id → image path (for `pdf`).
    #[serde(default)]
    pub images: HashMap<String, String>,
}

/// The note, ready for pandoc.
#[must_use]
pub fn prepare(p: &Prepare) -> String {
    let md = &p.markdown;
    let mut out = String::with_capacity(md.len());
    let mut last = 0;
    for f in fences(md) {
        out.push_str(&obsidian_to_pandoc(&md[last..f.start]));
        if LANGS.contains(&f.lang) {
            let fid = id(f.lang, f.body);
            if p.target == "html" {
                // a raw block: pandoc passes it through untouched
                let _ = write!(
                    out,
                    "\n```{{=html}}\n<div data-painter=\"{fid}\"></div>\n```\n\n"
                );
            } else if let Some(img) = p.images.get(&fid) {
                let _ = write!(out, "\n![]({}){{width=100%}}\n\n", img.replace(' ', "%20"));
            }
        } else {
            out.push_str(&md[f.start..f.end]);
        }
        last = f.end;
    }
    out.push_str(&obsidian_to_pandoc(&md[last..]));
    out
}

/// Obsidian's syntax in prose (never inside code) as plain Markdown:
/// `[[a|b]]` → `b`, `[[a#h]]` → `a`, `![[img.png]]` → an image, `![[note]]` →
/// its name, `%%comments%%` dropped, `==mark==` → a highlight span, and a
/// callout's `[!type] Title` → a bold title.
fn obsidian_to_pandoc(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    // comments first: they may span lines
    let mut rest = text;
    while let Some(a) = rest.find("%%") {
        s.push_str(&rest[..a]);
        match rest[a + 2..].find("%%") {
            Some(b) => rest = &rest[a + 2 + b + 2..],
            None => {
                rest = "";
            }
        }
    }
    s.push_str(rest);
    let mut out = String::with_capacity(s.len());
    for line in s.split_inclusive('\n') {
        out.push_str(&line_to_pandoc(line));
    }
    out
}

fn line_to_pandoc(line: &str) -> String {
    let mut line = line.to_string();
    // > [!note] Title  →  > **Title**
    if let Some(rest) = line.trim_start().strip_prefix("> [!") {
        if let Some(close) = rest.find(']') {
            let title = rest[close + 1..]
                .trim()
                .trim_start_matches(['+', '-'])
                .trim();
            let kind = &rest[..close];
            let shown = if title.is_empty() { kind } else { title };
            line = format!("> **{shown}**\n");
        }
    }
    let mut out = String::new();
    let mut rest = line.as_str();
    while let Some(a) = rest.find("[[") {
        let embed = a > 0 && rest[..a].ends_with('!');
        let before = if embed { &rest[..a - 1] } else { &rest[..a] };
        let Some(b) = rest[a + 2..].find("]]") else {
            break;
        };
        out.push_str(before);
        let inner = &rest[a + 2..a + 2 + b];
        let (target, label) = inner
            .split_once('|')
            .map_or((inner, None), |(t, l)| (t, Some(l)));
        let target_text = target.split(['#', '^']).next().unwrap_or(target).trim();
        let is_image = [".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".pdf"]
            .iter()
            .any(|e| target_text.to_lowercase().ends_with(e));
        if embed && is_image {
            let _ = write!(out, "![]({})", target_text.replace(' ', "%20"));
        } else {
            out.push_str(label.unwrap_or(target_text).trim());
        }
        rest = &rest[a + 2 + b + 2..];
    }
    out.push_str(rest);
    // ==mark== → [mark]{.mark}
    let mut marked = String::new();
    let mut parts = out.split("==");
    if let Some(first) = parts.next() {
        marked.push_str(first);
        let rest: Vec<&str> = parts.collect();
        let mut i = 0;
        while i < rest.len() {
            if i + 1 < rest.len() && !rest[i].is_empty() {
                let _ = write!(marked, "[{}]{{.mark}}{}", rest[i], rest[i + 1]);
                i += 2;
            } else {
                marked.push_str("==");
                marked.push_str(rest[i]);
                i += 1;
            }
        }
    }
    marked
}

#[derive(Debug, Deserialize)]
pub struct PdfOptions {
    pub input: String,
    pub output: String,
    /// `xelatex` (default), `lualatex`, `tectonic`, `typst`, `weasyprint`, ...
    #[serde(default)]
    pub engine: Option<String>,
    /// Folders pandoc looks in for images.
    #[serde(default)]
    pub resource_path: Vec<String>,
    /// Anything else, one argument per item.
    #[serde(default)]
    pub extra: Vec<String>,
}

/// pandoc's arguments for a PDF that reads like a paper: real typesetting,
/// sane margins, coloured links, numbered sections off by default.
#[must_use]
pub fn pdf_args(o: &PdfOptions) -> Vec<String> {
    let engine = o
        .engine
        .clone()
        .filter(|e| !e.trim().is_empty())
        .unwrap_or_else(|| "xelatex".into());
    let latex = matches!(
        engine.as_str(),
        "xelatex" | "lualatex" | "pdflatex" | "tectonic" | "latexmk"
    );
    let mut args = vec![
        o.input.clone(),
        "--from=markdown+mark+wikilinks_title_after_pipe-auto_identifiers+auto_identifiers".into(),
        "--standalone".into(),
        format!("--pdf-engine={engine}"),
        "-o".into(),
        o.output.clone(),
    ];
    if !o.resource_path.is_empty() {
        args.push(format!(
            "--resource-path={}",
            o.resource_path.join(if cfg!(windows) { ";" } else { ":" })
        ));
    }
    if latex {
        for v in [
            "geometry:margin=2.5cm",
            "fontsize=11pt",
            "colorlinks=true",
            "linkcolor=blue",
            "urlcolor=blue",
            "linestretch=1.15",
        ] {
            args.push("-V".into());
            args.push(v.into());
        }
    }
    args.extend(o.extra.iter().cloned());
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTE: &str = "---\ntitle: T\n---\n# Waves\nSee [[1a Rogue|the rogue note]] and [[Draupner#1995]]. %%private%%\n![[buoy.png]] ==key==\n> [!warning] Mind the gap\n```desmos\ny=x\n```\n```rust\nlet [[a]] = 1;\n```\n~~~archify\n{\"diagram_type\":\"flowchart\"}\n~~~\n";

    #[test]
    fn finds_the_figures() {
        let b = blocks(NOTE);
        assert_eq!(b.len(), 2);
        assert_eq!(
            (b[0].lang.as_str(), b[0].source.as_str()),
            ("desmos", "y=x\n")
        );
        assert_eq!(b[1].lang, "archify");
        assert_eq!(
            b[0].id,
            id("desmos", "y=x"),
            "the processor's body has no last newline"
        );
        assert_ne!(b[0].id, b[1].id);
    }

    #[test]
    fn prepares_for_pdf_and_html() {
        let b = blocks(NOTE);
        let pdf = prepare(&Prepare {
            markdown: NOTE.into(),
            target: "pdf".into(),
            images: [
                (b[0].id.clone(), "/tmp/a b.png".into()),
                (b[1].id.clone(), "/tmp/c.png".into()),
            ]
            .into(),
        });
        assert!(
            pdf.starts_with("---\ntitle: T\n---\n"),
            "front matter is pandoc's metadata"
        );
        assert!(pdf.contains("See the rogue note and Draupner."));
        assert!(!pdf.contains("private"));
        assert!(pdf.contains("![](buoy.png) [key]{.mark}"));
        assert!(pdf.contains("> **Mind the gap**"));
        assert!(
            pdf.contains("![](/tmp/a%20b.png){width=100%}")
                && pdf.contains("![](/tmp/c.png){width=100%}")
        );
        assert!(pdf.contains("let [[a]] = 1;"), "code is left alone");
        let html = prepare(&Prepare {
            markdown: NOTE.into(),
            target: "html".into(),
            images: HashMap::new(),
        });
        assert!(html.contains(&format!("<div data-painter=\"{}\"></div>", b[0].id)));
    }

    #[test]
    fn pdf_args_read_like_a_paper() {
        let a = pdf_args(&PdfOptions {
            input: "n.md".into(),
            output: "n.pdf".into(),
            engine: None,
            resource_path: vec!["/v".into()],
            extra: vec!["--toc".into()],
        });
        assert!(
            a.contains(&"--pdf-engine=xelatex".to_string())
                && a.contains(&"geometry:margin=2.5cm".to_string())
        );
        assert_eq!(a.last().unwrap(), "--toc");
        let t = pdf_args(&PdfOptions {
            input: "n.md".into(),
            output: "n.pdf".into(),
            engine: Some("typst".into()),
            resource_path: vec![],
            extra: vec![],
        });
        assert!(
            !t.contains(&"-V".to_string()),
            "LaTeX variables only for LaTeX engines"
        );
    }
}
