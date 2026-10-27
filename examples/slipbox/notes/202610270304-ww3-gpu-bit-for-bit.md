---
id: "202610270304"
title: "ww3-gpu: WAVEWATCH III on a GPU without changing the answer"
type: literature
status: verified
source: "marola-dev/marola-research, README, experiment 003 (read there on 2026-10-17 at f83aa0c); h0ffmann/h0ffmann, site/data/works.tsv; read by Claude, 2026-10-27. The repository itself is private and was not read."
tags: [water, ww3-gpu, gpu, wave-model]
created: 2026-10-27
---

# ww3-gpu: WAVEWATCH III on a GPU without changing the answer

ww3-gpu ports WAVEWATCH III's Fortran to C++20/Kokkos and checks it bit for bit against the
Fortran with WW3's own regtests, starting with the DIA nonlinear source term (`W3SNL1`). Status
then: 20.12 % of `model/src` translated. The upstream is a public fork, h0ffmann/WW3 (from
NOAA-EMC/WW3), next to h0ffmann/WW4 and h0ffmann/wavespectra.

Its place in the chain: [[202610270323-the-water-pipeline]]. Why it is closed for now:
[[202610270325-public-face-private-working-copy]].
