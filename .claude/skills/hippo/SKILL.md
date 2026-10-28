---
name: hippo
description: Work with a Hippo in Pirsig, a rhizome of notes for ADHD, autistic, neurotic or obsessive minds - sort a brain dump, find where they left off, grow a fixation, reconnect scattered notes. Use when the person mentions their Hippo, a fixation, hyperfocus or "where was I".
---

# hippo: a rhizome, not a tree

A **Hippo** is a self-described way of thinking stored in Pirsig: a name, traits (`adhd`,
`autistic`, `neurotic`, `obsessive`, any mix) and **fixations** (obsessions, special
interests). It is the person's own word about themselves, never a diagnosis. Do not suggest a
trait they have not given. Do not mention traits back to them unless they do.

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

No Hippo yet, and they want one: `psg hippo -c -n NAME --trait adhd --fixation "rogue waves"`.
Use only the traits and fixations they name.

## Adapt to the traits they gave

| Trait | Help like this |
|---|---|
| `adhd` | One step at a time. Capture first, sort later. Start from "Where you left off". Keep replies short. Never count the backlog at them. |
| `autistic` | Literal, precise words; no idioms. Say exactly what each command will change before you run it. Keep the same order every time. Treat the depth of a special interest as a strength. |
| `neurotic` | State facts that calm: nothing is lost, a delete heals the rhizome, the archive keeps everything, `--dry-run` writes nothing. Avoid alarming words ("error", "broken") when a plain one works. |
| `obsessive` | A fixation is fuel: give each one its plateau and go deep with them. When they ask to stop a loop, close it with a short note ("parked: ...") so it can wait safely. |

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
