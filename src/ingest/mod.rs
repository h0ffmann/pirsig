//! Inbound channels that bring sources into the brain as PARA resources:
//! `pirsig.web.rss` (feeds), `pirsig.sci.arxiv` (papers) and `pirsig.dev.github`
//! (repositories).
//!
//! Inbound never needs permission (it only adds private material), so these
//! write straight into the brain. Fetching and parsing are split: the parsers
//! are pure and tested on fixtures, `fetch` is the only network call.
pub mod arxiv;
pub mod github;
pub mod rss;

use std::time::Duration;

use loco_rs::{Error, Result};

pub(crate) const USER_AGENT: &str = concat!(
    "pirsig/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/h0ffmann/pirsig)"
);

/// GET `url` and return the body.
///
/// # Errors
/// When the request fails or the status is not 2xx.
pub async fn fetch(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| Error::Message(format!("http client: {e}")))?;
    let res = client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::Message(format!("{url}: {e}")))?;
    let status = res.status();
    if !status.is_success() {
        return Err(Error::Message(format!("{url}: HTTP {status}")));
    }
    let bytes = res
        .bytes()
        .await
        .map_err(|e| Error::Message(format!("{url}: {e}")))?;
    Ok(bytes.to_vec())
}

/// HTML (or plain text) as one line of readable text: tags dropped, the
/// common entities decoded, whitespace collapsed.
#[must_use]
pub fn plain_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    let out = out
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&ensp;", " ")
        .replace("&emsp;", " ")
        .replace("&middot;", "·")
        .replace("&bull;", "•")
        .replace("&mdash;", "—")
        .replace("&reg;", "®")
        .replace("&copy;", "©")
        .replace("&trade;", "™")
        .replace("&amp;", "&");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `&`, `<`, `>`, `"` and `'` escaped for XML text and attributes.
#[must_use]
pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_from_html() {
        assert_eq!(
            plain_text("<p>Waves &amp; <b>buoys</b></p>\n\n<p>2&nbsp;m</p>"),
            "Waves & buoys 2 m"
        );
        assert_eq!(xml_escape("a<b & \"c\""), "a&lt;b &amp; &quot;c&quot;");
    }
}
