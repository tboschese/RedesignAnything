# Changelog — Spec Redesign Anything

## Documentação 1.2.0 — 19/11/2026

- `COUNCIL_ROUND_2.md` acrescenta uma segunda revisão crítica com oito perspectivas, divergências, votos condicionais, lacunas e decisão GO/NO-GO para os slices iniciais.


- Conselho simulado de software, IA, design e produto consolidado; inclusão de segurança, QA, DX/OSS e domínio corporativo na revisão seguinte.
- `SPEC.md` recebe adendo vinculante v1.2: Task Model, melhoria mensurável, políticas de DS em três categorias, checks antecipados, early stopping e simplificação por evidência.
- `EXECUTION_V12.md` substitui a ordem anterior de PRs por slices verticais, mantendo backlog detalhado histórico.
- `BENCHMARK_PROTOCOL.md` define braços A/B/C/D, avaliação cega, controle experimental e critérios de parada.
- `COUNCIL_DECISIONS.md` registra decisões e hipóteses não comprovadas.
- `VALIDATION_MATRIX.md` ganha T-045 a T-056.
- Schemas JSON 1.0.0 mantidos para compatibilidade; Task Model inicial é opcional até implementação/migração formal.
- **Sem testes de integração ou benchmark executados.**

## Documentação 1.1.0 — 19/11/2026

Atualização motivada pela análise de viabilidade da execução via agente e estimativas de tokens.

- Introduz `COST_AND_EXECUTION.md` com controlador único (`host_agent` / `craft_runner`), modos Lean/Deep, recuperação sob demanda, regras anti-loop, limites por adapter e telemetria honesta.
- Inclui `execution` opcional em `redesign-config.schema.json`, sem tornar incompatível o contrato de persistência 1.0.0. Exige `allow_nested_builder=false`; em `host_agent`, `model.use_craft_selected_backend=false`.
- Adiciona `usage-snapshot.schema.json`, com dados de entrada/saída/cache, medição `reported`/`estimated`/`unavailable`, custo `null` quando não observável e exemplos sintéticos.
- Atualiza `SPEC.md`, `CONTRACTS.md`, `AGENT_FIRST.md`, `AGENTS_REDESIGN.md`, `IMPLEMENTATION_PLAN.md` e `VALIDATION_MATRIX.md` para tornar controle de custo e agente único requisitos P0.
- Amplia a matriz de testes T-031 a T-044 para orçamento, reinício, fallback, cache, payloads, handoff e benchmark Craft/Lean/Deep.
- Corrige o gerador de schemas para usar caminho relativo ao kit, em vez de um diretório temporário de execução.
- **Sem teste real de integração com Cursor/Craft/REA nesta revisão.** Os nove JSON Schemas e os exemplos foram validados por JSON Schema Draft 2020-12.

## Documentação 1.0.0 — 19/11/2026

Especificação inicial, arquitetura derivada do Craft, contratos semânticos, bibliotecas de UX, design system corporativo, opcional REA/MCP da aplicação, segurança, validação, agent-first e implementação em PRs.

## v1.3.0 — incorporação normativa do Council II (19/11/2026)

- Quatro políticas normativas: trust boundaries, oráculos independentes, workspace/patch seguro, DX/OSS/upstream.
- Adendos vinculantes em SPEC, execução, backlog, benchmark, agent-first, custos e validação.
- Testes negativos T-057 a T-074 definidos, **não executados**.
- O relatório `COUNCIL_ROUND_2.md` continua como registro da revisão; suas recomendações críticas passam a requisitos explícitos.
- Nenhuma implementação de software, benchmark real, criação de GitHub ou aprovação de segurança é reivindicada.

## v1.4.0 — V1 Impactante

- Substituído MVP/release parcial por primeira entrega pública completa e impactante.
- Documento normativo `V1_IMPACTANTE.md` com capacidades, critérios de impacto, seis frentes paralelas, demo e gate de release.
- Alinhados README, SPEC, plano, execução, agentes, benchmark, custo e matriz.
- Testes T-075..T-082 especificados, não executados.
- Mantidas políticas v1.3 de segurança, paridade, workspace e licença.
