# V1 Impactante — estratégia vinculante v1.4.0

## Decisão de produto
A primeira entrega **pública** será uma V1 completa e demonstravelmente diferenciada, não um MVP, demo de tela única ou versão artificialmente limitada. O custo marginal de produzir código com agentes justifica maior ambição de funcionalidade, mas **não elimina** custo de integração, verificação, segurança, manutenção e inferência. Slices e PRs são marcos **internos**, não releases públicos. A primeira experiência deve impressionar pelo resultado e ser tão simples quanto apontar um agente para o repositório e para uma aplicação autorizada.

## Promessa e fronteiras
Redesenhar aplicações web existentes com mudanças significativas de estrutura e experiência, embasadas em tarefas e evidências, respeitando design systems e preservando **comportamentos conhecidos e verificados**. Nunca afirmar extração completa de regras ocultas, paridade integral ou prontidão de produção automática. O produto é open source, agent-first, sem API paga obrigatória; a inferência utiliza o modelo autenticado do host ou backend local suportado. REA e Application MCP são **capacidades incluídas na V1** com ativação seletiva, não dependências obrigatórias em toda execução.

## Capacidades obrigatórias antes da primeira publicação pública
| Frente | Critério de lançamento | Fallback legítimo |
|---|---|---|
| Core Craft | Study→Build→Check operacional; compatibilidade upstream; execução headless via agente | Não há fallback para fluxo principal |
| UX Intelligence | biblioteca rica versionada (IA, formulários, tabelas, densidade, navegação, estados, conteúdo, acessibilidade, fluxos empresariais); recuperação contextual; aplicabilidade/contraindicação; decisões com evidências e alternativas `keep_as_is` | Busca textual local, sem embeddings |
| Redesign de jornadas | transformação estrutural de uma jornada com múltiplas telas e estados, React/TS executável, responsividade quando aplicável | declarar partes não observadas; não inventar backend |
| DS corporativo | ingestão real de tokens e componentes React + pelo menos um caminho documental (Storybook ou docs); política strict/flexible, conformidade de uso e versionamento | modo sem DS quando usuário não fornece um |
| Preservação funcional | Task Model, cenários positivos/negativos, oráculos independentes, Playwright, cobertura e unknowns | declarar inconclusivo, nunca aprovação falsa |
| Investigação avançada | REA disponível para JS/network/state quando útil; Application MCP Bridge disponível com trust gateway, read-only e proveniência | fluxo funciona sem esses conectores |
| Quality Engine | build, checks funcionais, a11y automática e amostra manual, avaliação visual, regressão, segurança e relatório | relatório honesto quando checks não puderem rodar; gates críticos não passam |
| Agent-first / DX | README, AGENTS, CLI/MCP, doctor, setup assistido, sem agente duplicado, testes reais nos agentes anunciados | não anunciar compatibilidade não testada |
| Experiência | uma instrução de entrada; configuração progressiva; preview, comparação antes/depois, decisões, limitações e artefatos exportáveis | CLI/report headless é suficiente; desktop não obrigatório |
| OSS/distribuição | licença/attribution verificadas, SBOM, dependências pinadas, CI, documentação e fixtures reproduzíveis | nenhum asset corporativo privado distribuído |

## Critério de impacto (não só completude)
- Benchmark pré-registrado em ao menos três aplicações/fixtures de naturezas distintas: formulário empresarial denso, jornada multietapa e aplicação com design system real; incluir estados de erro e permissão. Tamanho final da amostra, repetições e thresholds devem ser definidos **antes** da execução e não inventados retrospectivamente.
- Comparação A/B/C: Craft original; Craft + boas instruções de UX; Redesign Anything. Mesmo modelo, permissões, tarefa e orçamento comparável. Avaliação cega humana por tarefas e rubrica; medir eficácia, erros, consistência DS, cobertura funcional, acessibilidade, esforço humano e tokens/tempo. Não usar somente LLM como juiz.
- A V1 precisa mostrar **ganho relevante contra B**, sem regressão crítica de função ou segurança. Se não houver ganho, corrigir antes do release público; não rebatizar uma demo como V1.
- Demo principal: aplicação corporativa web autorizada com jornada completa, DS corporativo e antes/depois executáveis, evidências de decisão e relatório de checks. Uma segunda demo sem DS comprova generalidade.

## Política de arquitetura
**Reutilizar antes de construir.** Craft permanece o motor de engenharia reversa e reconstrução; REA é extensão seletiva, não substituto de Study; DS CLI/Storybook e bibliotecas UX são fontes/adaptadores. Um único orquestrador ativo (`host_agent` XOR `craft_runner`). Knowledge Packs podem ser extensos em disco e mínimos no contexto. Não criar pipeline multiagente obrigatório nem serviço SaaS central. Integrações V1 devem estar implementadas, testadas e disponíveis; ativação por necessidade é otimização, não adiamento de escopo.

## Organização do desenvolvimento
Seis frentes podem avançar em paralelo, com integração contínua: (1) Core/Agent-first/REA/MCP; (2) Design Intelligence e Task Model; (3) DS corporativo; (4) geração e jornadas; (5) Quality/segurança; (6) DX/distribuição. Slices 0–5 de `EXECUTION_V12.md` são **checkpoints internos**, usados para reduzir retrabalho e identificar regressões. Não autorizam release público parcial. Critérios de bloqueio e trust boundaries v1.3 continuam integralmente válidos.

## Não fazer
- Não lançar uma V1 que só redesenha uma tela, sem DS e sem investigação opcional disponível.
- Não exigir REA, Application MCP ou DS para cada tarefa.
- Não aumentar profundidade do agente sem medir benefício.
- Não confundir interface bonita com produtividade, usabilidade ou paridade.
- Não ocultar desconhecidos nem rebaixar controles de segurança para cumprir cronograma.
- Não declarar integração/compatibilidade implementada antes de testes reais.

## Regra de precedência
Este documento substitui qualquer recomendação anterior de **MVP reduzido, P0-minimal ou release por slice** em `SPEC.md`, `EXECUTION_V12.md`, `IMPLEMENTATION_PLAN.md`, `COUNCIL_DECISIONS.md` e `COUNCIL_ROUND_2.md`. Preserva integralmente os requisitos normativos de segurança, oráculos, workspace, custo e qualidade. Em conflito, segurança e honestidade de evidência prevalecem sobre critérios de impacto.
