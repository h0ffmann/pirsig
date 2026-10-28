# Drawings

## `pirsig.svg`

A head in one line whose brain is a zettelkasten, by Matheus Hoffmann
([@h0ffmann](https://github.com/h0ffmann)), 2026. The Rust binary `pirsig-art` (crate
`crates/pirsig-drawings`) writes the drawing in two formats from one geometry:

| File | Command | What it is |
|---|---|---|
| `pirsig.svg` | `cargo run -q -p pirsig-drawings --bin pirsig-art > docs/drawings/pirsig.svg` | The README drawing |
| `pirsig.desmos.json` | `cargo run -q -p pirsig-drawings --bin pirsig-art desmos > docs/drawings/pirsig.desmos.json` | The same drawing as a Desmos graph state |

A test (`cargo test -p pirsig-drawings`) checks that the committed files are the output of the binary.

**What it shows.** The head faces right and is one line, from the nape round the skull and down the
face to the throat, after Picasso's one-line drawings. The eye is seen from the front in a face seen
from the side, as in his cubist portraits. Inside the skull, the brain is an outline with folds.
The dots are notes on the folds, and the dashed lines are links between notes: the zettelkasten.
The ear is a spiral. The signature `psg` is at the bottom right.

**How it is drawn.** Each line is a list of hand-placed points. A Catmull-Rom spline runs through
them, so the line is smooth and passes through every point. Each piece of the spline is a cubic
Bézier, and each Bézier is one Desmos parametric expression,
`(a+bt+ct^{2}+dt^{3}, …)` for `0 ≤ t ≤ 1`. The pen is `crates/pirsig-drawings/src/pen.rs`; the
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

- The style follows the one-line drawings of [picasso](https://github.com/h0ffmann/picasso)
  (`site/static/drawings/`), after Pablo Picasso's one-line drawings.
- The SVG-plus-Desmos render and its check come from
  [piabrota/alerts](https://github.com/piabrota/alerts) (`crates/alerts-drawings`).
- To make another drawing this way, use the `ddraw` skill in `.claude/skills/ddraw/`.
