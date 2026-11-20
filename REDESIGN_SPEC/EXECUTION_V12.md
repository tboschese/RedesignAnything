> **v1.4.0 — interpretação obrigatória:** slices 0–5 são checkpoints internos de integração, não MVPs nem releases públicos. O release V1 exige o conjunto completo e os gates de `V1_IMPACTANTE.md`. Trabalhar em frentes paralelas é permitido; integrar e testar continuamente.

# Plano de execução priorizado — v1.2.0

> **Prevalece sobre a ordem histórica de PRs em `IMPLEMENTATION_PLAN.md`**. Os PR-00..PR-12 permanecem como backlog técnico, mas o primeiro ciclo é reorganizado para provar valor e checks antes de infraestrutura avançada.

## Slice 0 — Baseline e fork mínimo

- Fixar SHA upstream, licenças, requisitos Rust/Node e `cargo test --workspace`/smokes conforme suporte local.
- Instalação assistida por URL do repo; nenhum setup ou script de terceiros executado silenciosamente.
- Criar fixtures originais, uma tela densa e uma jornada com 2–3 passos, com regras e oráculos conhecidos.
- Estabelecer baselines A=Craft comum, B=Craft+instruções UX e C=REA; congelar rubrica e condições antes da execução.
- Aceite: ambiente reproduzível, baseline documentado, sem alegar melhoria.

## Slice 1 — Primeiro redesign ponta a ponta

- `RedesignKind` com integração CLI/MCP no host_agent, sem runner aninhado.
- Reutilizar Study do Craft; persistir Task Model mínimo e evidências/unknowns.
- Gerar diagnóstico e opções incluindo `keep_as_is`; construir React/TS.
- **Antecipar checks mínimos:** build/typecheck, smoke da tarefa crítica, referências de evidência, falha fechada em timeout, relatório Concept/Prototype Ready.
- Aceite: tarefa fixture funciona e não regrediu; logs sem segredos; `unknown` não vira `passed`.

## Slice 2 — Diferenciação de UX

- Knowledge Packs locais com regras/contraindicações e recuperação enxuta.
- Rastrear decisões e qualidade de conteúdo (rótulos, erros, estados).
- Executar benchmark A/B/C com revisão cega e métricas pré-registradas; contabilizar chamadas, duração e tokens quando disponíveis.
- Aceite: ganho demonstrável em pelo menos uma dimensão relevante sem regressão crítica; caso contrário, simplificar para skills/checks.

## Slice 3 — DS corporativo real

- Integrar um pacote React/TS de fixture com tokens e componentes.
- `required_component`, `allowed_composition`, `extension_requires_approval`.
- Validar imports/props/versão, tokens e semântica de estado.
- Aceite: build e checks positivos/negativos com política strict consistente.

## Slice 4 — Investigação profunda seletiva

- Primeiro medir lacunas do Study nativo. Só então testar REA ou Application MCP isoladamente.
- Registrar cobertura marginal, falsos positivos, latência, custo e risco.
- Aceite: integrar apenas se benefício incremental for reproduzível.

## Slice 5 — Robustez, desktop e release

- Regressão upstream, segurança adversarial, instalação multiplataforma, acessibilidade manual e governança OSS.
- Aceite: release reproduzível, docs agent-first, limitações explícitas, sem promessa de produção automática.

## Regra de mudança

Nenhum módulo novo entra no caminho crítico sem hipótese, teste, orçamento e fallback documentados. Cada slice exige relatório de riscos residuais. Não há datas ou percentuais inventados.

## Adendo obrigatório v1.3.0 — condições de entrada por slice

**Slice 0:** registrar `TRUST_BOUNDARIES.md`, `ORACLE_POLICY.md`, baseline de licenças/attribution, SHA upstream, `doctor` sem efeitos, fixture com oráculos independentes e plano de benchmark pré-registrado.

**Slice 1:** output isolado por padrão, proibir patch sem aprovação, oráculos com origem externa à geração, falha fechada em timeout, redaction antes de persistir, escopo de paths/origens e proibição de builder aninhado.

**Slice 2:** avaliação cega com amostragem e thresholds definidos antes da execução; não usar apenas LLM-as-judge; métricas de tarefas e resultados funcionais.

**Slice 3:** DS versionado por hash/lockfile; não redistribuir ativos privados; validar semântica de componentes além de import/props.

**Slice 4:** REA/MCP opt-in; evidência nova, permissões e política de dados comprovadas; nenhuma chamada mutante automática.

**Slice 5:** CI/regressão upstream, smoke real por agente/SO, teclado/foco/zoom/leitor de tela em amostra, retenção e purge testados.

**Regra de bloqueio:** nenhum slice pode declarar `Ready` se houver testes críticos sem execução ou oráculos apenas inferidos. O escopo e os desconhecidos permanecem visíveis.
