//! The Rust core of obsidian-painter, an Obsidian plugin with three features:
//! Desmos drawings rendered offline ([`desmos`]), archify figures shown in the
//! note and composed into one HTML page ([`archify`], [`compose`]), and a PDF
//! through pandoc ([`pandoc`]).
//!
//! The plugin's TypeScript only touches Obsidian and the machine (files,
//! `node`, `pandoc`); everything else is here, behind one call:
//! [`call`]`(op, json) -> json`, exported to WebAssembly by [`ffi`].
#![allow(clippy::missing_errors_doc)]
pub mod archify;
pub mod compose;
pub mod desmos;
pub mod latex;
pub mod pandoc;

#[cfg(target_arch = "wasm32")]
mod ffi;

use serde::Deserialize;
use serde_json::{json, Value};

fn input<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, String> {
    serde_json::from_str(s).map_err(|e| format!("bad input: {e}"))
}

#[derive(Deserialize)]
struct Archify {
    page: String,
    #[serde(default)]
    extra_css: String,
}

#[derive(Deserialize)]
struct Finalize {
    ir: String,
    script: String,
    src: String,
    page: String,
    receipts: String,
}

/// One operation, JSON in and JSON out.
///
/// | op | input | output |
/// |---|---|---|
/// | `desmos` | `{source}` | `{svg, skipped}` |
/// | `archify_args` | `{ir, script, src, page, receipts}` | `[arg...]` for `node` |
/// | `archify` | `{page, extra_css}` | `{css, svg}` |
/// | `standalone` | `{css, svg}` | `"<svg...>"` with its styles inside |
/// | `sized` | `{svg, scale}` | `{svg, width, height}`, sized for a raster |
/// | `id` | `{lang, source}` | the block's cache key |
/// | `blocks` | `{markdown}` | `[{id, lang, source}]` |
/// | `prepare` | `{markdown, target, images}` | `"markdown"` |
/// | `pdf_args` | `{input, output, engine, resource_path, extra}` | `[arg...]` for `pandoc` |
/// | `compose` | `{title, body, figures}` | `"<!doctype html>..."` |
///
/// # Errors
/// An unknown op, input that does not fit the op, or what the op reports.
pub fn call(op: &str, input_json: &str) -> Result<Value, String> {
    #[derive(Deserialize)]
    struct Source {
        source: String,
    }
    #[derive(Deserialize)]
    struct Markdown {
        markdown: String,
    }
    Ok(match op {
        "desmos" => json!(desmos::render(&input::<Source>(input_json)?.source)),
        "archify_args" => {
            let f: Finalize = input(input_json)?;
            json!(archify::finalize_args(
                &f.ir,
                &f.script,
                &f.src,
                &f.page,
                &f.receipts
            )?)
        }
        "archify" => {
            let a: Archify = input(input_json)?;
            json!(archify::fragment(&a.page, &a.extra_css)?)
        }
        "standalone" => {
            let f: compose::FragmentIn = input(input_json)?;
            json!(archify::standalone(&f.into()))
        }
        "sized" => {
            #[derive(Deserialize)]
            struct Sized {
                svg: String,
                scale: f64,
            }
            let s: Sized = input(input_json)?;
            let (svg, width, height) = archify::sized(&s.svg, s.scale)?;
            json!({ "svg": svg, "width": width, "height": height })
        }
        "id" => {
            #[derive(Deserialize)]
            struct Id {
                lang: String,
                source: String,
            }
            let i: Id = input(input_json)?;
            json!(pandoc::id(&i.lang, &i.source))
        }
        "blocks" => json!(pandoc::blocks(&input::<Markdown>(input_json)?.markdown)),
        "prepare" => json!(pandoc::prepare(&input(input_json)?)),
        "pdf_args" => json!(pandoc::pdf_args(&input(input_json)?)),
        "compose" => json!(compose::page(input(input_json)?)),
        op => return Err(format!("unknown op {op}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_dispatch_and_report() {
        let v = call("desmos", r#"{"source":"y=x"}"#).unwrap();
        assert!(v["svg"].as_str().unwrap().starts_with("<svg"));
        assert!(call("nope", "{}").unwrap_err().contains("unknown op"));
        assert!(call("desmos", "{}").unwrap_err().contains("bad input"));
    }
}
