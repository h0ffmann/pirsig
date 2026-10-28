//! The Pirsig logo: 無 (*mu*, "no thing") brushed in its twelve strokes, in the standard order,
//! inside an open ensō. Pirsig answers yes-or-no questions with *mu* in *Zen and the Art of
//! Motorcycle Maintenance* (chapter 26).
//!
//! Each stroke is a brush line through hand-placed points with a width at each point, so it
//! presses in, carries, and lifts the way a brush does: horizontals and verticals stop firmly
//! (*tome*), the first stroke sweeps out to a point (*harai*), and the fire dots (灬) swell as
//! they fall. The stroke order follows KanjiVG; the points are drawn here.

use std::f64::consts::PI;

use crate::pen::{brush, Drawing, Mark, P};

/// A stroke in a 0–100 box: (x, y, brush width at that point).
type Stroke = &'static [(f64, f64, f64)];

/// The twelve strokes of 無, in writing order.
const MU: [Stroke; 12] = [
    // 1 ㇒ the falling stroke at the top left: pressed in, swept out to a point
    &[
        (41.0, 12.0, 7.0),
        (39.5, 17.0, 7.0),
        (34.0, 24.5, 5.2),
        (26.5, 31.5, 3.0),
        (19.5, 36.0, 1.0),
    ],
    // 2 ㇐ the short top horizontal, rising a little, stopped at the end
    &[
        (36.5, 27.0, 6.0),
        (40.0, 25.6, 4.8),
        (56.0, 23.6, 3.0),
        (71.0, 21.8, 3.2),
        (77.5, 21.4, 5.4),
        (79.0, 22.8, 6.0),
    ],
    // 3 ㇐ the long horizontal that the verticals hang from
    &[
        (11.5, 51.0, 7.0),
        (16.0, 49.8, 5.4),
        (40.0, 48.2, 3.4),
        (70.0, 46.4, 3.3),
        (92.0, 45.4, 5.4),
        (96.5, 46.8, 7.0),
    ],
    // 4–7 ㇑ the four verticals, left to right, through the long horizontal
    &[
        (31.5, 37.5, 7.0),
        (32.6, 41.0, 5.8),
        (33.6, 54.0, 5.0),
        (34.4, 66.0, 6.0),
    ],
    &[
        (46.2, 33.0, 7.0),
        (47.3, 37.0, 5.8),
        (47.6, 52.0, 5.0),
        (47.9, 65.5, 6.0),
    ],
    &[
        (62.0, 31.5, 7.0),
        (62.7, 35.5, 5.8),
        (62.4, 50.0, 5.0),
        (61.8, 64.8, 6.0),
    ],
    &[
        (77.8, 32.0, 7.0),
        (78.4, 36.5, 5.8),
        (77.0, 50.0, 5.0),
        (75.2, 64.0, 5.6),
    ],
    // 8 ㇐ the bottom horizontal, closing the frame
    &[
        (18.0, 69.0, 7.0),
        (22.5, 68.2, 5.4),
        (50.0, 66.4, 3.4),
        (80.0, 65.0, 3.3),
        (89.0, 65.0, 5.4),
        (91.5, 66.4, 7.0),
    ],
    // 9–12 ㇔ fire 灬: the first dot falls to the left, the other three to the right
    &[
        (24.5, 81.5, 3.0),
        (23.0, 86.5, 5.4),
        (20.0, 92.0, 6.2),
        (16.0, 96.5, 2.8),
    ],
    &[
        (38.5, 79.5, 2.4),
        (41.8, 83.2, 5.0),
        (44.6, 88.8, 7.0),
        (45.6, 93.0, 6.0),
    ],
    &[
        (57.8, 78.5, 2.4),
        (61.6, 82.4, 5.0),
        (65.0, 88.4, 7.0),
        (66.2, 92.2, 6.0),
    ],
    &[
        (80.5, 78.6, 2.4),
        (84.8, 82.8, 5.0),
        (88.8, 89.4, 7.4),
        (90.6, 93.8, 6.4),
    ],
];

/// The kanji's box on the canvas: centre and scale of the 0–100 box.
const CENTRE: P = (160.0, 122.0);
const SCALE: f64 = 1.5;
/// How heavy the brush is: the widths of [`MU`] times this.
const INK: f64 = 1.5;

fn place((x, y): P) -> P {
    (
        (x - 55.0).mul_add(SCALE, CENTRE.0),
        (y - 55.0).mul_add(SCALE, CENTRE.1),
    )
}

/// The ensō: one stroke round the kanji, begun pressed at the lower left, carried up and over,
/// and lifted dry before it closes, leaving the circle open.
fn enso() -> (Vec<P>, Vec<f64>) {
    let (start, end, n) = (214.0_f64, -116.0_f64, 30);
    let mut pts = vec![];
    let mut widths = vec![];
    for i in 0..=n {
        let k = f64::from(i) / f64::from(n);
        let a = (end - start).mul_add(k, start) * PI / 180.0;
        // the hand drifts a little inward as the stroke goes round
        let r = 4.0f64.mul_add(-k, 100.0);
        pts.push((r.mul_add(a.cos(), CENTRE.0), r.mul_add(-a.sin(), CENTRE.1)));
        // pressed hard at the start, thinning as the ink runs out
        widths.push(5.5f64.mul_add(-k.sqrt(), 6.5).max(1.0));
    }
    (pts, widths)
}

#[must_use]
pub fn drawing() -> Drawing {
    let mut marks: Vec<Mark> = vec![];
    let (ring, ring_widths) = enso();
    marks.extend(brush(&ring, &ring_widths));
    for s in MU {
        let pts: Vec<P> = s.iter().map(|&(x, y, _)| place((x, y))).collect();
        let widths: Vec<f64> = s.iter().map(|&(_, _, w)| w * INK).collect();
        marks.extend(brush(&pts, &widths));
    }
    Drawing {
        w: 320.0,
        h: 240.0,
        label: "The Pirsig logo: the kanji 無 (mu, no thing) brushed in its twelve strokes inside an open ensō, by @h0ffmann, rendered from its Desmos expressions",
        desc: "Blue: the geometry, cubic Béziers through hand-placed points, each piece at its brush width. Orange: each expression of pirsig.desmos.json, sampled.",
        marks,
    }
}

#[cfg(test)]
mod tests {
    use super::MU;

    #[test]
    fn mu_has_its_twelve_strokes_in_order() {
        assert_eq!(MU.len(), 12);
        // the four verticals go left to right, and hang across the long horizontal
        let long_y = MU[2][2].1;
        for pair in MU[3..7].windows(2) {
            assert!(pair[0][0].0 < pair[1][0].0);
        }
        for v in &MU[3..7] {
            assert!(v[0].1 < long_y && v[v.len() - 1].1 > long_y);
        }
        // the first fire dot falls left, the others right
        let dx = |s: &[(f64, f64, f64)]| s[s.len() - 1].0 - s[0].0;
        assert!(dx(MU[8]) < 0.0);
        assert!(MU[9..].iter().all(|s| dx(s) > 0.0));
    }
}
