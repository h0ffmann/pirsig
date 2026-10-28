//! Plasticity: quick exercises that train the brain itself, next to the brain
//! Pirsig stores.
//!
//! A **plast** is one kind of exercise in one domain (mental arithmetic,
//! sequences, argument forms, intervals, classic first lines, Latin words).
//! Every exercise is generated from a `(seed, level)` pair, so it can be shown
//! now and checked later without storing it. For now the plasts are the ones in
//! [`CATALOG`], shipped by Pirsig; a plast's `origin` column leaves room for the
//! person's own.
pub mod abstraction;
pub mod languages;
pub mod literature;
pub mod logic;
pub mod math;
pub mod music;
pub mod rng;

use serde::Serialize;

/// The domains plasts train.
pub const DOMAINS: [&str; 6] = [
    "math",
    "abstraction",
    "logic",
    "music",
    "literature",
    "languages",
];

/// Levels run from 1 (warm-up) to this.
pub const MAX_LEVEL: u8 = 5;

/// What counts as a right answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Number(i64),
    /// Index into `choices`; the choice's text or its letter is accepted.
    Choice(usize),
    /// Any of these, compared after [`normalize`].
    Text(Vec<String>),
}

/// One exercise, ready to show.
#[derive(Debug, Clone, Serialize)]
pub struct Exercise {
    pub plast: &'static str,
    pub seed: u64,
    pub level: u8,
    pub prompt: String,
    /// Multiple choice, when the exercise has choices (answer with a letter or the text).
    pub choices: Vec<String>,
    #[serde(skip)]
    pub answer: Answer,
    /// Shown after answering: why the answer is what it is.
    #[serde(skip)]
    pub explain: String,
}

impl Exercise {
    /// The right answer, as a person reads it.
    #[must_use]
    pub fn expected(&self) -> String {
        match &self.answer {
            Answer::Number(n) => n.to_string(),
            Answer::Choice(i) => format!(
                "{}) {}",
                char::from(b'A' + u8::try_from(*i).unwrap_or(0)),
                self.choices[*i]
            ),
            Answer::Text(forms) => forms.first().cloned().unwrap_or_default(),
        }
    }

    /// Whether `given` is right. Spaces, case, accents and thousands
    /// separators do not count.
    #[must_use]
    pub fn check(&self, given: &str) -> bool {
        let g = normalize(given);
        match &self.answer {
            Answer::Number(n) => g.replace([',', '_', ' '], "").parse::<i64>() == Ok(*n),
            Answer::Choice(i) => {
                let letter = char::from(b'a' + u8::try_from(*i).unwrap_or(0)).to_string();
                g == letter || g == format!("{letter})") || g == normalize(&self.choices[*i])
            }
            Answer::Text(forms) => forms.iter().any(|f| normalize(f) == g),
        }
    }
}

/// Lowercase, trimmed, accents folded, inner whitespace collapsed.
#[must_use]
pub fn normalize(s: &str) -> String {
    let folded: String = s
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A plast Pirsig ships.
pub struct Plast {
    pub slug: &'static str,
    pub domain: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub generate: fn(u64, u8) -> Exercise,
}

/// The plasts provided by Pirsig, one per domain.
pub const CATALOG: [Plast; 6] = [
    Plast {
        slug: "mental-arithmetic",
        domain: "math",
        title: "Mental arithmetic",
        description: "Sums, products, percentages and squares, in your head.",
        generate: math::generate,
    },
    Plast {
        slug: "sequences",
        domain: "abstraction",
        title: "What comes next",
        description: "Find the rule behind a sequence and continue it.",
        generate: abstraction::generate,
    },
    Plast {
        slug: "argument-forms",
        domain: "logic",
        title: "Valid or not",
        description: "Tell valid arguments from the classic fallacies.",
        generate: logic::generate,
    },
    Plast {
        slug: "intervals",
        domain: "music",
        title: "Intervals",
        description: "Semitones, interval names and notes above and below.",
        generate: music::generate,
    },
    Plast {
        slug: "first-lines",
        domain: "literature",
        title: "Who wrote it",
        description: "Famous lines of the classics: name the author or the work.",
        generate: literature::generate,
    },
    Plast {
        slug: "latin-words",
        domain: "languages",
        title: "Latin words",
        description: "Everyday Latin, both ways, the roots of half of Portuguese and English.",
        generate: languages::generate,
    },
];

/// The catalog plast with this slug.
#[must_use]
pub fn plast(slug: &str) -> Option<&'static Plast> {
    CATALOG.iter().find(|p| p.slug == slug)
}

/// Shuffle `right` among `wrong` into four choices; returns them and where
/// `right` landed.
pub(crate) fn choices(rng: &mut rng::Rng, right: &str, wrong: &[&str]) -> (Vec<String>, usize) {
    let mut pool: Vec<&str> = wrong.iter().copied().filter(|w| *w != right).collect();
    pool.sort_unstable();
    pool.dedup();
    rng.shuffle(&mut pool);
    let mut out: Vec<String> = pool.into_iter().take(3).map(str::to_string).collect();
    out.push(right.to_string());
    rng.shuffle(&mut out);
    let at = out.iter().position(|c| c == right).unwrap_or(0);
    (out, at)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_plast_is_deterministic_and_answerable() {
        for p in &CATALOG {
            for level in 1..=MAX_LEVEL {
                for seed in 0..300 {
                    let e = (p.generate)(seed, level);
                    let again = (p.generate)(seed, level);
                    assert_eq!(e.prompt, again.prompt, "{} is not deterministic", p.slug);
                    assert!(!e.prompt.is_empty());
                    let right = match &e.answer {
                        Answer::Number(n) => n.to_string(),
                        Answer::Choice(i) => {
                            assert_eq!(e.choices.len(), 4, "{} {}", p.slug, e.prompt);
                            let mut seen = e.choices.clone();
                            seen.sort();
                            seen.dedup();
                            assert_eq!(
                                seen.len(),
                                4,
                                "{} repeats a choice: {:?}",
                                p.slug,
                                e.choices
                            );
                            e.choices[*i].clone()
                        }
                        Answer::Text(forms) => forms[0].clone(),
                    };
                    assert!(
                        e.check(&right),
                        "{}: {} does not accept {right}",
                        p.slug,
                        e.prompt
                    );
                    assert!(!e.check("definitely wrong 98765"), "{}", p.slug);
                }
            }
        }
    }

    #[test]
    fn answers_forgive_form() {
        let e = Exercise {
            plast: "x",
            seed: 0,
            level: 1,
            prompt: String::new(),
            choices: vec!["Leo Tolstoy".into(), "b".into(), "c".into(), "d".into()],
            answer: Answer::Choice(0),
            explain: String::new(),
        };
        assert!(e.check("a") && e.check(" A) ") && e.check("leo  TOLSTOY"));
        let n = Exercise {
            answer: Answer::Number(1225),
            choices: vec![],
            ..e
        };
        assert!(n.check("1,225") && n.check(" 1225 "));
        assert_eq!(normalize("  Camões "), "camoes");
    }
}
