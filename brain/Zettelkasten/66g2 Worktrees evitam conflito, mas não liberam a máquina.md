---
pirsig: note
zid: "66g2"
title: "Worktrees evitam conflito, mas não liberam a máquina"
aliases: ["Worktrees evitam conflito, mas não liberam a máquina"]
kind: literature
source: "@ondrej2026workflow"
resource: "[[Resources/My Agentic Engineering Workflow (after 15,700 sessions)]]"
tags: ["agentic-engineering", "status/unverified", "video"]
created: 2026-11-01T00:52:03+00:00
updated: 2026-11-01T00:52:03+00:00
---

# Worktrees evitam conflito, mas não liberam a máquina

Agentes consomem CPU, memória e disco. No mesmo checkout, agentes alteram os mesmos arquivos e brigam entre si. O worktree resolve esse conflito, e David usou essa solução por algumas semanas.

Sobram dois problemas. Cada worktree copia o checkout inteiro, que pode ter gigabytes, e o disco enche depressa. Além disso, todos os agentes continuam na mesma máquina. Um teste pesado, uma renderização ou um agente que clica na tela atrapalham a pessoa.

Esse limite não incomoda quem roda dois ou três agentes. Mas David quer rodar 10, 20 ou 30 em paralelo. David tem um MacBook Pro M3 Max com 128 GB de memória, de cerca de US$ 7.000, e mesmo assim não consegue rodar mais de 50 agentes nessa máquina.

Fonte: David Ondrej, vídeo, [00:29:40-00:30:50](https://www.youtube.com/watch?v=F8rc-ULd4_4&t=1780s). Marca: (v) confere com a legenda automática; a afirmação de David não foi checada. Status: unverified.

## Links

- [[66g1 Agentes na nuvem cresceram, mas as soluções fechadas prendem o usuário|66g1]] Agentes na nuvem cresceram, mas as soluções fechadas prendem o usuário: continua o argumento da nota anterior no vídeo

