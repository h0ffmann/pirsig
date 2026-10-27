//! Mental arithmetic: sums (1), differences and short products (2), two-digit
//! products (3), percentages (4), squares near 50 and 100 (5).
use super::{rng::Rng, Answer, Exercise};

#[must_use]
pub fn generate(seed: u64, level: u8) -> Exercise {
    let mut r = Rng::new(seed);
    let (prompt, value, explain) = match level {
        1 => {
            let (a, b) = (r.range(10, 99), r.range(10, 99));
            (
                format!("{a} + {b} = ?"),
                a + b,
                "add tens, then units".to_string(),
            )
        }
        2 if r.one_in(2) => {
            let (a, b) = (r.range(120, 999), r.range(11, 99));
            (
                format!("{a} − {b} = ?"),
                a - b,
                "subtract tens, then units".into(),
            )
        }
        2 => {
            let (a, b) = (r.range(3, 9), r.range(12, 99));
            (
                format!("{a} × {b} = ?"),
                a * b,
                format!("{a}×{}0 + {a}×{}", b / 10, b % 10),
            )
        }
        3 => {
            let (a, b) = (r.range(11, 99), r.range(11, 99));
            (
                format!("{a} × {b} = ?"),
                a * b,
                format!("{a}×{} + {a}×{}", b - b % 10, b % 10),
            )
        }
        4 => {
            let p = *r.pick(&[5, 10, 15, 20, 25, 30, 40, 50, 75]);
            let n = 20 * r.range(1, 20);
            (
                format!("{p}% of {n} = ?"),
                p * n / 100,
                format!("{n} × {p} ÷ 100"),
            )
        }
        _ => {
            let n = if r.one_in(2) {
                r.range(41, 59)
            } else {
                r.range(91, 109)
            };
            let base = if n < 75 { 50 } else { 100 };
            let d = n - base;
            (
                format!("{n}² = ?"),
                n * n,
                format!(
                    "({base} {} {})² = {}² {} 2×{base}×{} + {}²",
                    sign(d),
                    d.abs(),
                    base,
                    sign(d),
                    d.abs(),
                    d.abs()
                ),
            )
        }
    };
    Exercise {
        plast: "mental-arithmetic",
        seed,
        level,
        prompt,
        choices: vec![],
        answer: Answer::Number(value),
        explain,
    }
}

const fn sign(d: i64) -> char {
    if d < 0 {
        '−'
    } else {
        '+'
    }
}
