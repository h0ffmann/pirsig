---
type: llm
weight: 1
---

The answer picks an order and gives the next commands, and:

- names a small first step (one to three actions), not all eight problems at once;
- uses real ztk commands for it, such as `ztk project --archive 4`, `ztk task --done 7` or `ztk task -u 7 --due DATE`, `ztk box inbox`, `ztk link ZID OTHER -w why`;
- does not tell the person to delete inbox notes without asking them;
- stays short.
