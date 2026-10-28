//! A pen for Desmos line art.
//!
//! A stroke is a list of points. The pen runs a Catmull-Rom spline through them, so the line is
//! smooth and passes through every point. Each piece of the spline is a cubic Bézier, and each
//! Bézier is one Desmos parametric expression, `(a+bt+ct^2+dt^3, …)` for `0 ≤ t ≤ 1`.
//!
//! The SVG has y down and Desmos has y up: a Desmos y is `H - y` of the SVG.

use std::fmt::Write as _;

use serde_json::{json, Value};

/// A point in SVG coordinates (y down).
pub type P = (f64, f64);

/// The blue of the drawing, the same as in piabrota/alerts.
pub const BLUE: &str = "#1682D4";
/// The orange of the Desmos layer.
pub const ORANGE: &str = "#FF4F00";

#[derive(Debug, Clone, Copy)]
pub struct Bezier(pub [P; 4]);

impl Bezier {
    /// The power form `a + bt + ct² + dt³` of each axis: `[a, b, c, d]`.
    #[must_use]
    pub fn coeffs(&self) -> [P; 4] {
        let [p0, p1, p2, p3] = self.0;
        let f = |a: f64, b: f64, c: f64, d: f64| {
            [
                a,
                3.0 * (b - a),
                3.0 * (2.0f64.mul_add(-b, a) + c),
                3.0f64.mul_add(b - c, d - a),
            ]
        };
        let x = f(p0.0, p1.0, p2.0, p3.0);
        let y = f(p0.1, p1.1, p2.1, p3.1);
        [(x[0], y[0]), (x[1], y[1]), (x[2], y[2]), (x[3], y[3])]
    }
}

/// One mark on the paper.
#[derive(Debug, Clone)]
pub enum Mark {
    /// A smooth line, drawn as Béziers with this stroke width, solid or dashed.
    Line {
        beziers: Vec<Bezier>,
        width: f64,
        dashed: bool,
    },
    /// A filled dot. `size` is the Desmos `pointSize`.
    Dot { at: P, size: f64 },
}

/// A smooth stroke through `pts`. A closed stroke returns to its first point.
#[must_use]
pub fn stroke(pts: &[P], closed: bool, width: f64) -> Mark {
    let n = pts.len();
    // Point `j - n`: wrapped round a closed stroke, held at the ends of an open one.
    let get = |j: usize| -> P {
        if closed {
            pts[j % n]
        } else {
            pts[j.saturating_sub(n).min(n - 1)]
        }
    };
    let segments = if closed { n } else { n - 1 };
    let beziers = (n..n + segments)
        .map(|j| {
            let (p0, p1, p2, p3) = (get(j - 1), get(j), get(j + 1), get(j + 2));
            Bezier([
                p1,
                (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0),
                (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0),
                p2,
            ])
        })
        .collect();
    Mark::Line {
        beziers,
        width,
        dashed: false,
    }
}

/// A gently bent, dashed link from `a` to `b`: `bend` moves the middle sideways, as a share of
/// the length.
#[must_use]
pub fn link(a: P, b: P, bend: f64, width: f64) -> Mark {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let m = (
        dy.mul_add(-bend, a.0 + dx / 2.0),
        dx.mul_add(bend, a.1 + dy / 2.0),
    );
    match stroke(&[a, m, b], false, width) {
        Mark::Line { beziers, width, .. } => Mark::Line {
            beziers,
            width,
            dashed: true,
        },
        dot @ Mark::Dot { .. } => dot,
    }
}

#[must_use]
pub const fn dot(at: P, size: f64) -> Mark {
    Mark::Dot { at, size }
}

/// A finished drawing.
#[derive(Debug)]
pub struct Drawing {
    pub w: f64,
    pub h: f64,
    pub label: &'static str,
    pub desc: &'static str,
    pub marks: Vec<Mark>,
}

/// A number with at most three decimals, without trailing zeros: `12`, `-3.5`, `0.125`.
fn num(v: f64) -> String {
    let s = format!("{:.3}", (v * 1000.0).round() / 1000.0);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

/// `a+bt+ct^{2}+dt^{3}`, skipping the terms that round to zero.
fn poly(c: [f64; 4]) -> String {
    let mut out = num(c[0]);
    for (k, v) in c.iter().enumerate().skip(1) {
        let s = num(*v);
        if s == "0" {
            continue;
        }
        let pow = match k {
            1 => "t".to_string(),
            k => format!("t^{{{k}}}"),
        };
        let sign = if s.starts_with('-') { "" } else { "+" };
        let _ = write!(out, "{sign}{s}{pow}");
    }
    out
}

impl Drawing {
    /// The drawing as a Desmos graph state (`version` 11).
    #[must_use]
    pub fn desmos(&self) -> Value {
        let mut list = vec![];
        let mut push = |mut e: Value| {
            if let Some(m) = e.as_object_mut() {
                m.insert("type".into(), json!("expression"));
                m.insert("id".into(), json!((list.len() + 1).to_string()));
                m.insert("color".into(), json!(BLUE));
            }
            list.push(e);
        };
        for mark in &self.marks {
            match mark {
                Mark::Line {
                    beziers,
                    width,
                    dashed,
                } => {
                    for b in beziers {
                        let c = b.coeffs();
                        let x = poly([c[0].0, c[1].0, c[2].0, c[3].0]);
                        let y = poly([self.h - c[0].1, -c[1].1, -c[2].1, -c[3].1]);
                        let mut e = json!({
                            "latex": format!("\\left({x},{y}\\right)"),
                            "parametricDomain": {"min": "0", "max": "1"},
                            "lineWidth": num(*width),
                        });
                        if *dashed {
                            e["lineStyle"] = json!("DASHED");
                        }
                        push(e);
                    }
                }
                Mark::Dot { at, size } => push(json!({
                    "latex": format!("\\left({},{}\\right)", num(at.0), num(self.h - at.1)),
                    "pointSize": num(*size),
                })),
            }
        }
        json!({
            "version": 11,
            "graph": {
                "viewport": {"xmin": 0, "xmax": self.w, "ymin": 0, "ymax": self.h},
                "showGrid": false, "showXAxis": false, "showYAxis": false,
            },
            "expressions": {"list": list},
        })
    }

    /// The drawing as SVG: the geometry in blue and, on top, each Desmos expression sampled in
    /// orange. Where the two agree, the orange line runs inside the blue one.
    ///
    /// # Errors
    /// When an expression of [`Self::desmos`] cannot be read back.
    pub fn svg(&self) -> Result<String, String> {
        let mut blue = vec![];
        for mark in &self.marks {
            match mark {
                Mark::Line {
                    beziers,
                    width,
                    dashed,
                } => {
                    let mut d = String::new();
                    for (i, Bezier([p0, p1, p2, p3])) in beziers.iter().enumerate() {
                        if i == 0 {
                            let _ = write!(d, "M{} {}", num(p0.0), num(p0.1));
                        }
                        let _ = write!(
                            d,
                            " C{} {} {} {} {} {}",
                            num(p1.0),
                            num(p1.1),
                            num(p2.0),
                            num(p2.1),
                            num(p3.0),
                            num(p3.1)
                        );
                    }
                    let dash = if *dashed {
                        " stroke-dasharray=\"3 3\""
                    } else {
                        ""
                    };
                    blue.push(format!(
                        "<path d=\"{d}\" stroke-width=\"{}\"{dash}/>",
                        num(*width)
                    ));
                }
                Mark::Dot { at, size } => blue.push(format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{BLUE}\" stroke=\"none\"/>",
                    num(at.0),
                    num(at.1),
                    num(size / 3.0)
                )),
            }
        }
        let orange = self.desmos_layer()?;
        let (w, h, label, desc) = (num(self.w), num(self.h), self.label, self.desc);
        Ok(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="{label}">
  <title>{label}</title>
  <desc>{desc}</desc>
  <g fill="none" stroke="{BLUE}" stroke-linecap="round" stroke-linejoin="round">
    {}
  </g>
  <g fill="none" stroke="{ORANGE}" stroke-linecap="round" stroke-linejoin="round">
    {}
  </g>
</svg>"#,
            blue.join("\n    "),
            orange.join("\n    ")
        ))
    }

    /// Reads each expression of the Desmos state back and samples it.
    fn desmos_layer(&self) -> Result<Vec<String>, String> {
        let state = self.desmos();
        let list = state["expressions"]["list"]
            .as_array()
            .ok_or("no expressions")?;
        let mut out = vec![];
        for e in list {
            let latex = e["latex"].as_str().ok_or("no latex")?;
            let (fx, fy) = parse_pair(latex).ok_or_else(|| format!("cannot read {latex}"))?;
            if e.get("parametricDomain").is_some() {
                let pts: Vec<String> = (0..=24)
                    .map(|i| {
                        let t = f64::from(i) / 24.0;
                        format!("{:.1},{:.1}", eval(&fx, t), self.h - eval(&fy, t))
                    })
                    .collect();
                // A thin orange line inside the blue one: never more than 0.4 of its width.
                let w: f64 = e["lineWidth"]
                    .as_str()
                    .and_then(|w| w.parse().ok())
                    .unwrap_or(2.0);
                out.push(format!(
                    "<polyline points=\"{}\" stroke-width=\"{}\"/>",
                    pts.join(" "),
                    num((0.4 * w).min(0.8))
                ));
            } else {
                out.push(format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"0.8\" fill=\"{ORANGE}\" stroke=\"none\"/>",
                    num(eval(&fx, 0.0)),
                    num(self.h - eval(&fy, 0.0))
                ));
            }
        }
        Ok(out)
    }
}

/// The two coordinates of `\left(x,y\right)`, each as polynomial coefficients by power of t.
fn parse_pair(latex: &str) -> Option<(Vec<f64>, Vec<f64>)> {
    let inner = latex.strip_prefix("\\left(")?.strip_suffix("\\right)")?;
    let (x, y) = inner.split_once(',')?;
    Some((parse_poly(x)?, parse_poly(y)?))
}

/// `12.5-3t+0.25t^{2}` → `[12.5, -3, 0.25]`.
fn parse_poly(s: &str) -> Option<Vec<f64>> {
    let mut coeffs = vec![0.0; 4];
    let mut terms = vec![];
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        if i > 0 && (ch == '+' || ch == '-') {
            terms.push(&s[start..i]);
            start = i;
        }
    }
    terms.push(&s[start..]);
    for term in terms {
        let term = term.strip_prefix('+').unwrap_or(term);
        let (c, k) = if let Some((c, pow)) = term.split_once("t^{") {
            (c, pow.strip_suffix('}')?.parse::<usize>().ok()?)
        } else if let Some(c) = term.strip_suffix('t') {
            (c, 1)
        } else {
            (term, 0)
        };
        *coeffs.get_mut(k)? += c.parse::<f64>().ok()?;
    }
    Some(coeffs)
}

fn eval(c: &[f64], t: f64) -> f64 {
    c.iter().rev().fold(0.0, |acc, v| acc.mul_add(t, *v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_short() {
        assert_eq!(num(12.0), "12");
        assert_eq!(num(-3.5), "-3.5");
        assert_eq!(num(0.1234), "0.123");
        assert_eq!(num(-0.0001), "0");
    }

    #[test]
    fn a_polynomial_reads_back() {
        let s = poly([12.5, -3.0, 0.0, 0.25]);
        assert_eq!(s, "12.5-3t+0.25t^{3}");
        assert_eq!(parse_poly(&s), Some(vec![12.5, -3.0, 0.0, 0.25]));
    }

    #[test]
    fn the_power_form_is_the_bezier() {
        let b = Bezier([(0.0, 0.0), (10.0, 30.0), (40.0, -10.0), (50.0, 20.0)]);
        let c = b.coeffs();
        let x: Vec<f64> = c.iter().map(|p| p.0).collect();
        // Midpoint of a cubic Bézier: (p0 + 3p1 + 3p2 + p3) / 8.
        assert!((eval(&x, 0.5) - 25.0).abs() < 1e-9);
        assert!((eval(&x, 1.0) - 50.0).abs() < 1e-9);
    }
}
