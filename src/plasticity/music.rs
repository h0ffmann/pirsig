//! Intervals.
//!
//! Semitones between natural notes (1), interval names (2), the note an
//! interval above (3), from any key (4), and below (5). Notes are compared by
//! pitch class, so any spelling of the right note counts.
use super::{choices, rng::Rng, Answer, Exercise};

const SHARPS: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
const FLATS: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];
const NATURALS: [usize; 7] = [0, 2, 4, 5, 7, 9, 11];
pub const INTERVALS: [&str; 12] = [
    "unison",
    "minor 2nd",
    "major 2nd",
    "minor 3rd",
    "major 3rd",
    "perfect 4th",
    "tritone",
    "perfect 5th",
    "minor 6th",
    "major 6th",
    "minor 7th",
    "major 7th",
];

/// Every way to write pitch class `pc`.
fn spellings(pc: usize) -> Vec<String> {
    let mut out = vec![SHARPS[pc].to_string(), FLATS[pc].to_string()];
    out.extend([SHARPS[pc].replace('#', "♯"), FLATS[pc].replace('b', "♭")]);
    // the white-key enharmonics: E#=F, B#=C, Fb=E, Cb=B
    match pc {
        5 => out.push("E#".into()),
        0 => out.push("B#".into()),
        4 => out.push("Fb".into()),
        11 => out.push("Cb".into()),
        _ => {}
    }
    out.dedup();
    out
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn generate(seed: u64, level: u8) -> Exercise {
    let mut r = Rng::new(seed);
    let names = if level >= 4 && r.one_in(2) {
        &FLATS
    } else {
        &SHARPS
    };
    let (prompt, answer, choice_list, explain) = match level {
        1 => {
            let from = *r.pick(&NATURALS);
            let mut to = *r.pick(&NATURALS);
            if to == from {
                to = (from + 7) % 12;
            }
            let st = (to + 12 - from) % 12;
            (
                format!(
                    "How many semitones from {} up to {}?",
                    SHARPS[from], SHARPS[to]
                ),
                Answer::Number(i64::try_from(st).unwrap_or_default()),
                vec![],
                format!("{} to {} is a {}", SHARPS[from], SHARPS[to], INTERVALS[st]),
            )
        }
        2 => {
            let from = *r.pick(&NATURALS);
            let st = 1 + r.below(11) as usize;
            let to = (from + st) % 12;
            let wrong: Vec<&str> = INTERVALS[1..].to_vec();
            let (c, at) = choices(&mut r, INTERVALS[st], &wrong);
            (
                format!(
                    "Name the interval from {} up to {}.",
                    SHARPS[from], SHARPS[to]
                ),
                Answer::Choice(at),
                c,
                format!("{st} semitones: {}", INTERVALS[st]),
            )
        }
        _ => {
            let from = if level == 3 {
                *r.pick(&NATURALS)
            } else {
                r.below(12) as usize
            };
            let st = 1 + r.below(11) as usize;
            let down = level >= 5 && r.one_in(2);
            let to = if down {
                (from + 12 - st) % 12
            } else {
                (from + st) % 12
            };
            let dir = if down { "below" } else { "above" };
            (
                format!("Which note is a {} {dir} {}?", INTERVALS[st], names[from]),
                Answer::Text(spellings(to)),
                vec![],
                format!(
                    "{st} semitones {dir} {}: {} (any spelling)",
                    names[from], names[to]
                ),
            )
        }
    };
    Exercise {
        plast: "intervals",
        seed,
        level,
        prompt,
        choices: choice_list,
        answer,
        explain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fifth_above_d_is_a() {
        assert!(spellings(9).contains(&"A".to_string()));
        assert!(
            spellings(10).contains(&"Bb".to_string()) && spellings(10).contains(&"A#".to_string())
        );
        assert_eq!(INTERVALS[7], "perfect 5th");
    }
}
