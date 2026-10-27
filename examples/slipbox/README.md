# A sample slip box: h0ffmann's repositories

The worked example of [`docs/use-cases.md`](../../docs/use-cases.md).

- [`para.sh`](para.sh) seeds a ztk database with the PARA map: four areas (the four lights), six
  projects, the public repositories and forks as resources, the archived ones in the archive.
- [`notes/`](notes/index.md) is a slip box of 29 notes in the owner's format
  (h0ffmann/h0ffmann, `NOTES.md`): 21 literature notes, one idea from one README each, and 8
  permanent notes, the ties between them. Start at [`notes/index.md`](notes/index.md).

`cargo test --test mod slipbox` checks the notes: ids, front matter, links, the index.
