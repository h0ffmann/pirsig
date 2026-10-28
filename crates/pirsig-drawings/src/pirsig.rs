//! The Pirsig drawing: a head in one line, after Picasso's one-line drawings, whose brain is a
//! zettelkasten: one folded line of thought, notes (dots) on it, and links between them.

use std::f64::consts::PI;

use crate::pen::{dot, link, stroke, Drawing, Mark, P};

/// The head, facing right, from the nape round the skull and down the face to the throat.
const HEAD: &[P] = &[
    (96.0, 236.0),
    (100.0, 206.0),
    (96.0, 184.0),
    (80.0, 160.0),
    (70.0, 126.0),
    (72.0, 88.0),
    (92.0, 50.0),
    (128.0, 24.0),
    (172.0, 16.0),
    (214.0, 28.0),
    (240.0, 56.0),
    (250.0, 86.0),
    (251.0, 104.0),
    (248.0, 112.0),
    (260.0, 132.0),
    (268.0, 146.0),
    (254.0, 150.0),
    (256.0, 160.0),
    (251.0, 166.0),
    (256.0, 174.0),
    (249.0, 184.0),
    (252.0, 198.0),
    (236.0, 210.0),
    (212.0, 212.0),
    (204.0, 236.0),
];

/// The brain's outline: a closed line around an ellipse, swelling into lobes.
fn brain_outline() -> Vec<P> {
    let (cx, cy, rx, ry) = (156.0, 80.0, 58.0, 42.0);
    (0..28)
        .map(|i| {
            let a = 2.0 * PI * f64::from(i) / 28.0;
            let r = 0.04f64.mul_add(
                (13.0 * a).sin(),
                0.07f64.mul_add(7.0f64.mul_add(a, 0.4).sin(), 1.0),
            );
            ((r * rx).mul_add(a.cos(), cx), (r * ry).mul_add(a.sin(), cy))
        })
        .collect()
}

/// The folds (gyri and sulci), each one line, drawn by hand.
const FOLDS: &[&[P]] = &[
    &[
        (171.0, 39.9),
        (164.4, 55.2),
        (172.8, 68.5),
        (162.5, 83.8),
        (169.1, 97.2),
    ],
    &[
        (113.0, 104.8),
        (135.4, 96.2),
        (157.9, 101.0),
        (178.5, 92.4),
        (195.3, 98.1),
    ],
    &[
        (195.3, 47.5),
        (187.8, 60.9),
        (200.0, 70.5),
        (194.4, 83.8),
        (204.6, 89.5),
    ],
    &[
        (139.2, 41.8),
        (145.7, 56.1),
        (133.5, 64.7),
        (143.8, 78.1),
        (129.8, 85.7),
    ],
    &[(105.5, 64.7), (118.6, 72.4), (110.2, 85.7)],
    &[(133.5, 113.4), (150.4, 109.6), (161.6, 116.3)],
    &[(178.5, 74.3), (186.9, 81.9), (179.4, 87.6)],
    &[(118.6, 49.5), (122.3, 59.0)],
];

/// A cubist eye: seen from the front, set in a face seen from the side.
const EYE: &[P] = &[
    (220.0, 110.0),
    (231.0, 104.0),
    (243.0, 110.0),
    (231.0, 116.0),
    (220.0, 110.0),
];

/// The ear, a spiral.
fn ear() -> Vec<P> {
    let c = (134.0, 146.0);
    (0..=18)
        .map(|i| {
            let t = f64::from(i) / 18.0;
            let a = 2.2f64.mul_add(PI * t, 0.6 * PI);
            let r = 14.0f64.mul_add(1.0 - t, 2.0);
            (
                r.mul_add(a.cos(), c.0) - 2.0,
                (r * 1.25).mul_add(a.sin(), c.1),
            )
        })
        .collect()
}

/// The signature, `psg`, written small at the bottom right.
const Z: &[P] = &[
    (262.0, 214.0),
    (274.0, 213.0),
    (262.0, 226.0),
    (276.0, 225.0),
];
const T: &[P] = &[
    (282.0, 207.0),
    (281.0, 220.0),
    (285.0, 226.0),
    (290.0, 222.0),
];
const T_BAR: &[P] = &[(277.0, 213.0), (288.0, 212.0)];
const K: &[P] = &[(295.0, 204.0), (294.0, 226.0)];
const K_ARM: &[P] = &[(305.0, 212.0), (296.0, 218.0), (306.0, 226.0)];

#[must_use]
pub fn drawing() -> Drawing {
    // Notes sit on the folds; links jump between folds, as links between notes do.
    let notes: Vec<P> = [
        (0, 2),
        (1, 2),
        (2, 2),
        (3, 2),
        (4, 1),
        (5, 1),
        (6, 1),
        (1, 3),
    ]
    .iter()
    .map(|&(f, i)| FOLDS[f][i])
    .collect();
    let links = [
        (3, 0, -0.15),
        (0, 2, 0.15),
        (4, 3, 0.12),
        (1, 5, -0.2),
        (6, 7, 0.2),
        (0, 6, -0.15),
        (3, 1, 0.12),
    ];

    let mut marks: Vec<Mark> = vec![
        stroke(HEAD, false, 3.2),
        stroke(&brain_outline(), true, 2.2),
        stroke(EYE, false, 2.0),
        dot((232.0, 110.0), 7.0),
        stroke(&ear(), false, 2.0),
    ];
    for fold in FOLDS {
        marks.push(stroke(fold, false, 1.8));
    }
    for (a, b, bend) in links {
        marks.push(link(notes[a], notes[b], bend, 1.0));
    }
    for n in notes {
        marks.push(dot(n, 9.0));
    }
    for s in [Z, T, T_BAR, K, K_ARM] {
        marks.push(stroke(s, false, 1.6));
    }
    Drawing {
        w: 320.0,
        h: 240.0,
        label: "The Pirsig logo: a head in one line whose brain is a zettelkasten, by @h0ffmann, rendered from its Desmos expressions",
        desc: "Blue: the geometry, cubic Béziers through hand-placed points. Orange: each expression of pirsig.desmos.json, sampled.",
        marks,
    }
}
