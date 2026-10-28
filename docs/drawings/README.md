# Drawings

## `pirsig.svg`

無 (*mu*), by Matheus Hoffmann ([@h0ffmann](https://github.com/h0ffmann)), 2026: the only drawing
in this repository. The Rust binary `pirsig-art` (crate `crates/pirsig-drawings`) writes it in two
formats from one geometry:

| File | Command | What it is |
|---|---|---|
| `pirsig.svg` | `cargo run -q -p pirsig-drawings --bin pirsig-art > docs/drawings/pirsig.svg` | The README drawing |
| `pirsig.desmos.json` | `cargo run -q -p pirsig-drawings --bin pirsig-art desmos > docs/drawings/pirsig.desmos.json` | The same drawing as a Desmos graph state |

The tests (`cargo test -p pirsig-drawings`) check that the committed files are the output of the
binary, and that every Desmos expression, read back from its LaTeX, stays within 0.01 of the line
it stands for.

**What it shows.** The kanji 無, *mu*, "no thing", brushed in its twelve strokes in the standard
order: the falling stroke ノ, the short top horizontal, the long horizontal, the four verticals
from left to right, the bottom horizontal, and the four dots of fire (灬), the first falling to
the left and the other three to the right. Each stroke presses in, carries and lifts as a brush
does: horizontals and verticals stop firmly (*tome*), the first stroke sweeps out to a point
(*harai*), and the dots swell as they fall.

**Why mu.** In *Zen and the Art of Motorcycle Maintenance*, chapter 26, Pirsig takes
*mu*, "no thing", from Zen as the answer to a question that cannot be answered yes or no:
"A very strong case can be made for the statement that science grows by its mu answers more
than by its yes or no answer." ⚠ Page 413 is the page
[LitCharts](https://www.litcharts.com/lit/zen-and-the-art-of-motorcycle-maintenance/symbols/mu)
cites for that line; it does not name the edition, so the page could not be checked against a
copy. Page numbers differ between editions; the chapter does not.

**How it is drawn.** Each stroke is a list of hand-placed points, with a brush width at each. A
Catmull-Rom spline runs through them, so the line is smooth and passes through every point. Each piece of the spline is a cubic
Bézier, and each Bézier is one Desmos parametric expression,
`(a+bt+ct^{2}+dt^{3}, …)` for `0 ≤ t ≤ 1`. A brush stroke cuts each piece in four, each drawn
at the width of its middle, so the width changes smoothly along the stroke. The pen is
`crates/pirsig-drawings/src/pen.rs`; the drawing is `crates/pirsig-drawings/src/pirsig.rs`.

**The look is [picasso](https://github.com/h0ffmann/picasso)'s** (`site/static/drawings/`): one ink,
`#1f2328` on light pages and `#e6edf3` on dark ones (`prefers-color-scheme`), round ends, nothing
else on the page. To see the Desmos expressions over the geometry, as in
[piabrota/alerts](https://github.com/piabrota/alerts/tree/main/docs/drawings), run
`cargo run -q -p pirsig-drawings --bin pirsig-art check > /tmp/pirsig-check.svg`: the geometry in
blue, each expression sampled in orange inside it.

### Open the Desmos file

The file is a Desmos graph state (`version` 11): parametric expressions and points only. Desmos
uses y up, so each y value is `240 - y` of the SVG (the canvas is 240 × 240).

1. Open `https://www.desmos.com/calculator`.
2. Open the browser console.
3. Run `Calc.setState(<the content of pirsig.desmos.json>)`.

⚠ Not checked in Desmos: desmos.com could not be opened from the machine that made this file. The
check was local: each Desmos expression is read back from its LaTeX and compared with its line. Desmos is a free service but not FOSS: the file is data, and Pirsig does
not depend on Desmos.

### Credits

- The stroke order and the structure of 無 follow [KanjiVG](https://kanjivg.tagaini.net/)
  (`kanji/07121.svg`, © Ulrich Apel, CC BY-SA 3.0), used as a reference: the points and widths
  here are drawn by hand, and no KanjiVG path is copied.
- The look (ink, dark mode, a single centred drawing in the README header) follows
  [picasso](https://github.com/h0ffmann/picasso).
- The SVG-plus-Desmos check comes from
  [piabrota/alerts](https://github.com/piabrota/alerts) (`crates/alerts-drawings`).
- To make another drawing this way, use the `ddraw` skill in `.claude/skills/ddraw/`.
