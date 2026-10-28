---
type: llm
weight: 1
---

The answer picks an order and gives the next commands, and:

- names a small first step (one to three actions), not all eight problems at once;
- uses real Pirsig commands for it, such as `psg project --archive 4`, `psg task --done 7` or `psg task -u 7 --due DATE`, `psg box inbox`, `psg link ZID OTHER -w why`;
- does not tell the person to delete inbox notes without asking them;
- stays short.
