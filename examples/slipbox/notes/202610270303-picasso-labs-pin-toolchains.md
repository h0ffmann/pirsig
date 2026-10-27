---
id: "202610270303"
title: "picasso labs pin the toolchains of the other works"
type: literature
status: verified
source: "h0ffmann/picasso, README, 'In one minute' and 'Who uses picasso', read by Claude, 2026-10-27"
tags: [ideas, picasso, nix, reproducibility]
created: 2026-10-27
---

# picasso labs pin the toolchains of the other works

picasso (formerly `nix-config`) is a set of labs: self-contained Nix flakes that pin scientific,
publishing and agentic toolchains. A lab references nothing outside itself, so another repository
borrows it as a flake input, a sparse git submodule or a GitHub Action. ww3-gpu takes the
`publisher` lab both ways.

One instance of [[202610270322-pin-everything]].
