//! Argument forms: valid or not? Conditionals (1–2), disjunctions (3),
//! chains (4), contrapositive and converse (5). The feedback names the form,
//! so the fallacies become recognisable.
use super::{rng::Rng, Answer, Exercise};

/// Everyday propositions: (p, q, r) that read well in any form.
const PROPS: [(&str, &str, &str); 8] = [
    ("it rains", "the street is wet", "the bus is late"),
    ("the server is down", "the alarm rings", "someone is paged"),
    ("the tide is high", "the boats float", "the harbour opens"),
    ("you practise", "you improve", "you enjoy it"),
    ("the cat is hungry", "it meows", "nobody sleeps"),
    ("the note is atomic", "it is easy to link", "it gets reused"),
    (
        "the wind is strong",
        "the waves are high",
        "the buoy drifts",
    ),
    ("the build passes", "the release ships", "users upgrade"),
];

/// A premise or conclusion in a form: `If(a, b)`, `Or(a, b)`, a fact or its negation.
#[derive(Clone, Copy)]
pub enum Part {
    If(usize, usize),
    Or(usize, usize),
    Is(usize),
    Not(usize),
    IfNot(usize, usize),
}

pub struct Form {
    pub name: &'static str,
    pub valid: bool,
    pub premises: &'static [Part],
    pub conclusion: Part,
    pub min_level: u8,
}

use Part::{If, IfNot, Is, Not, Or};

pub const FORMS: [Form; 10] = [
    Form {
        name: "modus ponens",
        valid: true,
        premises: &[If(0, 1), Is(0)],
        conclusion: Is(1),
        min_level: 1,
    },
    Form {
        name: "affirming the consequent",
        valid: false,
        premises: &[If(0, 1), Is(1)],
        conclusion: Is(0),
        min_level: 1,
    },
    Form {
        name: "modus tollens",
        valid: true,
        premises: &[If(0, 1), Not(1)],
        conclusion: Not(0),
        min_level: 2,
    },
    Form {
        name: "denying the antecedent",
        valid: false,
        premises: &[If(0, 1), Not(0)],
        conclusion: Not(1),
        min_level: 2,
    },
    Form {
        name: "disjunctive syllogism",
        valid: true,
        premises: &[Or(0, 1), Not(0)],
        conclusion: Is(1),
        min_level: 3,
    },
    Form {
        name: "affirming a disjunct (\"or\" is inclusive)",
        valid: false,
        premises: &[Or(0, 1), Is(0)],
        conclusion: Not(1),
        min_level: 3,
    },
    Form {
        name: "hypothetical syllogism",
        valid: true,
        premises: &[If(0, 1), If(1, 2)],
        conclusion: If(0, 2),
        min_level: 4,
    },
    Form {
        name: "a chain that does not connect",
        valid: false,
        premises: &[If(0, 1), If(2, 1)],
        conclusion: If(0, 2),
        min_level: 4,
    },
    Form {
        name: "contraposition",
        valid: true,
        premises: &[If(0, 1)],
        conclusion: IfNot(1, 0),
        min_level: 5,
    },
    Form {
        name: "the converse",
        valid: false,
        premises: &[If(0, 1)],
        conclusion: If(1, 0),
        min_level: 5,
    },
];

fn say(part: Part, p: [&str; 3]) -> String {
    match part {
        If(a, b) => format!("If {}, then {}.", p[a], p[b]),
        Or(a, b) => format!("Either {} or {}.", p[a], p[b]),
        Is(a) => format!("{}.", capital(p[a])),
        Not(a) => format!("It is not true that {}.", p[a]),
        IfNot(a, b) => format!(
            "If it is not true that {}, then it is not true that {}.",
            p[a], p[b]
        ),
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

#[must_use]
pub fn generate(seed: u64, level: u8) -> Exercise {
    let mut rng = Rng::new(seed);
    let open: Vec<&Form> = FORMS.iter().filter(|f| f.min_level <= level).collect();
    let form = *rng.pick(&open);
    let props: [&str; 3] = (*rng.pick(&PROPS)).into();
    let mut lines: Vec<String> = form.premises.iter().map(|&x| say(x, props)).collect();
    lines.push(format!("Therefore: {}", say(form.conclusion, props)));
    let choices = vec!["valid".to_string(), "invalid".to_string()];
    Exercise {
        plast: "argument-forms",
        seed,
        level,
        prompt: format!("{}\nValid or invalid?", lines.join("\n")),
        choices: choices.clone(),
        answer: Answer::Text(vec![
            choices[usize::from(!form.valid)].clone(),
            if form.valid { "v" } else { "i" }.to_string(),
        ]),
        explain: format!("{}: {}", choices[usize::from(!form.valid)], form.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holds(part: Part, v: [bool; 3]) -> bool {
        match part {
            If(a, b) => !v[a] || v[b],
            Or(a, b) => v[a] || v[b],
            Is(a) => v[a],
            Not(a) => !v[a],
            IfNot(a, b) => v[a] || !v[b],
        }
    }

    /// Every form's `valid` agrees with its truth table.
    #[test]
    fn forms_are_labelled_by_their_truth_tables() {
        for f in &FORMS {
            let valid = (0..8).all(|bits| {
                let v = [bits & 1 != 0, bits & 2 != 0, bits & 4 != 0];
                !f.premises.iter().all(|&x| holds(x, v)) || holds(f.conclusion, v)
            });
            assert_eq!(valid, f.valid, "{}", f.name);
        }
    }
}
