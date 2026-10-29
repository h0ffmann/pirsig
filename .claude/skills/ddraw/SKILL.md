---
name: ddraw
description: Draw a picture as Desmos equations - line art, a logo, a README header drawing, "a drawing of X", "make art in Desmos". Makes a beautiful one-line drawing from a prompt, writes it as a Desmos graph state and as an SVG rendered from those same equations, looks at the render and redraws until it is good, and credits any artwork it refers to.
---

# ddraw: drawings from equations

A ddraw drawing is a few confident lines. Each line is a smooth curve through hand-placed points,
and each piece of the curve is one Desmos parametric equation. The same equations make the SVG that
people see, so the Desmos file and the picture never disagree.

The house style is the one-line drawing after Picasso, as in
[picasso](https://github.com/h0ffmann/picasso) (`site/static/drawings/`) and the README drawings of
`piabrota/alerts` and this repository (`docs/drawings/`). Read `docs/drawings/README.md` and
`crates/pirsig-drawings/src/pirsig.rs` once before the first drawing: they are a finished example.

## What makes a drawing good

1. **One idea, said once.** Write the idea in one sentence before drawing ("a head whose brain is a
   zettelkasten"). Every line serves that sentence. If a line does not, take it out.
2. **Few lines, long lines.** The main subject is one continuous stroke where it can be. Picasso's
   dog is one line. Ten short strokes read as a sketch; one long stroke reads as a drawing.
3. **One twist.** Give the drawing one surprise that only a drawing can make: a frontal eye in a
   profile (cubism), a spiral that is also an ear, a wave that is also a Fibonacci spiral. One, not
   three.
4. **Weight says what matters.** The subject's contour is heavy (3.2), inner detail is medium
   (1.8 to 2.2), connections and hints are light (1 to 1.4) or dashed. Never more than three weights.
5. **Empty space is part of it.** Leave at least 8% of the canvas empty on every side. Do not fill a
   region because it is empty.
6. **Colour is rare.** One line colour. An accent colour, if any, goes on one small thing (the
   strings of the guitar, the beak of the owl) and never carries meaning alone.
7. **Curves, not wobble.** Put points where the curve turns, not along straight runs. A turn needs a
   point at its peak and one on each side; two points too close make a kink.
8. **A signature is small** and sits in a corner, in the line colour, at the light weight.

## Steps

1. **Brief.** Write: the idea (one sentence), the twist, the canvas (width × height; 320 × 240 or
   320 × 180 for a README header), and what is *not* in the drawing.
2. **Compose on the grid.** SVG coordinates, y down. Block the big shapes first as rough rectangles
   in a comment: where the subject sits, where the empty space is. Then list the points of the main
   stroke, going round the shape in one direction.
3. **Write the drawing.** In this repository, copy `crates/pirsig-drawings/src/pirsig.rs` to a new module
   and use the pen in `pen.rs`:

   | Pen call | Makes | Desmos |
   |---|---|---|
   | `stroke(&points, closed, width)` | a smooth line through every point (Catmull-Rom → cubic Béziers) | one parametric expression per Bézier |
   | `link(a, b, bend, width)` | a gently bent dashed line from `a` to `b` | the same, `lineStyle: DASHED` |
   | `dot(at, size)` | a filled dot (`size` is Desmos `pointSize`) | a point |

   A procedural part (a spiral, a lobed outline) is a function that returns points: see `ear()` and
   `brain_outline()` in `pirsig.rs`. Keep hand-placed points as `const` arrays, so the shape is
   readable.
4. **Render and look.** Write the SVG and render it:

   ```sh
   cargo run -q -p pirsig-drawings --bin pirsig-art > docs/drawings/<name>.svg
   NODE_PATH=$(npm root -g) node .claude/skills/ddraw/scripts/shot.js docs/drawings/<name>.svg /tmp/<name>.png 3
   ```

   Open the PNG with the Read tool. **Always look; never ship a drawing you have not seen.**
5. **Critique, then redraw.** Check each item of "What makes a drawing good" against the picture and
   write down what fails: overlaps that were not meant, kinks, crowding, a line that says nothing.
   Move points, delete lines, redraw. Three rounds at least; stop when a round finds nothing.
   Look at scale 1 too: at README size, thin detail vanishes.
6. **Write the Desmos file** (`… pirsig-art desmos > docs/drawings/<name>.desmos.json`) and make the
   test that compares the committed files with the binary's output pass.
7. **Credit.** Say in `docs/drawings/README.md` what the drawing follows (an artist, a famous work, a
   repository). A drawing beside someone else's artwork loads that artwork from its source and
   links it, with title, author and the claim that makes it notable (a contest place, with year).
   Never copy someone else's graph or picture into the repository. Mark with ⚠ every claim you
   could not check.

## Desmos reference

- **A graph state** is JSON: `{"version": 11, "graph": {"viewport": {xmin, xmax, ymin, ymax},
  "showGrid": false, "showXAxis": false, "showYAxis": false}, "expressions": {"list": [...]}}`.
  Load it on desmos.com/calculator from the browser console with `Calc.setState(state)`.
- **y is up in Desmos and down in SVG.** A Desmos y is `H - y` of the SVG. Get this wrong and the
  drawing is upside down.
- **A cubic Bézier** `P0 P1 P2 P3` is, in power form, `a + bt + ct² + dt³` with `a = P0`,
  `b = 3(P1 − P0)`, `c = 3(P0 − 2P1 + P2)`, `d = P3 − P0 + 3(P1 − P2)`, for each axis. In LaTeX:
  `\left(12-3t+0.25t^{2}+t^{3},40+6t\right)` with `"parametricDomain": {"min": "0", "max": "1"}`.
- **An arc** is `\left(c_x+r\cos(t),c_y+r\sin(t)\right)` over `[θ0, θ1]`.
- **A point** is `\left(x,y\right)` with `"pointSize"`. Line expressions take `"lineWidth"`,
  `"lineStyle"` (`SOLID`, `DASHED`, `DOTTED`) and `"color"` (hex).
- **Numbers**: three decimals are enough. Write `-3` not `+-3`.
- **Thumbnails** of a saved graph: `https://saved-work.desmos.com/calc_thumbs/production/<hash>.png`,
  where the graph is `https://www.desmos.com/calculator/<hash>`.
- **Famous Desmos art** to learn from and to credit: the
  [Desmos Global Math Art Contest](https://www.desmos.com/art) (yearly since 2020), and
  [Harry Chen's graphs](https://harry7557558.github.io/desmos/), among them *Ammonite* (1st place,
  2021, ages 17-18).

## The render

The SVG people see is ink, as in [picasso](https://github.com/h0ffmann/picasso)'s drawings:
`#1f2328` on light pages and `#e6edf3` on dark ones, through `prefers-color-scheme` in the
SVG's own `<style>`. `pirsig-art check` writes the check render, in two layers as in
`piabrota/alerts`:

- **Blue** (`#1682D4`): the geometry, drawn as exact SVG paths.
- **Orange** (`#FF4F00`, thin): each Desmos expression, read back from its LaTeX and sampled. Where
  they agree the orange line runs inside the blue one, which proves the Desmos file is the picture.

A test does the same in numbers: no expression may stray 0.01 from its line. If one does, the
LaTeX is wrong: fix the pen, not the drawing.

## Outside this repository

Without the `pirsig-drawings` crate, do the same by hand: write the points, compute each Bézier's power
form, write the graph state as JSON, and render the curves to SVG with the same coefficients. Keep
the steps: brief, compose, draw, look, critique, redraw, credit.
