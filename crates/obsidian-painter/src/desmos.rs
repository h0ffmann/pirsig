//! Desmos graphs to SVG, offline: no API key, no network.
//!
//! The input is a Desmos graph state (what `Calc.getState()` returns, and what
//! `pirsig-art desmos` writes), or, for a hand-written block, one LaTeX expression
//! per line. Drawn: points `(a,b)`, parametric curves `(f(t),g(t))` over their
//! `parametricDomain`, functions `y=f(x)` (or a bare `f(x)`), `x=g(y)`, polar
//! `r=f(\theta)`, and assignments (`a=2`) that the others use. Implicit
//! equations and inequalities are skipped and reported.
use std::fmt::Write;

use serde::Serialize;
use serde_json::Value;

use crate::latex::{parse, Env, Expr, Line};

const WIDTH: f64 = 800.0;
const SAMPLES: usize = 400;
const PALETTE: [&str; 6] = [
    "#c74440", "#2d70b3", "#388c46", "#6042a6", "#fa7e19", "#000000",
];

#[derive(Debug, Serialize)]
pub struct Rendered {
    pub svg: String,
    /// Expressions that could not be drawn, and why.
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct View {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}

impl View {
    fn height(&self) -> f64 {
        WIDTH * (self.ymax - self.ymin) / (self.xmax - self.xmin)
    }

    /// Graph coordinates to SVG coordinates (Desmos has y up).
    fn px(&self, x: f64, y: f64) -> (f64, f64) {
        (
            (x - self.xmin) / (self.xmax - self.xmin) * WIDTH,
            (self.ymax - y) / (self.ymax - self.ymin) * self.height(),
        )
    }
}

struct Style {
    color: String,
    width: f64,
    dash: Option<&'static str>,
    point: f64,
}

enum Shape {
    Curve(Vec<Vec<(f64, f64)>>),
    Point(f64, f64),
}

fn num(v: Option<&Value>) -> Option<f64> {
    match v? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => match parse(s).ok()? {
            Line::Expr(e) => Some(e.eval(&Env::new())).filter(|x| x.is_finite()),
            Line::Equation(..) => None,
        },
        _ => None,
    }
}

fn style(e: &Value, i: usize) -> Style {
    let color = e
        .get("color")
        .and_then(Value::as_str)
        .filter(|c| {
            c.starts_with('#') && c.len() <= 9 && c[1..].chars().all(|ch| ch.is_ascii_hexdigit())
        })
        .map_or_else(|| PALETTE[i % PALETTE.len()].to_string(), str::to_string);
    let dash = match e.get("lineStyle").and_then(Value::as_str) {
        Some("DASHED") => Some("8 6"),
        Some("DOTTED") => Some("1 5"),
        _ => None,
    };
    Style {
        color,
        width: num(e.get("lineWidth")).unwrap_or(2.5),
        dash,
        point: num(e.get("pointSize")).unwrap_or(9.0),
    }
}

/// Sample `f` over `lo..=hi`, breaking the line where it leaves the plane or jumps.
fn sample(lo: f64, hi: f64, jump: f64, f: impl Fn(f64) -> (f64, f64)) -> Vec<Vec<(f64, f64)>> {
    let mut runs = vec![Vec::new()];
    let mut last: Option<(f64, f64)> = None;
    for k in 0..=SAMPLES {
        #[allow(clippy::cast_precision_loss)]
        let u = lo + (hi - lo) * k as f64 / SAMPLES as f64;
        let (x, y) = f(u);
        let ok = x.is_finite() && y.is_finite();
        let jumped = last.is_some_and(|(lx, ly)| (x - lx).abs() > jump || (y - ly).abs() > jump);
        if (!ok || jumped) && runs.last().is_some_and(|r| !r.is_empty()) {
            runs.push(Vec::new());
        }
        if let (true, Some(r)) = (ok, runs.last_mut()) {
            r.push((x, y));
        }
        last = ok.then_some((x, y));
    }
    runs.retain(|r| r.len() > 1);
    runs
}

fn with(env: &Env, name: &str, v: f64) -> Env {
    let mut e = env.clone();
    e.insert(name.to_string(), v);
    e
}

fn names(e: &Expr) -> Vec<String> {
    let mut out = Vec::new();
    e.names(&mut out);
    out
}

/// What one line draws, with `env` holding the values assigned so far.
fn shape(
    line: &Line,
    item: &Value,
    env: &Env,
    view: Option<View>,
) -> Result<Option<Shape>, String> {
    let (vx0, vx1, vy0, vy1) = view.map_or((-10.0, 10.0, -10.0, 10.0), |v| {
        (v.xmin, v.xmax, v.ymin, v.ymax)
    });
    let jump = (vy1 - vy0).max(vx1 - vx0);
    let free = |e: &Expr| -> Vec<String> {
        names(e)
            .into_iter()
            .filter(|n| !env.contains_key(n))
            .collect()
    };
    match line {
        Line::Expr(Expr::Pair(a, b)) => {
            let mut vars = free(a);
            vars.extend(free(b));
            if vars.is_empty() {
                return Ok(Some(Shape::Point(a.eval(env), b.eval(env))));
            }
            if vars.iter().any(|v| v != "t") {
                return Err(format!("unknown names {vars:?}"));
            }
            let dom = item.get("parametricDomain");
            let lo = num(dom.and_then(|d| d.get("min"))).unwrap_or(0.0);
            let hi = num(dom.and_then(|d| d.get("max"))).unwrap_or(1.0);
            Ok(Some(Shape::Curve(sample(lo, hi, jump, |t| {
                let e = with(env, "t", t);
                (a.eval(&e), b.eval(&e))
            }))))
        }
        Line::Equation(Expr::Var(v), rhs) if v == "y" || v == "x" || v == "r" => {
            let other = match v.as_str() {
                "y" => "x",
                "x" => "y",
                _ => "theta",
            };
            if free(rhs).iter().any(|n| n != other) {
                return Err("implicit equations are not drawn".into());
            }
            let curve = match v.as_str() {
                "y" => sample(vx0, vx1, jump, |x| (x, rhs.eval(&with(env, "x", x)))),
                "x" => sample(vy0, vy1, jump, |y| (rhs.eval(&with(env, "y", y)), y)),
                _ => {
                    let dom = item.get("polarDomain");
                    let lo = num(dom.and_then(|d| d.get("min"))).unwrap_or(0.0);
                    let hi = num(dom.and_then(|d| d.get("max"))).unwrap_or(std::f64::consts::TAU);
                    sample(lo, hi, jump, |th| {
                        let r = rhs.eval(&with(env, "theta", th));
                        (r * th.cos(), r * th.sin())
                    })
                }
            };
            Ok(Some(Shape::Curve(curve)))
        }
        Line::Expr(e) => {
            let vars = free(e);
            if vars.iter().all(|n| n == "x") && !vars.is_empty() {
                Ok(Some(Shape::Curve(sample(vx0, vx1, jump, |x| {
                    (x, e.eval(&with(env, "x", x)))
                }))))
            } else if vars.is_empty() {
                Ok(None) // a number on its own line draws nothing
            } else {
                Err(format!("unknown names {vars:?}"))
            }
        }
        Line::Equation(..) => Err("implicit equations are not drawn".into()),
    }
}

/// The drawable expressions: the state's `expressions.list`, or one LaTeX
/// line per expression.
fn items(input: &str) -> (Vec<Value>, Option<&Value>, Value) {
    match serde_json::from_str::<Value>(input) {
        Ok(state) if state.is_object() => {
            let list = state
                .pointer("/expressions/list")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            (list, None, state)
        }
        _ => {
            let list = input
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('%'))
                .map(|l| serde_json::json!({ "type": "expression", "latex": l }))
                .collect();
            (list, None, Value::Null)
        }
    }
}

/// Render a Desmos state (JSON) or LaTeX lines to an SVG.
#[must_use]
#[allow(clippy::too_many_lines)] // one pass: read, assign, draw, write
pub fn render(input: &str) -> Rendered {
    let (list, _, state) = items(input);
    let graph = state.get("graph");
    let viewport = graph.and_then(|g| g.get("viewport"));
    let view = viewport.and_then(|v| {
        Some(View {
            xmin: num(v.get("xmin"))?,
            xmax: num(v.get("xmax"))?,
            ymin: num(v.get("ymin"))?,
            ymax: num(v.get("ymax"))?,
        })
    });
    let flag = |k: &str| {
        graph
            .and_then(|g| g.get(k))
            .and_then(Value::as_bool)
            .unwrap_or(true)
    };

    let mut env = Env::new();
    let mut skipped = Vec::new();
    let mut drawn: Vec<(Shape, Style)> = Vec::new();
    for (i, item) in list.iter().enumerate() {
        if item
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|t| t != "expression")
            || item.get("hidden").and_then(Value::as_bool) == Some(true)
        {
            continue;
        }
        let Some(latex) = item.get("latex").and_then(Value::as_str) else {
            continue;
        };
        let line = match parse(latex) {
            Ok(l) => l,
            Err(e) => {
                skipped.push(format!("{latex}: {e}"));
                continue;
            }
        };
        // `a=2`: a value the other lines can use
        if let Line::Equation(Expr::Var(name), rhs) = &line {
            if !matches!(name.as_str(), "x" | "y" | "r")
                && names(rhs).iter().all(|n| env.contains_key(n))
            {
                env.insert(name.clone(), rhs.eval(&env));
                continue;
            }
        }
        match shape(&line, item, &env, view) {
            Ok(Some(s)) => drawn.push((s, style(item, i))),
            Ok(None) => {}
            Err(e) => skipped.push(format!("{latex}: {e}")),
        }
    }

    let view = view.unwrap_or_else(|| fit(&drawn));
    let mut svg = String::new();
    let h = view.height();
    let _ = write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {WIDTH:.0} {h:.1}" width="100%" class="painter-desmos">"#
    );
    if flag("showGrid") {
        grid(&mut svg, view);
    }
    if flag("showXAxis") || flag("showYAxis") {
        let (x0, y0) = view.px(0.0, 0.0);
        if flag("showXAxis") && (0.0..=h).contains(&y0) {
            let _ = write!(
                svg,
                r##"<line x1="0" y1="{y0:.1}" x2="{WIDTH:.0}" y2="{y0:.1}" stroke="#555" stroke-width="1"/>"##
            );
        }
        if flag("showYAxis") && (0.0..=WIDTH).contains(&x0) {
            let _ = write!(
                svg,
                r##"<line x1="{x0:.1}" y1="0" x2="{x0:.1}" y2="{h:.1}" stroke="#555" stroke-width="1"/>"##
            );
        }
    }
    for (shape, st) in &drawn {
        match shape {
            Shape::Curve(runs) => {
                for run in runs {
                    let d: Vec<String> = run
                        .iter()
                        .enumerate()
                        .map(|(k, &(x, y))| {
                            let (px, py) = view.px(x, y);
                            format!("{}{px:.2},{py:.2}", if k == 0 { "M" } else { "L" })
                        })
                        .collect();
                    let dash = st
                        .dash
                        .map(|d| format!(r#" stroke-dasharray="{d}""#))
                        .unwrap_or_default();
                    let _ = write!(
                        svg,
                        r#"<path d="{}" fill="none" stroke="{}" stroke-width="{:.2}" stroke-linecap="round" stroke-linejoin="round"{dash}/>"#,
                        d.join(""),
                        st.color,
                        st.width
                    );
                }
            }
            Shape::Point(x, y) => {
                let (px, py) = view.px(*x, *y);
                let _ = write!(
                    svg,
                    r#"<circle cx="{px:.2}" cy="{py:.2}" r="{:.2}" fill="{}"/>"#,
                    st.point / 2.0,
                    st.color
                );
            }
        }
    }
    svg.push_str("</svg>");
    Rendered { svg, skipped }
}

/// A viewport around what was drawn, with a margin; the Desmos default when nothing was.
fn fit(drawn: &[(Shape, Style)]) -> View {
    let pts = drawn.iter().flat_map(|(s, _)| match s {
        Shape::Curve(runs) => runs.iter().flatten().copied().collect::<Vec<_>>(),
        Shape::Point(x, y) => vec![(*x, *y)],
    });
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for (x, y) in pts {
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    if x0 > x1 {
        return View {
            xmin: -10.0,
            xmax: 10.0,
            ymin: -7.0,
            ymax: 7.0,
        };
    }
    let pad = ((x1 - x0).max(y1 - y0) * 0.08).max(0.5);
    View {
        xmin: x0 - pad,
        xmax: x1 + pad,
        ymin: y0 - pad,
        ymax: y1 + pad,
    }
}

/// Light lines at a round step, about ten across.
fn grid(svg: &mut String, view: View) {
    let span = view.xmax - view.xmin;
    let raw = span / 10.0;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| *s >= raw)
        .unwrap_or(mag * 10.0);
    let h = view.height();
    let mut x = (view.xmin / step).ceil() * step;
    while x <= view.xmax {
        let (px, _) = view.px(x, 0.0);
        let _ = write!(
            svg,
            r##"<line x1="{px:.1}" y1="0" x2="{px:.1}" y2="{h:.1}" stroke="#e5e5e5" stroke-width="1"/>"##
        );
        x += step;
    }
    let mut y = (view.ymin / step).ceil() * step;
    while y <= view.ymax {
        let (_, py) = view.px(0.0, y);
        let _ = write!(
            svg,
            r##"<line x1="0" y1="{py:.1}" x2="{WIDTH:.0}" y2="{py:.1}" stroke="#e5e5e5" stroke-width="1"/>"##
        );
        y += step;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_the_readme_drawing() {
        let state = include_str!("../../../docs/drawings/pirsig.desmos.json");
        let r = render(state);
        assert!(r.skipped.is_empty(), "{:?}", r.skipped);
        let paths = r.svg.matches("<path").count();
        let dots = r.svg.matches("<circle").count();
        assert_eq!(paths + dots, 130, "every expression is drawn");
        assert!(
            r.svg.contains(r#"viewBox="0 0 800 600.0""#),
            "the state's viewport (320x240)"
        );
        assert!(!r.svg.contains("#e5e5e5"), "the state hides the grid");
    }

    #[test]
    fn draws_hand_written_lines() {
        let r = render("a=2\ny=a\\sin(x)\n\\left(\\cos(t),\\sin(t)\\right)\nr=1+\\cos(\\theta)\n(1,1)\nx^2+y^2=4");
        assert!(r.svg.matches("<path").count() >= 3, "{}", r.svg);
        assert_eq!(r.svg.matches("<circle").count(), 1);
        assert_eq!(r.skipped.len(), 1, "the implicit circle is reported");
        assert!(r.skipped[0].contains("implicit"));
    }

    #[test]
    fn breaks_lines_at_poles() {
        let r = render(
            r#"{"expressions":{"list":[{"type":"expression","latex":"y=\\tan(x)"}]},"graph":{"viewport":{"xmin":-5,"xmax":5,"ymin":-5,"ymax":5}}}"#,
        );
        assert!(
            r.svg.matches("<path").count() >= 3,
            "tan(x) is drawn in pieces between its poles"
        );
    }
}
