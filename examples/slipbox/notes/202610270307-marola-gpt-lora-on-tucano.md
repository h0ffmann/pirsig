---
id: "202610270307"
title: "marola-gpt is a LoRA over Tucano-2b4, trained on a pinned corpus"
type: literature
status: verified
source: "marola-dev/marola-ml, README, 'marola-gpt', read by Claude, 2026-10-27"
tags: [water, marola-ml, llm, lora]
created: 2026-10-27
---

# marola-gpt is a LoRA over Tucano-2b4, trained on a pinned corpus

marola-gpt is a LoRA adapter over Tucano-2b4 (an open model trained from scratch in Brazilian
Portuguese), trained on marola-corpus at the version in `corpus.version`, with the base fixed at a
revision. In the local run the validation loss went from 3.17 (base alone) to 3.02, because the
corpus is small.

Two things pinned at once: [[202610270322-pin-everything]].
