//! Latin words: Latin to English (1–2), English to Latin (3), Latin to English
//! typed (4–5).
use super::{choices, rng::Rng, Answer, Exercise};

/// (Latin, English meanings).
pub const WORDS: [(&str, &[&str]); 40] = [
    ("aqua", &["water"]),
    ("ignis", &["fire"]),
    ("terra", &["earth", "land"]),
    ("lux", &["light"]),
    ("mare", &["sea"]),
    ("ventus", &["wind"]),
    ("liber", &["book", "free"]),
    ("amicus", &["friend"]),
    ("bellum", &["war"]),
    ("pax", &["peace"]),
    ("tempus", &["time"]),
    ("vita", &["life"]),
    ("mors", &["death"]),
    ("caelum", &["sky", "heaven"]),
    ("nox", &["night"]),
    ("dies", &["day"]),
    ("urbs", &["city"]),
    ("rex", &["king"]),
    ("homo", &["human", "man", "person"]),
    ("verbum", &["word"]),
    ("manus", &["hand"]),
    ("pes", &["foot"]),
    ("oculus", &["eye"]),
    ("cor", &["heart"]),
    ("sol", &["sun"]),
    ("luna", &["moon"]),
    ("stella", &["star"]),
    ("arbor", &["tree"]),
    ("flumen", &["river"]),
    ("mons", &["mountain"]),
    ("lupus", &["wolf"]),
    ("equus", &["horse"]),
    ("canis", &["dog"]),
    ("puer", &["boy", "child"]),
    ("puella", &["girl"]),
    ("domus", &["house", "home"]),
    ("via", &["road", "way"]),
    ("mater", &["mother"]),
    ("pater", &["father"]),
    ("lingua", &["tongue", "language"]),
];

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn generate(seed: u64, level: u8) -> Exercise {
    let mut r = Rng::new(seed);
    let (latin, meanings) = WORDS[r.below(WORDS.len() as u64) as usize];
    let others = WORDS.iter().filter(|w| w.0 != latin);
    let (prompt, answer, list) = match level {
        1 | 2 => {
            let wrong: Vec<&str> = others.map(|w| w.1[0]).collect();
            let (c, at) = choices(&mut r, meanings[0], &wrong);
            (format!("Latin “{latin}” means…"), Answer::Choice(at), c)
        }
        3 => {
            let wrong: Vec<&str> = others.map(|w| w.0).collect();
            let (c, at) = choices(&mut r, latin, &wrong);
            (
                format!("“{}” in Latin is…", meanings[0]),
                Answer::Choice(at),
                c,
            )
        }
        _ => (
            format!("Latin “{latin}” means… (type it in English)"),
            Answer::Text(meanings.iter().map(|m| (*m).to_string()).collect()),
            vec![],
        ),
    };
    Exercise {
        plast: "latin-words",
        seed,
        level,
        prompt,
        choices: list,
        answer,
        explain: format!("{latin}: {}", meanings.join(", ")),
    }
}
