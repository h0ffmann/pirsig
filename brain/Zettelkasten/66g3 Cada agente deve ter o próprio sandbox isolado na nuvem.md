---
pirsig: note
zid: "66g3"
title: "Cada agente deve ter o próprio sandbox isolado na nuvem"
aliases: ["Cada agente deve ter o próprio sandbox isolado na nuvem"]
kind: literature
source: "@ondrej2026workflow"
resource: "[[Resources/My Agentic Engineering Workflow (after 15,700 sessions)]]"
tags: ["agentic-engineering", "status/unverified", "video"]
created: 2026-11-01T00:52:03+00:00
updated: 2026-11-01T00:52:03+00:00
---

# Cada agente deve ter o próprio sandbox isolado na nuvem

David testou primeiro uma única máquina virtual. O resultado foi melhor que o local, mas 5 ou 10 agentes esgotaram a máquina.

A solução vem do raciocínio por primeiros princípios. Uma empresa dá um computador a cada funcionário novo. Da mesma forma, cada agente deve receber o próprio sandbox. O agente tem recursos próprios e não interrompe os outros.

Essa arquitetura dá dois ganhos. O primeiro é a escala: 10, 100 ou 1.000 agentes funcionam do mesmo jeito. O segundo é a segurança: um sandbox comprometido não contamina os outros e não chega ao MacBook da pessoa. David cita a onda de ataques e vazamentos dos últimos 7 a 14 dias ⚠ como motivo para trabalhar primeiro na nuvem.

Fonte: David Ondrej, vídeo, [00:30:50-00:32:20](https://www.youtube.com/watch?v=F8rc-ULd4_4&t=1850s). Marca: (v) confere com a legenda automática; a afirmação de David não foi checada. Status: unverified.

## Links

- [[66g2 Worktrees evitam conflito, mas não liberam a máquina|66g2]] Worktrees evitam conflito, mas não liberam a máquina: continua o argumento da nota anterior no vídeo
- [[66g7 Agentes na nuvem trabalham com o notebook fechado|66g7]] Agentes na nuvem trabalham com o notebook fechado: o sandbox próprio é o que mantém o agente ativo sem o notebook

