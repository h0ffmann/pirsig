---
pirsig: note
zid: "66k2"
title: "Um relato vira thread, investigação somente leitura e, com aprovação, correção entregue"
aliases: ["Um relato vira thread, investigação somente leitura e, com aprovação, correção entregue"]
kind: literature
source: "@ondrej2026workflow"
resource: "[[Resources/My Agentic Engineering Workflow (after 15,700 sessions)]]"
tags: ["agentic-engineering", "status/unverified", "video"]
created: 2026-11-01T00:52:04+00:00
updated: 2026-11-01T00:52:04+00:00
---

# Um relato vira thread, investigação somente leitura e, com aprovação, correção entregue

O usuário fala em inglês simples com o agente: "o Cloudroom está lento hoje, avise David". O agente usa a Room CLI, e o relato cai em uma tabela no Postgres. Um loop olha essa tabela a cada minuto. Cada relato novo abre uma thread na nuvem, no espaço de trabalho de David e com inferência paga por David.

Um agente investiga em modo somente leitura. Se o relato é ruído ou duplicata, a thread se arquiva sozinha. Um relato de versão antiga, já corrigido, também desaparece sem gastar tempo humano. Se o relato é real, vira uma issue no GitHub, com um resumo curto.

Então o fluxo espera a decisão humana. David aprova ou recusa. Sem aprovação, a thread é arquivada. Com aprovação, o agente corrige, um modelo de fronteira de outro laboratório revisa, e a correção vai para produção.

Pedidos de recurso seguem o mesmo fluxo, mas a decisão é sempre de produto e fica com a pessoa. Em casos simples e óbvios, a entrega pode ser direta, talvez com uma checagem de outro modelo.

O sistema mantém uma reputação para quem contribui. Um spammer recebe análise mais rigorosa. David diz que é difícil enganar o Opus 5.5 em esforço alto. Com esse sistema, David promete correção em até 24 horas e, no pior caso, 48 horas ⚠.

Fonte: David Ondrej, vídeo, [01:10:30-01:15:40](https://www.youtube.com/watch?v=F8rc-ULd4_4&t=4230s). Marca: (v) confere com a legenda automática; a afirmação de David não foi checada. Status: unverified.

## Links

- [[66k1 O objetivo é reduzir o tempo entre o problema do usuário e o desenvolvedor|66k1]] O objetivo é reduzir o tempo entre o problema do usuário e o desenvolvedor: continua o argumento da nota anterior no vídeo

