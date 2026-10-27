//! Sequences: find the rule, give the next term. Arithmetic (1), geometric (2),
//! quadratic (3), Fibonacci-like (4), two interleaved sequences (5).
use super::{rng::Rng, Answer, Exercise};

#[must_use]
pub fn generate(seed: u64, level: u8) -> Exercise {
    let mut r = Rng::new(seed);
    let (terms, explain): (Vec<i64>, String) = match level {
        1 => {
            let (a, d) = (r.range(1, 20), r.range(2, 9));
            (
                (0..6).map(|k| a + k * d).collect(),
                format!("add {d} each time"),
            )
        }
        2 => {
            let (a, q) = (r.range(1, 5), r.range(2, 3));
            (
                (0..6)
                    .map(|k| a * q.pow(u32::try_from(k).unwrap_or(0)))
                    .collect(),
                format!("multiply by {q} each time"),
            )
        }
        3 => {
            let (a, b, c) = (r.range(1, 3), r.range(0, 5), r.range(0, 9));
            (
                (1..=6).map(|k| a * k * k + b * k + c).collect(),
                "the differences grow by the same amount each time".into(),
            )
        }
        4 => {
            let mut t = vec![r.range(1, 9), r.range(1, 9)];
            for i in 2..6 {
                t.push(t[i - 1] + t[i - 2]);
            }
            (t, "each term is the sum of the two before it".into())
        }
        _ => {
            let (a, da, b, db) = (
                r.range(1, 9),
                r.range(2, 5),
                r.range(20, 40),
                -r.range(1, 4),
            );
            (
                (0..7)
                    .map(|k| {
                        if k % 2 == 0 {
                            a + k / 2 * da
                        } else {
                            b + k / 2 * db
                        }
                    })
                    .collect(),
                format!("two sequences take turns: +{da} and {db}"),
            )
        }
    };
    let (shown, next) = terms.split_at(terms.len() - 1);
    let shown: Vec<String> = shown.iter().map(i64::to_string).collect();
    Exercise {
        plast: "sequences",
        seed,
        level,
        prompt: format!("{}, ?", shown.join(", ")),
        choices: vec![],
        answer: Answer::Number(next[0]),
        explain,
    }
}
