# obsidian-painter

An Obsidian plugin with three features. Its logic is Rust (`crates/obsidian-painter`),
compiled to WebAssembly and bundled into `main.js`; the TypeScript in `src/` only talks to
Obsidian and to the machine (files, `node`, `pandoc`). The code-block approach comes from
[akitaroh/obsidian-mermaid-maker](https://github.com/akitaroh/obsidian-mermaid-maker): a
custom fenced block, rendered as SVG through `DOMParser`, never `innerHTML`.

## 1. Desmos drawings, offline

````markdown
```desmos
y=2\sin(x)
\left(\cos(t),\sin(t)\right)
r=1+\cos(\theta)
```
````

A block holds a Desmos graph state (what `Calc.getState()` returns, such as
[`docs/drawings/pirsig.desmos.json`](../docs/drawings/pirsig.desmos.json)) or one LaTeX expression per
line. It draws points, parametric curves over their domain, `y=f(x)`, `x=g(y)`, polar
`r=f(\theta)`, and assignments (`a=2`) the others use, with Desmos's colours, widths, dashes
and viewport. No API key and no network: the expressions are parsed and sampled in Rust.
Implicit equations and inequalities are listed under the figure as "not drawn".

## 2. Archify figures and HTML composition

````markdown
```archify
{ "diagram_type": "flowchart", ... }
```
````

The block is an [archify](https://github.com/tt-a1i/archify) JSON IR, the same source
`just figure` draws for this README. The plugin runs `archify finalize` with `node`, which
checks the figure and writes an HTML page; the page's styles and SVG go into a shadow root
in the note, so archify's CSS never touches Obsidian's. Pages are cached by content.

**Compose note as HTML page** (command palette) writes `<note>.html` next to the note: the
note through pandoc, each figure in its own declarative shadow root, one self-contained file.

## 3. PDF through pandoc

**Export note to PDF (pandoc)** writes `<note>.pdf` next to the note. Figures are rasterized
at 3× (about 300 dpi). Obsidian syntax becomes Markdown (`[[a|b]]` → `b`, `![[img.png]]`,
`==mark==`, callouts, `%%comments%%` dropped). LaTeX engines get 2.5 cm margins, 11 pt,
coloured links.

## Settings

| Setting | Default | For |
|---|---|---|
| pandoc | `pandoc` | PDF and HTML |
| PDF engine | `xelatex` | any `--pdf-engine`: `lualatex`, `tectonic`, `typst`, `weasyprint`… |
| Extra pandoc arguments | | e.g. `--toc --citeproc` |
| node / archify script | `node` / | archify blocks: `vendor/archify/archify/bin/archify.mjs` after `just figure-init` |
| archify look | | a CSS file over every figure, e.g. `docs/archify/picasso.css` |
| Raster scale | 3 | figure sharpness in the PDF |

## Build, test, install

```sh
just painter        # Rust core → build/painter.wasm → main.js; tsc; node tests
```

Copy `main.js`, `manifest.json` and `styles.css` into `<vault>/.obsidian/plugins/obsidian-painter/`
and enable it. Desktop only (it runs `node` and `pandoc`).

CI (`.github/workflows/obsidian-painter.yml`) checks every change and keeps the zip as an
artifact. A tag `obsidian-painter-v<version>`, matching `manifest.json`, publishes a GitHub
release with the three files and the zip.
