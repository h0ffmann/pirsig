# O fluxo de engenharia com agentes de David Ondrej

## Como ler este texto

Este texto indexa o vídeo "My Agentic Engineering Workflow (after 15,700 sessions)", publicado por David Ondrej em 2026-10-30. O vídeo dura 1 h 25 min 40 s. O texto cobre o vídeo do início ao fim, na ordem da fala.

**Quem fala.** Todo o texto relata o que David diz. Uma frase sem atribuição é fala de David, e não um fato checado. Um comentário nosso começa com `Leitura do indexador:`.

**Marcas.** Cada nota termina com o trecho do vídeo que a sustenta. O indexador leu a legenda inteira. A marca `(v)` diz que a nota confere com a legenda. A marca não diz que David está certo. O sinal `⚠` aponta um nome ou um número duvidoso. A lista completa está na última parte.

**Conflito de interesse.** David criou o Cloudroom e vende esse produto no vídeo. Leia com essa informação à mão todo elogio ao Cloudroom e toda crítica aos concorrentes.

**Vocabulário.** Estes termos têm um sentido fixo neste texto.

- **Agente:** um modelo de IA que usa ferramentas e trabalha em ciclos até concluir uma tarefa.
- **Harness:** o programa que roda o agente, como Claude Code, Codex, Pi, Hermes e OpenCode.
- **Modelo de fronteira:** o modelo mais capaz de um laboratório em um dado momento.
- **Thread:** uma sessão de conversa com um agente.
- **Subagente:** um agente que outro agente inicia para executar uma parte do trabalho.
- **Agente na nuvem:** um agente que roda em uma máquina remota, e não no computador da pessoa.
- **Sandbox:** um ambiente isolado que pertence a um único agente.
- **Worktree:** uma cópia de trabalho de um repositório Git na mesma máquina.
- **Skill:** um arquivo de instruções que o agente carrega para executar um procedimento.
- **Loop:** um agente agendado que repete uma tarefa sem pedido humano.
- **ADE:** ambiente de desenvolvimento com agentes, o aplicativo em que a pessoa comanda as threads.

![fig](mapa)

## Parte 1: A promessa e os modelos

### [k=credenciais t=00:00-00:50] David promete mostrar o fluxo que desenvolveu em mais de 2.500 horas
-> #9: este vídeo trata de agentes e harnesses de código, o tema deste índice de repositórios

David programa com IA há quatro anos. A conta de David soma mais de 2.500 horas e três startups de IA. David também entrevistou pessoas conhecidas nessa área: o CEO da Vercel, o CTO da Supabase, o CTO da Fireworks e Matt PCO ⚠. Para David, programar com IA parece trapaça quando a pessoa domina o método.

O vídeo promete cinco respostas. Quais modelos, harnesses e assinaturas compensam agora. Como David revisa código para entregar direto em produção. Quando usar subagentes. Quando usar agentes na nuvem. Quais loops fazem boa parte do trabalho diário.

Este vídeo atualiza outro, feito dois meses antes, que chegou perto de 200.000 visualizações. David afirma que muita coisa mudou nesse intervalo.

### [k=modelos t=00:50-02:00] Opus 5.5 e Astra são os dois únicos modelos na fronteira real

O modelo que David mais usa é o Opus 5.5, da Anthropic. David considera esse modelo o melhor do mundo e também rápido. Fable 5.5 pode chegar em breve. Enquanto isso, David roda o Opus no Claude Code. Quando o limite das duas assinaturas acaba, David roda o mesmo modelo em outro harness, como o Cursor CLI.

Na OpenAI, a escolha fica entre Astra e Soul ⚠. David prefere o Astra, porque esse modelo tem "cheiro de modelo grande". David acrescenta que a OpenAI está atrás e precisa lançar um modelo melhor.

Google, Meta e Kimi ⚠ fazem bons modelos, e David ainda usa esses modelos. Mas David não coloca nenhum deles na fronteira real. Esse lugar é só de Opus e Astra.

### [k=harness t=02:00-02:30] O harness importa cada vez menos quando o modelo é forte
-> ade-labs: o critério de aceitar todo harness parte desta tese de que o modelo pesa mais que o harness

David usa vários harnesses: Pi, Claude Code, Hermes e OpenCode. Mesmo assim, David vê o harness perder importância. Em um evento no escritório da Anthropic, dois engenheiros da empresa contaram a David o que esperam da próxima geração de modelos. A lição principal foi esta: dê a um modelo forte acesso ao terminal e à internet, e o harness quase deixa de importar. O resultado seria parecido no Claude Code, no Devin ou no Droid.

### [k=assinaturas t=02:30-03:30] A assinatura Claude Max de US$ 200 por mês dá o melhor retorno agora

David ordena as assinaturas por retorno. Os planos Claude de US$ 100 e US$ 200 por mês ficam em primeiro lugar, bem à frente do Codex. O plano ChatGPT Pro fica em segundo lugar, com limites que mudam a toda hora.

Com uma única escolha, David ficaria com o Claude Max de US$ 200, sem disputa.

Quem tem pouco dinheiro pode assinar o OpenCode Go, de US$ 10. Os produtos da Cursor e da SpaceX AI servem para quem gosta do Grogbot ⚠. David chama o Google de piada, mas vê promessa no Argon e no Carbon ⚠. O Carbon pode ser o melhor modelo do mundo, porém ainda não saiu.

### [k=ade-cloudroom t=03:30-04:20] David usa só o Cloudroom, um ADE próprio, gratuito e aberto

No lugar de ADE, David usa apenas o Cloudroom. David criou esse aplicativo. O produto é gratuito, tem código aberto e está em cloudroom.dev. David afirma que o Cloudroom é o único aplicativo de código com IA que oferece agentes na nuvem no plano gratuito ⚠. A pessoa pode escolher o modelo e usar as assinaturas que já paga, tanto no local quanto na nuvem.

Leitura do indexador: esta é a primeira de várias menções ao produto. A alegação "o único no mundo" não foi checada.

## Parte 2: O que aprender

### [k=arquiteto t=04:20-05:50] O engenheiro de agentes deve pensar como arquiteto, e não decorar sintaxe

Para David, decorar sintaxe de Python ou TypeScript já não vale a pena. Reconhecer se um trecho é C++ ou Rust é um truque bonito, mas sem valor. Vale entender as ideias altas: o que é uma linguagem com tipos, por que Rust é rápido, como Rust protege a memória.

A competência certa é pensar como arquiteto de software. David recomenda um vídeo de Mark Richards sobre esse tema. O arquiteto olha uma tela e formula hipóteses. A página abriu na hora? Talvez exista pré-carregamento, cache ou carregamento otimista. O profissional precisa de muita largura e de pouca profundidade em cada item.

### [k=conceitos t=05:50-06:40] Basta conhecer os conceitos de software, porque o modelo escreve a função

David mostra uma lista de conceitos: compromissos de projeto, cache, observabilidade, raio de explosão, acoplamento e coesão. Quem constrói software deve saber o que esses termos significam. A pessoa não precisa ser especialista. A pessoa também não precisa escrever a função do zero, porque o Opus 5.5 escreve essa função. O essencial é saber que o conceito existe.

Para cada conceito desconhecido, David sugere 30 minutos de estudo. O melhor professor é a própria IA. David diz que usar IA para aprender é um dos melhores previsores de sucesso que já viu. David também anuncia um tema para o fim do vídeo: o software que melhora a si mesmo conforme os usuários crescem.

## Parte 3: Como David revisa e entrega

Esta é a parte que David considera a mais valiosa. David pediu ao Opus que analisasse todas as sessões e desenhasse o processo real.

### [k=medir t=06:40-08:00] Quem trabalha com IA precisa medir a produtividade para não se enganar

David diz que entrega centenas de mudanças por dia em produção. O gráfico do GitHub mostra pouca atividade até março e abril de 2026, e depois uma aceleração. Em setembro e outubro, com os agentes na nuvem, David passou a 100 a 200 contribuições por dia. Um segundo gráfico, feito pelo Opus a partir dos dados do GitHub, mostra 700 commits em produção por semana. David descreve a curva como exponencial.

### [k=uma-thread t=08:00-09:30] Cada thread deve carregar uma única tarefa

David pergunta ao Cloudroom quantas threads estão abertas. A resposta é 550, depois de chegar a mais de 800 no dia anterior. Com esse volume, a pergunta central é como saber se o código é bom, sem bugs e sem falhas de segurança. Revisar ficou cada vez mais importante.

A primeira regra é separar o trabalho. Uma thread nova recebe uma única tarefa, como um chamado claro no Jira. David mostra exemplos reais: uma thread investiga por que o OBS quebrou, outra prepara o vídeo, outra limpa o disco. Se um assunto novo aparece, David faz um fork para outra sessão.

### [k=reproduzir t=09:30-11:00] Um bug deve ser reproduzido de ponta a ponta antes da correção

O fluxo começa com uma pergunta: a tarefa é um bug? Se for, o agente tenta reproduzir o problema como um usuário real. David não aceita correção baseada em descrição vaga. O agente pode rodar um teste, uma simulação ou um experimento para entender a falha.

Depois o Opus 5.5 escreve a correção. Em seguida entram testes, checagem de tipos e, quando a interface muda, capturas de tela do aplicativo real.

David considera as capturas de tela uma arma forte. David pede ao agente que imagine o aplicativo no estilo Windows 7 e mostre três versões. Nesse trecho, David também anuncia o suporte oficial ao Windows, lançado na véspera. Antes o Cloudroom só rodava em macOS e Linux.

### [k=skills t=11:00-12:50] Uma mudança grande ou arriscada aciona skills de decisão e de segurança
-> gargalo: a skill de risco protege clientes, e a frase sobre quebrar produção puxa no sentido oposto

Se a tarefa não é um bug, a pergunta é outra: a mudança é grande ou arriscada? Nesse caso, David usa skills do repositório público davidondrej/skills.

A skill 4x4 faz quatro blocos de quatro perguntas, 16 decisões no total. Os blocos são produto, desenho de software, uso prático e casos de borda. David só roda essa skill em mudanças grandes.

A skill de mudanças arriscadas funciona como trava de segurança para erros que atingem clientes. Os agentes acionam essa skill sozinhos. A skill nasceu de um erro real: um filtro passou em todos os testes e, com dados reais, desligou 99% do recurso. David diz que as próprias skills costumam nascer de lições caras.

David dá uma dica extra. Todo prompt frequente vira um atalho no Raycast ou nas substituições de texto do macOS. Quando o procedimento é maior que um prompt, vira skill. Em mudança pequena, o Opus apenas constrói.

### [k=testes t=12:50-14:00] A maioria dos testes escritos por agentes é lixo e deve ser evitada
-> reproduzir: o teste estratégico cobre uma classe de erro, e a reprodução cobre um bug concreto

David assume uma posição contrária: testes costumam ser superestimados. Segundo David, mais de 90% dos testes que os agentes escrevem são lixo. Esses testes incham o aplicativo, atrasam o trabalho e não verificam nada. O agente escreve o teste só para o teste passar.

Testes ainda têm função. Se os agentes repetem o mesmo tipo de erro, David pede um ou dois testes estratégicos para impedir a repetição. Fora disso, David não quer centenas de milhares de linhas de teste.

Os modelos já escrevem testes por conta própria, e cada geração melhora. Por isso, David manda o agente parar de escrever testes. Mesmo com essa ordem, o agente ainda escreve alguns. Para David, essa sobra costuma ser a quantidade certa.

### [k=outro-lab t=14:00-15:50] Mudança média ou grande deve ser revisada pelo modelo de fronteira do outro laboratório
-> loop-bug: o loop de bugs automatiza esta revisão cruzada entre laboratórios

Depois de construir, David decide a revisão pelo tamanho da mudança. Em mudança normal, David roda a skill de revisão com GPT. O motivo é simples: quem escreveu foi o Opus.

A regra exige modelos de fronteira. David avisa que o método não funciona com Haiku, Luna ou um modelo local de 27 B ⚠. Quem só usa modelo barato vê falhas constantes e não entende como alguém entrega centenas de commits por dia. David usa Opus 5.5 em esforço extra alto ou máximo em tudo.

A regra de revisão é cruzada. Quem constrói com Opus ou Fable revisa com o modelo de fronteira da OpenAI, em esforço extra alto ou máximo. Quem prefere GPT revisa com o Opus 5.5. David estima que esse único hábito reduz em cerca de 90% os bugs sérios, as quebras e o lixo ⚠.

### [k=revisao-total t=15:50-16:40] Revisar várias vezes seguidas gera falsos problemas
-> triagem: a triagem filtra os falsos problemas que a revisão repetida gera

Para uma mudança enorme e arriscada, David tem a skill de revisão total. Essa skill usa Fable, Opus, Astra, Soul e Muse ⚠, gasta muitos tokens e demora. O resultado é uma lista sem duplicatas com os achados mais sérios. David roda essa skill só algumas vezes por semana.

No dia a dia, David constrói com o Opus e roda a revisão com GPT. Se a revisão encontra muita coisa, David corrige e talvez rode a revisão mais uma vez.

O erro comum é repetir a revisão quatro vezes em sequência. Para David, isso produz lixo. Quando alguém manda um modelo achar bugs, o modelo acha algo, mesmo sem bug sério. O modelo pode até alucinar um problema, porque essa foi a tarefa recebida.

### [k=triagem t=16:40-18:30] Cada achado recebe nota em frequência e gravidade, e o Opus decide o que é obrigatório

Cada problema recebe uma nota de 1 a 10 em dois eixos: frequência e gravidade. O Astra acha casos de borda que podem ser graves, mas quase nunca acontecem. O problema real é o grave e frequente. A pessoa decide pelo próprio julgamento. Se a revisão acha seis problemas, a pessoa pode corrigir só os três mais sérios.

O melhor caminho é devolver a lista ao Opus. O Astra encontrou cinco problemas? David pergunta ao Opus: "isto é um problema real, obrigatório, ou é exagero?". David chama essa pergunta de ótimo prompt. A resposta costuma ser concordar com quatro e descartar um.

Se o problema é obrigatório, David aprova e o agente corrige. Na dúvida, David pede uma segunda opinião de outro modelo. Revisar de novo a correção é opcional e costuma atrasar. O rigor depende do contexto. Com milhares de clientes pagantes, a pessoa precisa de mais cuidado. No começo, a pessoa precisa construir rápido e aceitar quebrar coisas.

![fig](revisao)

![fig](triagem)

### [k=ship t=18:30-19:40] A entrega termina com um loop que caça efeitos colaterais
-> loops-fabrica: o loop de efeitos colaterais é um exemplo de loop que compõe a fábrica

Quando nada falta corrigir, o trabalho vai para produção. O agente atualiza a branch com a `main`, e a CI roda as checagens.

Depois da entrega, entram os loops de efeitos colaterais e de segurança. David explica o risco: mesmo com revisão, uma mudança rápida pode quebrar outra parte do código. Um único loop dedicado a esse risco pega 95% desses casos, segundo David ⚠. Muitos problemas são corrigidos antes de chegar aos clientes.

David avisa que este processo deve mudar em um mês.

## Parte 4: A mentalidade dos 100%

### [k=cem t=19:40-21:30] O ganho real só aparece quando agentes fazem 100% do trabalho
-> loops-fabrica: os loops são o caminho prático para chegar aos 100%

David cita um artigo de Jacob ⚠ e diz que a maioria dos fundadores que conheceu em São Francisco ainda não abraçou a IA por inteiro.

David explica com uma conta. Um trabalho toma 8 horas. A pessoa passa metade aos agentes. Os agentes fazem essas 4 horas em 10 a 20 minutos, mas as outras 4 horas continuam manuais. O total cai para cerca de 4 h 10 min. A pessoa sente que avançou, mas o ganho é modesto. David chama esse ganho de redução de cerca de 45%.

O salto de 10, 20, 50 ou 100 vezes só aparece com 100%. Nesse caso, agentes escrevem e revisam todas as linhas.

Leitura do indexador: a conta é uma versão da Lei de Amdahl. A parte lenta que sobra limita o ganho total. Pelos números de David, a redução é de 48%, e não de 45%. A figura mostra o cálculo.

![fig](cem)

### [k=gargalo t=21:30-22:30] A pessoa é o gargalo enquanto escreve código à mão

Quem usa IA em 50% ou 60% do trabalho segue como a peça mais lenta da máquina. David compara a decisão à pílula vermelha de Matrix.

O resumo da mentalidade tem duas partes. Agentes corrigem todo erro que um usuário encontra. Agentes verificam o próprio trabalho sem humano no meio. David ainda trabalha nessa segunda parte, com loops.

David prevê que, no último trimestre de 2026 e em 2027, quem rodar uma fábrica própria de software vai disparar à frente. Essa fábrica é uma série de loops sem humano. David conclui com uma frase forte: não importa se agentes quebram a produção. Importa que os agentes corrijam a falha sozinhos e depressa.

Leitura do indexador: essa frase contradiz em parte a skill de mudanças arriscadas, que nasceu para proteger clientes. David não trata dessa tensão.

## Parte 5: Subagentes

### [k=sub-quando t=22:30-24:00] Subagentes servem para trabalho claro, paralelo e divisível em partes previsíveis
-> ade-cli: a demonstração divide 553 threads entre subagentes, o caso previsto aqui

David cita um tuíte de Max ⚠: orquestrar subagentes é a astrologia de quem programa por intuição. Muita gente se orgulha do número de subagentes, quando um único agente de topo faria melhor.

Subagentes servem quando o trabalho é claro, paralelo e divisível em blocos previsíveis. Um exemplo é um escritório de advocacia com 2.000 documentos. Outro exemplo é a análise de milhares de sessões de agentes ou de uma pilha de PDFs. O processo é sempre o mesmo: ler e classificar.

Nesse caso, um agente principal inteligente, como o Opus 5.5, inicia 10 ou 20 subagentes pequenos, como o Haiku 5.5. Os subagentes rodam em paralelo e devolvem os resultados.

### [k=sub-nao t=24:00-25:50] Trabalho com surpresas exige o modelo mais forte, sem delegar

A maior parte do trabalho não tem essa forma. Quem constrói algo novo encontra surpresas, becos sem saída e mudanças inesperadas. Descobrir a causa de um bug exige ler bastante e pensar. Para esse trabalho, David não usa subagentes.

David descreve o desastre típico. Um modelo pequeno recebe um bug simples. Depois o modelo descobre que o bug está acoplado a outra parte do código, e que essa parte toca a cobrança. David quer um modelo forte em qualquer correção, recurso novo ou refatoração grande. Toda surpresa e toda chance de otimizar merecem o modelo mais inteligente com esforço máximo.

### [k=sub-regra t=25:50-27:00] Na dúvida, a resposta sobre subagentes é não
-> scan: o scan é um caso que passa na regra: entrada, processo e saída conhecidos

Subagentes só compensam quando a pessoa conhece a entrada, o processo e o formato da saída. A regra de David é curta: na dúvida, não use. A rotina de David é ter vários agentes de topo com modelo de fronteira, sem dividir o trabalho para ir mais rápido.

David nunca usa os modos ultra do Codex ou de outras ferramentas. A pessoa não sabe o que acontece lá dentro e desperdiça tokens. Em alguns casos, a pessoa paga cinco vezes mais para terminar duas vezes mais rápido. Com assinaturas tão valiosas, gastar o limite cinco vezes mais depressa para ganhar 1,5 ou 2 vezes não compensa ⚠.

![fig](subagentes)

## Parte 6: Nuvem contra local

David defende que quase todo agente deve rodar na nuvem. Aqui o interesse comercial pesa, porque o Cloudroom vende essa arquitetura.

### [k=nuvem-lock t=27:00-29:40] Agentes na nuvem cresceram, mas as soluções fechadas prendem o usuário
-> ade-assinatura: a prisão ao fornecedor reaparece como critério de escolha do ADE

David mostra um gráfico da Cursor. No começo de 2026, cerca de 1 em cada 10 pull requests que chegavam à produção vinha de agentes na nuvem. No meio do ano, o valor era cerca de 50%. Agora caminha para 60% a 70% ⚠.

David adotou a nuvem mais tarde, por um motivo. Antes do Cloudroom, todas as soluções eram fechadas. A pessoa não podia inspecionar, hospedar por conta própria nem fazer fork. David afirma que o Cloudroom é o primeiro agente na nuvem de código aberto do mundo ⚠. David também diz ter inventado a execução de qualquer harness como agente na nuvem ⚠.

O risco do fechado é a prisão ao fornecedor. A solução obriga o uso dos modelos e da assinatura da empresa. David dá um exemplo: quando a OpenAI retirou os modelos da Cursor, o Astra ficou indisponível ali por três semanas, justamente quando era o melhor modelo.

### [k=worktrees t=29:40-30:50] Worktrees evitam conflito, mas não liberam a máquina

Agentes consomem CPU, memória e disco. No mesmo checkout, agentes alteram os mesmos arquivos e brigam entre si. O worktree resolve esse conflito, e David usou essa solução por algumas semanas.

Sobram dois problemas. Cada worktree copia o checkout inteiro, que pode ter gigabytes, e o disco enche depressa. Além disso, todos os agentes continuam na mesma máquina. Um teste pesado, uma renderização ou um agente que clica na tela atrapalham a pessoa.

Esse limite não incomoda quem roda dois ou três agentes. Mas David quer rodar 10, 20 ou 30 em paralelo. David tem um MacBook Pro M3 Max com 128 GB de memória, de cerca de US$ 7.000, e mesmo assim não consegue rodar mais de 50 agentes nessa máquina.

### [k=sandbox t=30:50-32:20] Cada agente deve ter o próprio sandbox isolado na nuvem
-> sempre-ligado: o sandbox próprio é o que mantém o agente ativo sem o notebook

David testou primeiro uma única máquina virtual. O resultado foi melhor que o local, mas 5 ou 10 agentes esgotaram a máquina.

A solução vem do raciocínio por primeiros princípios. Uma empresa dá um computador a cada funcionário novo. Da mesma forma, cada agente deve receber o próprio sandbox. O agente tem recursos próprios e não interrompe os outros.

Essa arquitetura dá dois ganhos. O primeiro é a escala: 10, 100 ou 1.000 agentes funcionam do mesmo jeito. O segundo é a segurança: um sandbox comprometido não contamina os outros e não chega ao MacBook da pessoa. David cita a onda de ataques e vazamentos dos últimos 7 a 14 dias ⚠ como motivo para trabalhar primeiro na nuvem.

![fig](isolamento)

### [k=noventa t=32:20-34:00] David prevê 90% do trabalho na nuvem e 10% no local

A previsão de David é esta: cerca de 90% das tarefas vão para a nuvem, e cerca de 10% ficam no local. O local serve para assuntos privados ou para organizar a pasta de downloads, sem enviar um grande volume de arquivos.

David mostra os próprios dados. Mesmo depois de começar o Cloudroom, só 10% a 20% dos agentes de David rodavam na nuvem. Nas últimas três ou quatro semanas, a parte da nuvem passou de 50% e depois de 60%. David projeta 90% a 95%.

David também explica quem resiste. Pessoas que juram trabalhar só no local tiveram uma experiência ruim com a nuvem. Faltavam skills, chaves de API, repositórios ou dependências. Para David, o trabalho dessas pessoas não é local por natureza.

### [k=acesso t=34:00-36:40] O Cloudroom permite que o agente na nuvem leia arquivos do Mac

David mostra como o Cloudroom ataca essas falhas. Uma tela de configuração define quais skills, servidores MCP e chaves de API entram em todo sandbox.

O segundo recurso resolve o esquecimento. Um agente falha quando a pessoa pede algo e não anexa o arquivo. David dá um exemplo: "veja o PDF nos meus downloads e calcule quanto devo nesta fatura". David afirma que essa tarefa falharia em Codex, Claude Desktop e Cursor. No Cloudroom, o agente na nuvem pode alcançar o MacBook. A pessoa controla esse acesso em quatro níveis: desligado, somente leitura, perguntar e completo.

David testa ao vivo. O agente conta os arquivos da pasta de downloads e soma o tamanho total, que é de 2,5 GB.

### [k=velocidade t=36:40-38:20] A nuvem só convence quando parece tão rápida quanto o local

David aponta outro motivo de rejeição: em outros produtos, a máquina remota leva 10 a 30 segundos para iniciar ⚠. No teste, o agente começou a raciocinar em cerca de 1 segundo, segundo David. O agente confirma que roda em um sandbox Linux na nuvem e que acessou a pasta do Mac. David diz que nenhum outro produto oferece esse acesso direto ⚠.

David pede também um número real. O Cloudroom informa 553 threads abertas. Dessas, 353 estão na nuvem, o que dá 64% contra 36%. David espera que essa parte se aproxime de 100%, porque o acesso direto elimina os motivos para ficar no local.

### [k=sempre-ligado t=38:20-39:30] Agentes na nuvem trabalham com o notebook fechado

David diz que devia ter citado antes a vantagem mais óbvia. Agentes na nuvem nunca param. Agentes seguem às 3h da manhã, durante um apagão ou em uma viagem de trem com sinal ruim. Agentes têm sempre internet e eletricidade. Para David, essa constância é a base de um negócio apoiado em agentes.

## Parte 7: Como escolher um ADE

David lista os critérios de escolha. Todos os critérios favorecem o produto de David, então leia esta parte como argumento de venda.

### [k=ade-labs t=39:30-41:20] Um ADE precisa aceitar todo harness de ponta, e por isso não pode vir de um laboratório

O primeiro critério é ter todos os harnesses de ponta: Claude Code, Codex, Pi, Hermes, Droid, Devin, OpenCode e outros. O Claude Desktop serve para modelos da Anthropic e não oferece o resto. Nesse aplicativo, a pessoa não testa um DeepSeek novo nem roda a revisão com GPT. O Codex tem o problema inverso: tinha uma ótima interface no começo do ano, mas não roda Fable nem Opus.

Por isso, David conclui que o ADE não pode vir de um laboratório. David responde também à pergunta sobre concorrência. A OpenAI e a Anthropic não abrem o código e nunca vão aceitar modelos alheios. O Cloudroom tem mais de 230 forks, cerca de duas semanas depois do lançamento ⚠.

### [k=ade-assinatura t=41:20-41:50] O ADE deve aceitar as assinaturas que a pessoa já paga

O segundo critério é usar os tokens já pagos. Ferramentas de terceiros, como Cognition, Windsurf, Cursor e AM ⚠, obrigam o uso de assinatura e harness próprios. David acha isso absurdo para quem já paga US$ 200 por mês ao Claude Max ou ao ChatGPT Pro.

### [k=ade-cli t=41:50-45:30] Agentes devem controlar o ADE por linha de comando

O terceiro critério é que agentes possam iniciar outros agentes. A demonstração tem duas partes. Na primeira, o Opus 5.5 divide as 553 threads entre 20 subagentes Haiku 5.5, locais, para classificar o trabalho. Na segunda, um agente inicia cinco threads de topo na nuvem, cada uma com Codex e GPT-6 Astra.

Essas cinco threads não são subagentes. São agentes de topo, em outro harness e com outro modelo. David diz que o Claude Desktop não permite controlar threads por linha de comando.

O Cloudroom tem a Room CLI, com uma skill própria. Com essa ferramenta, um agente lê qualquer thread, abre novas e arquiva antigas. David manda parar os 20 subagentes, e o agente principal interrompe todos. Depois David manda arquivar.

A ideia geral é esta: o agente deve conseguir fazer tudo o que a pessoa faz. Clicar para abrir 20 threads é, para David, o trabalho em mina de carvão dos dias atuais.

## Parte 8: Loops

Esta parte é a mais longa e traz o maior conselho do vídeo: a pessoa deve sair do meio do fluxo.

### [k=loops-fabrica t=45:30-47:10] A fábrica de software nasce de um loop de cada vez

Para ganhar a produtividade do gráfico, a pessoa precisa se retirar do processo. O caminho é achar as partes repetitivas e previsíveis do trabalho e transformar essas partes em loops.

"Fábrica de software" é um termo que assusta, e David recomenda não tentar construir uma. A pessoa começa com loops pequenos e previsíveis para a tarefa mais chata. Com 10, 20 ou 30 loops ativos, a fábrica já existe.

O salto recente de David veio daí. Nas últimas duas ou três semanas, David passou de 400 ou 500 contribuições por semana para 700, quase 1.000 na semana atual. David diz trabalhar das 9h às 22h ou 23h, sete dias por semana. As horas têm limite, e por isso David precisa automatizar.

### [k=nao-copie t=47:10-48:20] Skills e loops alheios não devem ser copiados às cegas
-> primeiro-loop: a alternativa a copiar é extrair o loop das próprias sessões

David pede que a pessoa pare de copiar skills e loops de outras pessoas. A regra vale para as skills do próprio David. Cada pessoa deve escolher duas, três ou quatro skills úteis, e só essas.

O exemplo é Matt PCO ⚠. Muita gente copia tudo que Matt PCO faz. David pergunta se o dia dessas pessoas é igual ao de Matt PCO. Skills e loops devem resolver problemas reais de quem usa. Pessoas diferentes têm rotinas parecidas, mas nunca iguais.

### [k=madeira t=48:20-50:10] Automatize o corte de madeira e guarde para si a construção da catedral
-> dez-noventa: o 10/90 aplica o critério da madeira dentro de cada hábito

David explica o critério com Minecraft. Toda grande obra precisa de material. Cortar madeira é chato e repetitivo, e nenhuma pessoa ganha prestígio com isso. Construir uma catedral exige criatividade, planejamento e visão. A maioria dos jogadores nunca faz isso.

Num jogo, quem automatiza a coleta é chamado de trapaceiro. Nos negócios, tudo que é legal vale. Quem automatiza 80% do trabalho e cresce mais rápido sente que trapaceia.

O resultado é tempo para a catedral: invenção, planejamento e gosto. O corte de madeira fica com os agentes, de ponta a ponta.

![fig](madeira)

### [k=loop-bug t=50:10-51:30] Dois modelos de laboratórios diferentes bastam para entregar um bug sem humano

O melhor exemplo é o relatório de bug fácil de reproduzir. No painel de David, há 33 loops ativos. Esses loops somam cerca de 2.000 execuções por dia, e 86% rodam sozinhos.

O fluxo é este. O Opus confirma que o bug é real. O GPT 6.1 Soul ⚠ faz uma revisão adversária: isso é um bug real ou exagero? O segundo modelo tenta reproduzir a falha. Se os dois modelos concordam que o bug afeta usuários e que a correção é clara e segura, a correção vai para produção sem humano.

David também aceita humano no meio. Um recurso pedido por 70% dos clientes pagantes pode virar três variações, e a pessoa escolhe uma. Um loop pode ser autônomo, supervisionado ou condicional.

### [k=loop-alterna t=51:30-53:40] Loops devem alternar modelos, porque cada modelo enxerga falhas diferentes
-> outro-lab: explica por que a revisão cruzada funciona: cada modelo enxerga falhas diferentes

A maior parte dos loops de David roda com Opus 5.5 ou GPT 6.0 Soul ⚠. O Sonnet entra quando David quer economizar. Alguns loops alternam entre modelos, e David explica o motivo.

Pense em pedir opinião a cinco amigos inteligentes. Mesmo que um seja o mais inteligente, cada um vê algo diferente. O mesmo vale para Opus, Soul, Astra, Muse Wonder, Gemini Argon e Kimi 3.1 Pro ⚠.

O exemplo é a segurança web. Nenhum desenvolvedor revisa a segurança a cada quatro horas, mas um loop revisa. David responde também a quem defende o código escrito à mão. As falhas achadas agora vêm de código humano. Falhas de dia zero ficaram 12 anos escondidas, até que enxames de agentes encontraram essas falhas.

### [k=primeiro-loop t=53:40-55:20] O primeiro loop é o mais difícil e deve sair das suas próprias sessões

David explica por que as pessoas não adotam loops: o atrito inicial é grande. Com 30 loops, o 31º é fácil. O salto difícil vai de zero a dois.

A resposta está no próprio trabalho. Quem usa Claude Code, Codex, Hermes, Pi ou OpenCode guarda um histórico de sessões. David chama esse histórico de mina de ouro. As sessões registram hábitos, prompts, sequências de skills e o modo de resolver problemas.

Essa análise é barata. O recurso de scan do Cloudroom custa menos de 1% do limite semanal ⚠. Isso acontece porque o scan divide o trabalho entre subagentes Haiku. David lembra que este é o caso ideal de subagentes: trabalho previsível e divisível.

### [k=scan t=55:20-57:50] O scan lê milhares de sessões e sugere dois loops feitos para a pessoa

David roda o scan ao vivo. Um agente principal Opus 5.5 inicia subagentes Haiku. Neste caso, são 31 fatias de trabalho, sobre cerca de 3.000 sessões locais. O scan procura padrões repetidos e sugere um ou dois loops próprios da pessoa, e não de um influenciador.

O scan não propõe automatizar tudo. Esse ponto é a inovação de David. A pessoa pode não confiar no agente para a maior parte do trabalho, porque essa parte pede gosto, contexto e julgamento. Por isso, o scan procura dentro de cada hábito um pedaço pequeno e seguro.

### [k=dez-noventa t=57:50-59:10] A divisão 10/90 deixa o agente agir só no pedaço seguro e óbvio

David dá um nome ao método: divisão 10/90. O loop identifica os 10% mais seguros do trabalho e age sozinho nessa parte. Nos outros 90%, o loop analisa, pesquisa, apresenta o que faria e espera a opinião da pessoa.

A vantagem é psicológica e prática. A pessoa liga os dois primeiros loops sem medo de ver lixo chegar à produção. Quando o loop não tem certeza, nada é entregue.

![fig](scan)

### [k=changelog t=59:10-60:00] O changelog público é o teste contra quem ostenta números

David dá um conselho contra a ostentação de PRs, commits e arquivos Markdown. David reage com ironia ao relato de Garry Tan ⚠ sobre meio milhão de arquivos Markdown. David pergunta o que esse número mede, além da quantidade de lixo.

O teste é pedir o changelog público. O Cloudroom publica mudanças cerca de duas vezes por dia, segundo David. Quando alguém disser que roda bilhões de subagentes, a pessoa deve perguntar onde está o changelog e por que ele não é público.

### [k=sugestoes t=60:00-64:00] Os loops sugeridos para David saíram de hábitos reais

O scan de David conclui que a maior parte do trabalho repetido já roda em loops: triagem de feedback, efeitos colaterais, revisão de PRs e publicação.

Mesmo assim, uma rodada anterior sugeriu dois loops novos. O primeiro é a checagem de versão, diariamente às 9h do horário do Pacífico. O segundo é a varredura de threads antigas, diariamente às 19h do mesmo horário. Essa varredura decide o que arquivar. A rodada atual repetiu a checagem de versão e trouxe o foco matinal. Esse loop nasceu de um hábito novo de David, o de desafixar threads, porque David acumula threads fixadas demais.

David insiste que o scan não alucina e não copia a pilha de um influenciador. O método vale para marketing, vendas e pesquisa, porque todo trabalho tem partes repetíveis. A sensação de automatizar a primeira parte é libertadora. O tempo recuperado pode ir para trabalho melhor, para uma caminhada ou para a academia.

## Parte 9: A lição das revoluções

### [k=revolucoes t=64:00-66:00] Em cada revolução, as pessoas pararam de fazer a tarefa e passaram a operar a máquina
-> gargalo: a história dá o argumento para sair do papel de executor da tarefa

David ataca as crenças que limitam a pessoa. Uma crença diz que a IA é fraca. Outra diz que o trabalho da pessoa é especial demais. Uma terceira diz que tudo já acabou. Para David, as três são falsas.

A história mostra o padrão. Na década de 1830, nas tecelagens, as pessoas pararam de tecer à mão e passaram a operar teares mecânicos. Em 1913, Henry Ford criou a linha de montagem. O trabalho deixou de ser construir cada carro e passou a ser operar a linha. Em 1946, andares inteiros de computadores humanos faziam contas à mão. Depois, as pessoas passaram a usar calculadoras e a programar a máquina.

David conclui que nosso tempo não é tão diferente. O mesmo vai acontecer com tarefas que hoje parecem normais.

![fig](revolucoes)

### [k=trabalho-manual t=66:00-68:00] Quem domina agentes passa a enxergar o trabalho manual ao redor

Preencher formulários e pesquisar à mão parecem agora trabalho braçal a David. Pesquisar sem uma ferramenta como o DeepAPI é, para David, inaceitável. A descrição do vídeo traz um link para o DeepAPI. David não declara nessa fala se tem relação com o produto ⚠.

David lista as tarefas que já automatizou ou pode automatizar. Revisão de segurança do código a cada 3 horas. Relatório diário do funil. Monitor de tuítes sobre o produto. Lembrete de quem contatar. Relatórios de bug a cada minuto.

Mesmo quando a decisão final é humana, a IA pode fazer 80% antes: desenhar, implementar, revisar e mostrar três interfaces. A pessoa entra para escolher, aprovar ou recusar.

Para encerrar, David retoma o conselho sobre fábricas. No vídeo com Dex, Dex tentou construir uma fábrica durante quatro meses, e o resultado foi lixo e usuários irritados. O caminho é o inverso: um loop, depois outro. Marketing, prospecção e pesquisa também aceitam loops.

## Parte 10: Software que melhora a si mesmo

### [k=feedback t=68:00-70:30] O objetivo é reduzir o tempo entre o problema do usuário e o desenvolvedor
-> loop-bug: o relato do usuário alimenta o loop que corrige bugs

Todo bug relatado no Cloudroom passa por uma análise do Opus em 10 a 30 minutos. A análise separa bug legítimo de spam, porque agentes também enviam lixo a repositórios abertos.

A entrada é simples. O usuário clica em "reportar bug" ou pede ao próprio agente que relate o problema. O agente também pode relatar sozinho um erro que encontrou. A ideia central é reduzir o atrito entre o problema e o desenvolvedor. A meta de David é corrigir em menos de 24 horas.

David ainda não transformou esse fluxo em recurso para outros produtos.

### [k=autoaprimoramento t=70:30-75:40] Um relato vira thread, investigação somente leitura e, com aprovação, correção entregue

O usuário fala em inglês simples com o agente: "o Cloudroom está lento hoje, avise David". O agente usa a Room CLI, e o relato cai em uma tabela no Postgres. Um loop olha essa tabela a cada minuto. Cada relato novo abre uma thread na nuvem, no espaço de trabalho de David e com inferência paga por David.

Um agente investiga em modo somente leitura. Se o relato é ruído ou duplicata, a thread se arquiva sozinha. Um relato de versão antiga, já corrigido, também desaparece sem gastar tempo humano. Se o relato é real, vira uma issue no GitHub, com um resumo curto.

Então o fluxo espera a decisão humana. David aprova ou recusa. Sem aprovação, a thread é arquivada. Com aprovação, o agente corrige, um modelo de fronteira de outro laboratório revisa, e a correção vai para produção.

Pedidos de recurso seguem o mesmo fluxo, mas a decisão é sempre de produto e fica com a pessoa. Em casos simples e óbvios, a entrega pode ser direta, talvez com uma checagem de outro modelo.

O sistema mantém uma reputação para quem contribui. Um spammer recebe análise mais rigorosa. David diz que é difícil enganar o Opus 5.5 em esforço alto. Com esse sistema, David promete correção em até 24 horas e, no pior caso, 48 horas ⚠.

![fig](autoaprimoramento)

## Parte 11: Inferência própria

David fecha com hardware. A posição de David é moderada: a fronteira serve para o trabalho difícil, e o local serve para privacidade e autonomia.

### [k=hardware-nuance t=75:40-77:00] Modelos locais têm valor, mas o debate público perdeu a nuance

David usa modelos de fronteira quase sempre. Mesmo assim, mantém estrutura própria. David tem dois DGX Sparks e comprou mais dois. Quatro Sparks somam 512 GB de memória.

David diz que ninguém tem uma opinião matizada. Um lado diz que dá para rodar algo como o Opus 5 em casa. O outro lado diz que hospedar por conta própria é inútil. David rejeita os dois.

Para David, quem não tem computação local é "um escravo" da assinatura. Se a assinatura acabar, a pessoa fica sem IA.

### [k=hardware-protecao t=77:00-79:30] Possuir hardware protege a pessoa, e o valor desse hardware cresce com o tempo
-> privacidade: a proteção contra crises tem um uso diário na privacidade

David compara o hardware próprio a ter painéis solares ou uma horta em casa. Para David, o recurso mais valioso do futuro será o token de inteligência.

Todo mês, o mesmo hardware roda modelos melhores. O que rodava nos dois Sparks seis meses atrás era fraco. Agora os dois rodam o DeepSeek V4.1 Flash ⚠. Com quatro Sparks, David poderá rodar mais instâncias, ou um modelo maior, ou o mesmo modelo mais rápido.

O preço do hardware sobe, mas o valor que o hardware produz sobe mais rápido. David cita gráficos da a16z ⚠: GPUs de seis anos, como a A100, ainda ganham valor de saída de tokens, embora muita gente esperasse depreciação em três ou quatro anos.

Essa proteção cobre um governo tirânico, a nacionalização de um laboratório ou outra crise. Talvez a pessoa não rode nada do tamanho do Fable ou do Astra, mas consegue rodar algo.

### [k=privacidade t=79:30-81:20] O modelo local serve para dados privados e para perguntas sem filtro

O uso prático é a privacidade. A equipe de David envia contratos legais e dados de funcionários ao DeepSeek hospedado nos Sparks. Nenhum desses dados vai à OpenAI, à Anthropic ou ao Google. Para esses casos, um modelo com cerca de 90% da qualidade da fronteira basta ⚠.

O segundo uso são modelos sem restrição. David diz que ChatGPT e Claude dão respostas enviesadas em temas políticos e controversos, e que o Grok é um pouco melhor. Para uma pergunta honesta, David recorre a esses modelos.

### [k=fronteira-padrao t=81:20-83:30] A fronteira segue como regra para trabalho difícil, e o local exige investimento
-> modelos: a regra de uso volta aos modelos de fronteira do início do vídeo

David traz um terceiro motivo. Em ataques famosos, os modelos de fronteira recusaram ajuda à vítima, e as pessoas recorreram a GLM ou DeepSeek para se defender ⚠.

A regra de uso continua clara. Para trabalho difícil, a pessoa deve usar o melhor modelo disponível. Para algo importante, sensível ou privado, deve ter a opção local.

O obstáculo é o hardware. Um notebook comum limita tanto o modelo que a diferença para a fronteira desanima. David destaca o Mac Studio M5 Ultra de 512 GB, que teria a mesma memória de quatro Sparks e banda cinco ou seis vezes maior ⚠. Quem investe milhares ou dezenas de milhares de dólares fica protegido de crises: uma guerra mundial ⚠, a falência de um laboratório ou uma proibição de exportar tokens. Um assunto sensível, como a saúde de um filho, também pode ficar em casa.

![fig](inferencia)

### [k=precos t=83:30-84:20] Quem espera o preço do hardware cair vai esperar muito

David diz que deve comprar o Mac Studio, que começa perto de US$ 22.000 a US$ 23.000 ⚠. Os Sparks já custam 30% mais do que na compra anterior, feita meses atrás ⚠. A previsão de David é de preços ainda mais altos.

David encerra o tema com um pedido de bom senso. Use o local quando fizer sentido e a fronteira quando fizer sentido. David critica os absolutistas dos dois lados e diz que a maioria das pessoas não pensa com nuance.

### [k=fim t=84:20-85:40] David encerra com o convite para testar o Cloudroom

O fechamento é comercial. David repete que o Cloudroom é gratuito, aberto e disponível para Windows, macOS e Linux. David pede que a pessoa relate problemas pelo botão ou pelo agente, com a frase "report this to David". A pessoa pode usar o harness favorito com as assinaturas atuais. David termina com as duas alegações principais: o melhor ADE do mundo e a única ferramenta com agentes na nuvem no plano gratuito, para qualquer harness ⚠.

## O que não foi checado

**Status das notas.** Todas as notas ficam com `status: unverified`. O indexador comparou cada nota com a legenda, mas não checou as afirmações de David em fontes independentes. O dono do cérebro decide o que vira nota permanente.

**Legenda automática.** O vídeo não tem legenda humana. O texto usa a legenda automática em inglês, e a legenda erra nomes. A descrição do vídeo confirma David Ondrej, Cloudroom e DeepAPI. As outras correções abaixo são leitura do indexador.

- `David Andre` virou David Ondrej, conforme o título do canal.
- `cloud room` virou Cloudroom, conforme a descrição.
- `Enthropic`, `cloth code`, `Versell`, `Superbase` e `A6Z` viraram Anthropic, Claude Code, Vercel, Supabase e a16z. Estas correções são prováveis, não confirmadas.
- `DJX Sparks` virou DGX Sparks. Correção provável.
- `Gary Tan` virou Garry Tan. Correção provável.

**Nomes mantidos com ⚠.** A grafia destes nomes não foi confirmada: Matt PCO, Jacob, Max, Dex, Kimi, Grogbot, AM, Soul, Astra, Luna, Muse, Muse Wonder, Carbon e DeepSeek V4.1 Flash. "27 B" na legenda provavelmente significa um modelo de 27 bilhões de parâmetros.

**Números sem checagem.** Horas de uso, commits por semana, 33 loops, 2.000 execuções por dia, 86% de autonomia, 95% de efeitos colaterais capturados e redução de 90% nos bugs são números declarados por David. O vídeo não mostra o método desses números. O título fala em 15.700 sessões, e a descrição fala em 10.540 commits por ano. O texto não reconcilia esses números com os 700 commits semanais, que dariam cerca de 36.400 por ano se a taxa fosse constante.

**Alegações de pioneirismo.** "Primeiro do mundo", "único no mundo" e "melhor ADE do mundo" são alegações comerciais. O indexador não pesquisou concorrentes.

**Trecho duvidoso.** Em 1:22:46, a legenda diz `World War II` em uma hipótese sobre o futuro. O texto escreve "uma guerra mundial".

**Datas.** Os preços, as versões de modelos e os estados de produto valem para a data do vídeo, 2026-10-30.
