# Redesign Anything — Pacote de especificação (v1.4.0)

**Status:** pacote embutido neste fork (`tboschese/RedesignAnything`); Slice 0 bootstrap em andamento; capacidades Redesign ainda não implementadas no engine. Base: fork do Craft Anything, gratuito/open source, local-first, sem API paga obrigatória. Foco inicial: qualquer aplicação web.

## Comece por aqui

1. `V1_IMPACTANTE.md`: **estratégia vinculante da primeira entrega pública completa**.
2. `EXECUTION_V12.md`: sequência de checkpoints **internos**, sem release público parcial.
2. `COUNCIL_DECISIONS.md`: decisões do council, riscos, hipóteses e condições.
3. `BENCHMARK_PROTOCOL.md`: comparação A/B/C e critérios de avaliação.
4. `COUNCIL_ROUND_2.md`: **segunda revisão adversarial, com segurança, QA, DX/OSS e domínio empresarial**.
5. `SPEC.md`: produto, arquitetura, requisitos, limites, segurança, outputs, gates, interfaces propostas.
5. `IMPLEMENTATION_PLAN.md`: PR-00 a PR-12, dependências, tarefas e critérios de aceite.
6. `CONTRACTS.md`: modelos semânticos e integrações; complementado por `schemas/`.
7. `COST_AND_EXECUTION.md`: **controle de agente único, perfis Lean/Deep, orçamento de tokens, mensuração, limites executáveis e benchmark comparativo**.
8. `AGENT_FIRST.md`: **como passar a URL do futuro repo ao Cursor/Claude/Codex e usar CLI/MCP sem desktop**; os comandos ali são PROPOSTOS.
9. `AGENTS_REDESIGN.md`: instruções para codificação no fork.
10. `VALIDATION_MATRIX.md`: testes de unidade, integração, segurança, E2E, benchmark, smoke tests agent-first.
11. `SOURCES.md`: repositórios e documentos consultados, licenças e riscos de redistribuição.
12. `CHANGELOG.md`: mudanças e compatibilidade entre versões da documentação.
13. `examples/` e `schemas/`: configuração Lean reproduzível e nove JSON Schemas versionados (documentação v1.2.0, envelopes de schema v1.0.0).

## Primeira entrega de código

Criar o fork independente, copiar este pacote para `REDESIGN_SPEC/`, ler `AGENTS.md`/`docs/extending.md`/`docs/pitfalls.md` upstream e executar os slices como checkpoints internos. **A V1 pública exige todas as capacidades e gates de `V1_IMPACTANTE.md`; nenhuma entrega parcial será apresentada como lançamento.** O `AGENT_FIRST.md` é obrigatório desde o P0.

## Uso após implementação (exemplo de intenção)

> “Use o Redesign Anything no GitHub para redesenhar `https://app.exemplo.com`, usando meu design system e preservando os fluxos conhecidos. Leia as instruções do repo, diagnostique dependências e solicite permissão antes de instalar ou acessar ambientes privados.”

Passar a URL do repositório no agente é um caminho de bootstrap, **não uma instalação automática garantida**.

## Adendo normativo v1.3.0

Leia também, **antes de implementar**:
- `TRUST_BOUNDARIES.md` — escopo real de controles e segurança do host.
- `ORACLE_POLICY.md` — expected results independentes, gates e estados inconclusivos.
- `SAFE_WORKSPACE_POLICY.md` — output isolado, diff e aprovação para patches.
- `OSS_DX_AND_UPSTREAM.md` — instalação assistida, design systems privados, upstream e acessibilidade.

O adendo v1.3.0 prevalece sobre a ordem e políticas antigas onde houver conflito. `EXECUTION_V12.md` continua sendo a sequência de slices, com condições adicionais v1.3.0. Nenhum requisito foi testado em software real.

## Estratégia de release v1.4.0

A primeira versão pública é a **V1 Impactante**, não um MVP. Leia `V1_IMPACTANTE.md` primeiro. A arquitetura avançada (REA, Application MCP, DS corporativo e jornadas) é requisito de disponibilidade da V1, embora cada integração seja usada seletivamente por tarefa.
