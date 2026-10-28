# Pandoc: papers and citations (`pirsig.pub.pandoc`)

## Why

A zettelkasten exists to produce writing. Luhmann's slip box produced books and papers,
not a website. Pirsig should take you from atomic notes to a paper with proper citations
(any CSL style: ABNT, APA, IEEE, Nature, …) and LaTeX-quality typesetting.

Those papers are usually **closed source** until submitted: unpublished results, a
thesis in progress, a paper under review. So the paper pipeline runs entirely on your
machine, inside the private brain, and is independent of [`psg publish`](private-fork-and-publish.md).

[Pandoc](https://pandoc.org) already does the hard parts (Markdown → LaTeX/PDF/DOCX/HTML,
`citeproc`, CSL, cross-references with filters). Pirsig's job is to feed it well.

## What Pirsig adds

| Pirsig side | Pandoc side |
|---|---|
| a **paper** = an ordered list of notes (transclusions) plus metadata | one Pandoc Markdown document with YAML metadata |
| resources with a DOI / ISBN / arXiv id / BibTeX | `bibliography: refs.json` (CSL-JSON, generated) |
| `[[Resource]]` link inside a paper | citation `[@key]` |
| `[[Note]]` link to a note that is in the paper | internal cross-reference `[Note](#note)` |
| `[[Note]]` link to a note that is not in the paper | plain text (or a footnote with `--link-notes=footnote`) |
| labels, PARA ids, private links | stripped |

### Resources become citable

New optional fields on resources: `citation_key`, `doi`, `isbn`, `arxiv`, `csl` (raw
CSL-JSON). `psg resource cite <doi|arxiv|isbn>` fetches the metadata (Crossref, arXiv,
OpenLibrary) into `csl` and picks a stable key (`luhmann1981`). `psg resource import
refs.bib` brings in an existing BibTeX/Zotero library.

### Papers

```sh
psg paper new "Atomic notes for forecasting" -p 3        # inside project 3
psg paper add 1 --note 12 --note 15 --note 9              # order = order in the paper
psg paper build 1 --to pdf --csl abnt.csl                 # → out/atomic-notes-for-forecasting.pdf
psg paper build 1 --to latex                              # hand off the .tex to a journal template
psg paper build 1 --to docx --reference-doc template.docx
psg paper check 1                                         # unresolved citations, missing metadata
```

`psg paper build` writes a Pandoc defaults file and a Markdown source to `out/` (gitignored)
and runs the `pandoc` binary. Pirsig does not link Pandoc as a library: it shells out, which
keeps Pirsig free of Haskell and of Pandoc's GPL. If `pandoc` is missing, the command says
how to install it and still writes the Markdown + `refs.json`, so you can build elsewhere.

Bundled Lua filters (installed under `~/.local/share/pirsig/pandoc/`):

- `pirsig-links.lua`: wikilinks → citations / cross-references, as in the table above;
- `pirsig-strip.lua`: drops labels, frontmatter keys Pirsig uses internally, and anything
  `visibility: private` that is not part of the paper itself.

Cross-references to figures, tables and equations use `pandoc-crossref` when it is
installed.

## Privacy

A paper is `private` like everything else. `psg paper build` only ever writes locally.
To publish the PDF, mark the paper `public` and it goes through `psg publish` like any
item, with the same plan and leak checks.

## Out of scope

Editing in a WYSIWYG editor, journal submission, reference management beyond import and
`cite` (Zotero stays the tool for that; Pirsig reads its export).

## Implementation steps

1. Citation fields on resources, `psg resource cite`, BibTeX import.
2. `papers` table (title, project, ordered note ids, pandoc options).
3. Markdown + CSL-JSON generation, Lua filters, `psg paper build` shelling out to `pandoc`.
4. `psg paper check`.
