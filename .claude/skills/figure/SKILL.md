---
name: figure
description: "Add, change or explain a diagram in this repo's Markdown (README.md, docs): a Mermaid fence or an Archify picture, including the PARA tree that `ztk figure` exports from a brain, and the graphify knowledge graph from `ztk graph`. Use when asked for a diagram, flowchart, mind map, a picture or a graph of something (or of the brain), or to update one after its prose changed."
---

# figure

A diagram is a ```` ```mermaid ```` fence in the page it illustrates; GitHub draws it there. A
flow that a fence draws badly becomes an Archify picture. Every figure has a reading card.

## Steps

1. **Read the prose the figure sits beside.** Draw only what it says; the text is the source of
   truth and a figure that disagrees with it is a bug in the figure.
2. **Write the fence.** First two lines, which Mermaid ignores:

   ```
   %% figure: <id>          lowercase words joined by '-', unique in the repo
   %% title: <question>     the question the figure answers
   ```

   Then the diagram, at most about 15 nodes; split it rather than grow it.
3. **Write the reading card** right after the closing fence (or the embedded PNG):

   ```
   <details open>
   <summary>How to read this figure</summary>

   **Takeaway.** one plain sentence, what to remember
   **How to read.** direction, what shapes and arrows mean
   **Not shown.** what was left out on purpose
   **Evidence.** file, paragraph or command, with ⚠ if unchecked

   </details>
   ```

   `open` keeps the explanation visible under the diagram. Each label is its own paragraph (blank
   lines between). The takeaway has no jargon. Evidence names the file or section the figure
   summarises; a discrepancy found between figure and text, or a claim not checked, is stated
   there with ⚠, not hidden.
4. **Check** that every node and label traces to a file you read. Never remove a ⚠ without doing
   the check.

## Visual grammar, the same in every figure

- Rectangle: a thing or a step. Diamond: a test, written as a question. Every test labels both
  its *yes* and its *no* edge.
- Solid arrow: makes or leads to the next thing. Dashed arrow (`-. label .->`): checks, tests or
  compares against. Label every arrow that is not obvious from its two ends.
- No self-loops: draw a check as a dashed arrow to the thing it is checked against.
- Number boxes that happen in order (`1 ·`, `2 ·`) so the sequence reads without the arrows.
- One flow direction per figure (`LR` or `TD`); groups (`subgraph`) for stages.
- Leave `classDef`, `style` and `%%{init}%%` out. A colour never carries meaning on its own.

## A picture drawn with Archify

Write the source as `docs/archify/<slug>.json` (Archify's typed JSON IR; read the vendored
[`archify` skill](../archify/SKILL.md) for the schema of each `diagram_type`) and run `just figure`:
the vendored archify (`.claude/skills/archify`, the pinned submodule `vendor/archify`) validates and
checks it (`finalize`: schema, layout, routes, real browser), picasso's look
(`docs/archify/picasso.css`) is set over it and `docs/img/<slug>.png` is taken. `just figure-init`
checks the submodule out once; `just figure-check` fails when a source has no PNG, README.md does not
show it, or it changed since the PNG. `docs/archify/crop.tsv` trims empty space under a picture
(`<slug>\t<height>`).

- Commit the JSON, `docs/archify/figures.lock` and the PNG; never the HTML (`build/`).
- Embed the PNG by its relative path with an alt text that says what it shows, and put the reading
  card under it. At most about 10 boxes, short labels, no jargon.
- Look at the PNG at full size and shrunk to 800 px wide before you commit it: every label must read.
- Every box and label traces to a file you read; the card's evidence names it.
- `just figure` needs node, jq, perl and Chrome (`CHROME` names another binary; headless as root
  needs a wrapper that adds `--no-sandbox`). Archify's own browser check uses `ARCHIFY_CHROME`.

## The brain as a picture: `ztk figure` (Rust)

Don't hand-write a PARA tree. `ztk figure` reads the brain through the same model layer as the CLI
and API and writes the Archify tree IR (`diagram_type: "tree"`): the brain at the root, then areas,
their projects, open tasks, and resources under the project or area they belong to (unfiled ones
under their own node). The code is `src/figures/archify.rs`; `GET /api/figures/tree` serves the same.

```sh
ztk figure -o docs/archify/para.json            # --archived, --all (done tasks), -t "title"
just figure                                     # -> docs/img/para.png
```

`just figure-para` rebuilds `docs/archify/para.json` from the README's demo brain in a throwaway
SQLite file; run it when the export format changes. For a figure of a real brain, set
`DATABASE_URL` and write the JSON outside `docs/` unless the user wants it published.

## The brain as a knowledge graph: graphify

[graphify](https://github.com/safishamsi/graphify) clusters, queries and draws a `graph.json`.
`ztk graph` writes one (`src/figures/graphify.rs`, also `GET /api/figures/graph`): every area,
project, task and resource is a node (`file_type` `concept`, resources `document`,
`source_file` `ztk/<kind>s/<id>`), containment is a `contains` edge, all `EXTRACTED`.

```sh
pip install graphifyy==0.9.96                   # once; the CLI is `graphify`
just graph                                      # ztk graph -o graphify-out/graph.json + cluster-only
graphify query "what feeds the marathon?"       # also: path "A" "B", explain "X", god-nodes
```

Output lands in `graphify-out/` (`graph.html`, `GRAPH_REPORT.md`), which is not committed. Use it
to answer questions about the brain's shape; to put a graph in the docs, draw the part that matters
as a Mermaid fence or an Archify picture with a reading card.
