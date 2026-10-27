//! Archify figures. Archify (`tt-a1i/archify`, the tool behind `just figure`)
//! checks a diagram's JSON IR and writes one HTML page per figure. The plugin
//! runs it (that needs node), and this module turns the page into what a
//! note can hold: the page's styles and its SVG, to live in a shadow root so
//! the page's CSS never touches the note.
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Fragment {
    pub css: String,
    pub svg: String,
}

/// The `node` arguments for `archify finalize`, read from the JSON IR the way
/// the justfile does (`diagram_type`, `meta.quality_profile`).
pub fn finalize_args(
    ir: &str,
    script: &str,
    src: &str,
    page: &str,
    receipts: &str,
) -> Result<Vec<String>, String> {
    let v: Value =
        serde_json::from_str(ir).map_err(|e| format!("the archify block is not JSON: {e}"))?;
    let kind = v
        .get("diagram_type")
        .and_then(Value::as_str)
        .ok_or("the archify JSON has no diagram_type")?;
    let quality = v
        .pointer("/meta/quality_profile")
        .and_then(Value::as_str)
        .unwrap_or("showcase");
    Ok([
        script,
        "finalize",
        kind,
        src,
        page,
        "--out-dir",
        receipts,
        "--quality",
        quality,
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect())
}

/// Every `<style>` of the page, plus `extra` (a look such as `picasso.css`),
/// and the page's figure: from its first `<svg` to the last `</svg>`.
pub fn fragment(page: &str, extra: &str) -> Result<Fragment, String> {
    let start = page
        .find("<svg")
        .ok_or("archify wrote a page without an SVG")?;
    let end = page
        .rfind("</svg>")
        .ok_or("archify wrote an SVG that never closes")?
        + "</svg>".len();
    if end <= start {
        return Err("archify wrote an SVG that never closes".into());
    }
    let mut css = String::new();
    let mut rest = page;
    while let Some(open) = rest.find("<style") {
        let after = &rest[open..];
        let Some(gt) = after.find('>') else { break };
        let Some(close) = after.find("</style>") else {
            break;
        };
        css.push_str(&after[gt + 1..close]);
        css.push('\n');
        rest = &after[close + "</style>".len()..];
    }
    css.push_str(extra);
    Ok(Fragment {
        css,
        svg: page[start..end].to_string(),
    })
}

/// The SVG with explicit `width` and `height`: its `viewBox` times `scale`, so
/// it rasterizes sharply. Returns the SVG and the pixel size.
pub fn sized(svg: &str, scale: f64) -> Result<(String, f64, f64), String> {
    let open_end = svg.find('>').ok_or("not an SVG")?;
    let head = &svg[..open_end];
    let vb = head
        .split("viewBox=\"")
        .nth(1)
        .and_then(|r| r.split('"').next())
        .ok_or("the SVG has no viewBox")?;
    let nums: Vec<f64> = vb
        .split([' ', ','])
        .filter_map(|n| n.parse().ok())
        .collect();
    let [_, _, w, h] = nums[..] else {
        return Err(format!("odd viewBox {vb}"));
    };
    let (w, h) = (w * scale, h * scale);
    let mut kept = String::new();
    // drop any width/height the SVG had, then set ours
    let mut rest = head;
    while !rest.is_empty() {
        let next = [" width=\"", " height=\""]
            .iter()
            .filter_map(|k| rest.find(k).map(|i| (i, *k)))
            .min();
        if let Some((i, k)) = next {
            kept.push_str(&rest[..i]);
            let after = &rest[i + k.len()..];
            rest = after.find('"').map_or("", |j| &after[j + 1..]);
        } else {
            kept.push_str(rest);
            rest = "";
        }
    }
    Ok((
        format!(
            "{kept} width=\"{w:.0}\" height=\"{h:.0}\"{}",
            &svg[open_end..]
        ),
        w,
        h,
    ))
}

/// One SVG file that carries its own styles, for a PDF or an image.
#[must_use]
pub fn standalone(f: &Fragment) -> String {
    let style = format!("<style>{}</style>", f.css.replace("]]>", "]] >"));
    match f.svg.find('>') {
        Some(i) => format!("{}{style}{}", &f.svg[..=i], &f.svg[i + 1..]),
        None => f.svg.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = r#"<html><head><style>.n{fill:red}</style><style media="x">.e{stroke:blue}</style></head>
<body><h1>t</h1><svg viewBox="0 0 10 10"><g><svg x="1"></svg></g><rect class="n"/></svg></body></html>"#;

    #[test]
    fn cuts_the_figure_and_its_styles() {
        let f = fragment(PAGE, ".look{}").unwrap();
        assert!(
            f.css.contains(".n{fill:red}")
                && f.css.contains(".e{stroke:blue}")
                && f.css.ends_with(".look{}")
        );
        assert!(f.svg.starts_with("<svg viewBox") && f.svg.ends_with("<rect class=\"n\"/></svg>"));
        let one = standalone(&f);
        assert!(one.starts_with(r#"<svg viewBox="0 0 10 10"><style>"#));
        assert!(fragment("<p>no figure</p>", "").is_err());
    }

    #[test]
    fn sizes_from_the_view_box() {
        let (svg, w, h) = sized(
            r#"<svg xmlns="x" viewBox="0 0 800 600.0" width="100%" class="c"><path/></svg>"#,
            2.0,
        )
        .unwrap();
        assert_eq!((w, h), (1600.0, 1200.0));
        assert_eq!(
            svg,
            r#"<svg xmlns="x" viewBox="0 0 800 600.0" class="c" width="1600" height="1200"><path/></svg>"#
        );
        assert!(sized("<svg><g/></svg>", 1.0).is_err());
    }

    #[test]
    fn finalize_reads_the_ir() {
        let args = finalize_args(
            r#"{"diagram_type":"flowchart","meta":{"quality_profile":"draft"}}"#,
            "a.mjs",
            "s.json",
            "p.html",
            "r",
        )
        .unwrap();
        assert_eq!(
            args,
            [
                "a.mjs",
                "finalize",
                "flowchart",
                "s.json",
                "p.html",
                "--out-dir",
                "r",
                "--quality",
                "draft"
            ]
        );
        assert!(finalize_args("{}", "a", "b", "c", "d").is_err());
    }
}
