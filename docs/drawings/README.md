# Drawings

## `pirsig.svg`

無 (*mu*) inside an open ensō, by Matheus Hoffmann ([@h0ffmann](https://github.com/h0ffmann)),
2026. The Rust binary `pirsig-art` (crate
`crates/pirsig-drawings`) writes the drawing in two formats from one geometry:

| File | Command | What it is |
|---|---|---|
| `pirsig.svg` | `cargo run -q -p pirsig-drawings --bin pirsig-art > docs/drawings/pirsig.svg` | The README drawing |
| `pirsig.desmos.json` | `cargo run -q -p pirsig-drawings --bin pirsig-art desmos > docs/drawings/pirsig.desmos.json` | The same drawing as a Desmos graph state |

A test (`cargo test -p pirsig-drawings`) checks that the committed files are the output of the binary.

**What it shows.** The kanji 無, *mu*, "no thing", brushed in its twelve strokes in the standard
order: the falling stroke ノ, the short top horizontal, the long horizontal, the four verticals
from left to right, the bottom horizontal, and the four dots of fire (灬), the first falling to
the left and the other three to the right. Each stroke presses in, carries and lifts as a brush
does: horizontals and verticals stop firmly (*tome*), the first stroke sweeps out to a point
(*harai*), and the dots swell as they fall. Around it, an ensō, the Zen circle of one stroke,
begun pressed and left open as the ink runs dry.

**Why mu.** In *Zen and the Art of Motorcycle Maintenance*, chapter 26, Pirsig takes
*mu*, "no thing", from Zen as the answer to a question that cannot be answered yes or no:
"A very strong case can be made for the statement that science grows by its mu answers more
than by its yes or no answer." ⚠ Page 413 is the page
[LitCharts](https://www.litcharts.com/lit/zen-and-the-art-of-motorcycle-maintenance/symbols/mu)
cites for that line; it does not name the edition, so the page could not be checked against a
copy. Page numbers differ between editions; the chapter does not.

**How it is drawn.** Each stroke is a list of hand-placed points, with a brush width at each. A Catmull-Rom spline runs through
them, so the line is smooth and passes through every point. Each piece of the spline is a cubic
Bézier, and each Bézier is one Desmos parametric expression,
`(a+bt+ct^{2}+dt^{3}, …)` for `0 ≤ t ≤ 1`. A brush stroke cuts each piece in four, each drawn
at the width of its middle, so the width changes smoothly along the stroke. The pen is `crates/pirsig-drawings/src/pen.rs`; the
drawing is `crates/pirsig-drawings/src/pirsig.rs`.

**The default render is the Desmos render**, the same as in
[piabrota/alerts](https://github.com/piabrota/alerts/tree/main/docs/drawings). `pirsig.svg` draws the
geometry in blue. On top, in orange, it draws each expression of `pirsig.desmos.json`, read back from
its LaTeX and sampled at 25 points. Where the two agree, the orange line runs inside the blue line.

### Open the Desmos file

The file is a Desmos graph state (`version` 11): parametric expressions and points only. Desmos
uses y up, so each y value is `240 - y` of the SVG.

1. Open `https://www.desmos.com/calculator`.
2. Open the browser console.
3. Run `Calc.setState(<the content of pirsig.desmos.json>)`.

⚠ Not checked in Desmos: desmos.com could not be opened from the machine that made this file. The
check was local: the orange layer is each Desmos expression, read back and drawn over the blue
geometry, and the two match. Desmos is a free service but not FOSS: the file is data, and Pirsig does
not depend on Desmos.

### Credits

- The stroke order and the structure of 無 follow [KanjiVG](https://kanjivg.tagaini.net/)
  (`kanji/07121.svg`, © Ulrich Apel, CC BY-SA 3.0), used as a reference: the points and widths
  here are drawn by hand, and no KanjiVG path is copied.
- The ensō follows the Zen brush circle (*ensō*, 円相).
- Earlier versions of this logo followed the one-line drawings of
  [picasso](https://github.com/h0ffmann/picasso), after Pablo Picasso.
- The SVG-plus-Desmos render and its check come from
  [piabrota/alerts](https://github.com/piabrota/alerts) (`crates/alerts-drawings`).
- To make another drawing this way, use the `ddraw` skill in `.claude/skills/ddraw/`.
