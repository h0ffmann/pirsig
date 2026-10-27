---
name: zk-method
description: Check that notes follow the Zettelkasten method - review a note, a vault, a slip-box or a ztk brain, "is this a good zettel", "audit my notes", "split this note", "why is my slipbox not connecting", or before importing/exporting notes in ztk. Classifies each note (fleeting, literature, permanent, structure), checks atomicity, own words, links with reasons and sources, runs a mechanical lint over a Markdown folder, and proposes concrete fixes without rewriting the person's thinking.
---

# zk-method: verifying the Zettelkasten method

A Zettelkasten is not a pile of notes. It is a network of small notes, each one an idea in its
owner's words, each one linked to others with a reason. The value is in the links: a note that
links to nothing is a note you will never find again by thinking.

This skill checks notes against the method of Niklas Luhmann as written down by Sönke Ahrens
(*How to Take Smart Notes*, 2017) and the community around zettelkasten.de. It checks; it does not
take over. **The person's ideas and wording are theirs.** Propose a split, a missing link, a sharper
title; never rewrite a note's argument or invent a claim to fill a gap.

## The four kinds of note

Classify every note first. Most problems are a note of one kind dressed as another.

| Kind | What it is | Lives | Must have | Lifetime |
|---|---|---|---|---|
| **Fleeting** | a quick capture: a thought, a quote, a todo | inbox | nothing but the capture | days; processed or deleted |
| **Literature** | what one source says, in your words, with where | per source | a source (citekey, URL, book + page) | permanent, but not the slip-box |
| **Permanent** (the zettel) | one idea of yours, written to be understood in five years by a stranger | the slip-box | a unique ID, a title that is a claim, at least one link with a reason | forever |
| **Structure** (hub, MOC, index) | an entry point: an ordered list of links into a topic, with a sentence each | the slip-box | links with one line of reason each | grows |

Signals: a bare quote or a bullet list with no sentence is fleeting. A note that paraphrases one
source is literature. A note that says what *you* think, and could be argued with, is permanent.
A note that is mostly links is structure.

## What makes a permanent note good

1. **Atomic: one idea.** You can state the idea in one sentence, and the title is that sentence
   ("Spaced repetition beats rereading because retrieval is the practice", not "Memory"). Two ideas
   that could be linked separately are two notes. A note with several `##` sections usually is.
2. **Own words.** Written as an explanation, not copied. A quote is allowed as evidence inside your
   explanation, marked as a quote, with its source. A note that is mostly quote is a literature note.
3. **Self-contained.** It makes sense without the source open and without the note it came from.
   No "as said above", no "this", no undefined abbreviations.
4. **Linked, with a reason.** Every link says *why*: "this contradicts [[202610271200]]",
   "an instance of [[…]]", "see [[…]] for the mechanism". A bare "Related: [[a]] [[b]] [[c]]" list
   is a smell; a link inside a sentence is the method. At least one link to an older note (the note
   joins the conversation) is the minimum; a note with no links is an orphan.
5. **Sourced.** If an idea came from somewhere, the permanent note links the literature note, and
   the literature note carries the citation. Ideas are yours; credit is not optional.
6. **Stable identity.** A unique ID that never changes (`202610271254`, or a Luhmann-style
   `1/3a`), so links survive renames. The title can change; the ID cannot.
7. **Short.** Typically 100 to 500 words. Longer usually means more than one idea or a literature
   summary that has not been distilled.

## What makes a slip-box healthy

- **Inbox drains.** Fleeting notes older than about two weeks are a backlog, not a slip-box.
- **No orphans.** Every permanent note has at least one inbound or outbound link; ideally both.
- **Entry points exist.** Each topic with more than a handful of notes has a structure note.
- **Links are bidirectional in meaning.** If A says "B is a special case of A", B should be
  reachable from A and should know about A (a backlink is enough).
- **Folders do not replace links.** Topic folders inside the slip-box are a smell: Luhmann's box
  was ordered by ID and connection, not by subject. Folders belong to PARA (below), not to ideas.
- **Tags are few and cross-cutting.** A tag that duplicates a link or a folder adds nothing.

## Zettelkasten and PARA in ztk

ztk has both, and they answer different questions:

- **PARA** (projects, areas, resources, archives) is *where the work is*: actionable, organised by
  outcome, and things move to the archive when done.
- **The slip-box** is *what you think*: organised by connection, and nothing is ever archived.

The interop rules:

- A **resource** is the PARA home of a source; its **literature notes** hang off it. Citations live
  there (citekey, URL), so one source has one resource.
- A **project** pulls permanent notes in by link (a project's reading list, its arguments); it does
  not own them. When the project is archived, its notes stay in the slip-box, still linked.
- **Fleeting notes** from a project's work go to the inbox, not to the project; processing them
  is how project work becomes lasting knowledge.
- **Never file a permanent note under a project or area folder.** Link it instead.

When reviewing a ztk brain or an Obsidian vault headed for ztk, flag a permanent note inside a
PARA folder, a literature note with no resource/source, and a project with no linked notes as
"knowledge that will be lost when this is archived".

## Steps

1. **Scope.** Ask, or infer from the request: one note, a set, or the whole vault? Which folders are
   the inbox, the literature notes, the slip-box, and the PARA tree? Read the vault's own
   conventions first (an `ID` format, a template folder, frontmatter keys): **BYOX** in ztk means
   the person's conventions win over this skill's defaults.
2. **Mechanical lint.** For a folder of Markdown notes, run:

   ```sh
   python3 .claude/skills/zk-method/scripts/zk_lint.py <vault> [--inbox Inbox] [--json]
   ```

   It reports, per note: kind (from frontmatter `type:`/tags/folder, else guessed), missing ID,
   orphans, bare link lists, links to notes that do not exist, literature notes without a source,
   long or multi-section permanent notes, and stale fleeting notes. It only finds what can be
   counted: treat its output as a list of places to look, not as a verdict.
3. **Read and judge.** For each note in scope (for a whole vault: every flagged note, plus a sample
   of about ten unflagged permanent notes), read it and check the seven points of "What makes a
   permanent note good". Own words and atomicity need reading; no script can check them.
   - For "own words", compare against the linked source or literature note if there is one. A
     long verbatim run without quote marks is a finding; say where it comes from.
4. **Report.** Group findings by note, most valuable first (an orphan permanent note with a strong
   idea is worth more than a missing ID). For each: what fails, which rule, and the smallest fix,
   written as a proposal:

   ```text
   202610271254 "Memory"  (permanent, 820 words)
     ✗ atomic: three ideas - spacing, retrieval, interleaving
       → split into three notes; suggested titles:
         "Spacing beats massing because forgetting is part of learning"
         "Retrieval is the practice, rereading is not"
         "Interleaving helps discrimination between similar problems"
     ✗ links: "Related" list with no reasons
       → say why each link is there, inside a sentence
   ```

   End with the slip-box view: inbox size and age, orphan count, topics without a structure
   note, PARA interop findings.
5. **Fix only on request.** If the person asks for fixes, make the mechanical ones directly (IDs,
   broken links, moving a note out of a PARA folder, turning a bare link list into a stub sentence
   *they* finish). For splits and rewording, draft and show the draft; keep their sentences, move
   them, do not paraphrase them. Never delete a note; propose deletion of fleeting notes and let
   the person decide.

## Defaults when the vault has no conventions

- ID: `YYYYMMDDHHMM` timestamp, in frontmatter `id:` and as the filename prefix.
- Frontmatter: `id`, `title`, `type` (`fleeting` | `literature` | `permanent` | `structure`),
  `source` (literature: citekey or URL), `tags` (few).
- Links: `[[id]]` or `[[id|readable text]]`, inside sentences.
- Citations: a BibTeX citekey (`@ahrens2017`), so notes can go through pandoc unchanged.

## Sources

- Sönke Ahrens, *How to Take Smart Notes* (2017): fleeting/literature/permanent notes, own words,
  the slip-box as a writing partner.
- Niklas Luhmann, "Kommunikation mit Zettelkästen" (1981): the box as a communication partner,
  fixed IDs, branching (`1/3a`), links over hierarchy. The digitised box is at
  [niklas-luhmann-archiv.de](https://niklas-luhmann-archiv.de/).
- [zettelkasten.de](https://zettelkasten.de/introduction/): atomicity, structure notes, links with
  reasons.
- Tiago Forte, *Building a Second Brain* (2022): PARA.
