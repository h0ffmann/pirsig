---
name: hippo
description: Work with a Hippo in Pirsig, a rhizome of notes grown around fixations - sort a brain dump, find where they left off, grow a fixation, reconnect scattered notes. Use when the person mentions their Hippo, a fixation, hyperfocus or "where was I".
---

# hippo: a rhizome, not a tree

A **Hippo** is a way of thinking stored in Pirsig: a name, a description and **fixations**
(the subjects its notes gather around). Use only the fixations the person names.

A Hippo's notes form a **rhizome** (Deleuze and Guattari): any note connects to any other,
there is no trunk, and every note is an entry point. Pirsig keeps it that way:

- each note is linked in when it is saved, to notes that share a fixation, a tag or a title
  word, else to the last thing captured; every link says why (`rhizome: ...`);
- `-a` (filing under another note) is refused, because a rhizome has no trunk;
- deleting a note links its neighbours to each other, so nothing is cut off;
- `psg hippo --map ID` shows where they left off, each fixation's notes, and any pieces;
  `psg hippo --weave ID` joins the pieces.

## Start here

```sh
psg hippo                 # which Hippos exist
psg hippo --map ID        # where they left off, the plateaus, the pieces
```

No Hippo yet, and they want one: `psg hippo -c -n NAME --fixation "rogue waves"`.

## How to help

- One step at a time: capture first, sort later, and start from "Where you left off".
- Say exactly what each command will change before you run it, in plain, literal words.
- Nothing is lost: a delete heals the rhizome, the archive keeps everything, and `--dry-run`
  writes nothing.
- A fixation is fuel: give each one its plateau and go deep. To pause one, close it with a
  short note ("parked: ...") so it can wait safely.

## Recipes

- **Brain dump.** Split it into one note per idea, in their words. Show the list, and wait for
  their OK. Then save each one with `psg note -c "..." -H ID`, adding `--tag FIXATION` when it
  belongs to a fixation. Never use `-a`. End with `psg hippo --map ID`.
- **"Where was I?"** Run `psg hippo --map ID` and read back only "Where you left off", then ask
  which one to open (`psg note -s ZID`).
- **A link they feel.** `psg link A B -w "their reason"`. Their reasons beat the automatic
  `rhizome:` ones; never remove one of theirs.
- **Scattered.** If the map shows pieces, offer `psg hippo --weave ID`, which only adds links.

You help with notes, not health. If the person says they are in distress or unsafe, stop the
note work, answer as a person, and point them to someone who can help now (a trusted person,
a local emergency number, or a crisis line).
