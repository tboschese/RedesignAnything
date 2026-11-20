# Redesign Anything — Plano executável de implementação (backlog detalhado v1.1; ordem substituída em v1.2)

> **DECISÃO v1.4:** `V1_IMPACTANTE.md` define o escopo obrigatório da primeira entrega pública. `EXECUTION_V12.md` organiza checkpoints internos; PRs não são releases. Checks essenciais continuam antecipados, mas todas as frentes devem convergir antes da publicação. Leia `BENCHMARK_PROTOCOL.md`.

> **Leia primeiro:** `SPEC.md`, `CONTRACTS.md`, `AGENTS_REDESIGN.md`. Este plano descreve PRs para um **fork independente** do Craft, não é uma solicitação de upstream. Nenhum PR/branch foi criado nesta análise. **Os caminhos de arquivos existentes foram conferidos na documentação/código; os novos são propostas e exigem inspeção do checkout.**

## 0. Estratégia e sequência

**Objetivo do primeiro ciclo:** em uma URL fixture, demonstrar `Study → Plan → Build → Check → Ready` do Craft com redesign React/TS e DS real; cobertura de jornada conhecida e relatórios de limites. **Não** começar com REA ou múltiplos MCPs antes de provar esse núcleo.

**Controle de consumo P0:** desde PR-00 estabelecer `host_agent` versus `craft_runner`, `lean` default e nenhuma invocação aninhada de builder. PR-02 incorpora campos de `execution` no contrato; PR-03/04 aplicam recuperação seletiva; PR-08 cria gates e medidas. Referência obrigatória: `COST_AND_EXECUTION.md`.

**Ordem para chegar a valor:** PR-00 → PR-01 → PR-02 → PR-03 → PR-04 → PR-05 → PR-06 → PR-07 → PR-08. Integrações avançadas vêm depois. Cada PR pequeno com teste, docs, mudanças de schema versionadas e regressão do Craft.

### Preparação local (comandos de referência, validar no checkout)

```bash
git clone https://github.com/storytold/craft-anything redesign-anything
cd redesign-anything
git remote rename origin upstream
# Criar seu fork remoto pelo GitHub e adicioná-lo; NÃO fazer push para upstream.
git checkout -b feature/redesign-bootstrap
rustc --version
cargo --version
cargo test --workspace
cargo xtask layers
cargo xtask secrets
```

O README consultado exige Rust 1.97+ e dependências específicas por SO. **Não considerar baseline verde até executar esses comandos e registrar plataforma/commit.** Leia `AGENTS.md`, `docs/pitfalls.md`, `docs/extending.md` e `docs/ROADMAP.md` antes de editar. O `AGENTS.md` original afirma que apenas itens do roadmap podem ser desenvolvidos e que não se deve nomear outros produtos de engenharia reversa em código; em um fork independente, **atualizar conscientemente essas regras e o roadmap** para o novo produto, respeitando créditos, licenças e princípios de segurança. Não usar o fork como contribuição upstream sem seguir as políticas upstream.

## PR-00 — Fundação, baseline e uso agent-first por URL do repo

**Arquivos:** `AGENTS.md`, `docs/ROADMAP.md`, `docs/STATUS.md`, `README.md`, `THIRD_PARTY_NOTICES.md`, documentação de arquitetura; CI existente. Sem modificar engine.

**Tarefas:** definir teste baseline de tokens/rodadas no mesmo agente (ou `unavailable` se não mensurável), explicitar dois modos de controlador, padrão `lean` e regras anti-loop; criar fork, marcar commit upstream, definir nomenclatura/finalidade independente e attribution; registrar versão Rust/OS, passar testes; criar fixtures originais; definir código de conduta e distribuição; preservar legados de Craft. Instalar hook de secrets conforme AGENTS do Craft. **Documentar fluxo agent-first desde a URL de GitHub**, quickstart sem UI, AGENTS.md próprio, `doctor`/pré-requisitos e segurança de setup; README com prompts para agentes. O repositório deve funcionar como entrada para Cursor/Claude/Codex sem exigir a aplicação desktop.

**Aceite:** baseline compila; suíte upstream verde ou falhas documentadas; nenhuma mudança silenciosa no Craft original; permissões/licenças e origem documentadas. Guia agent-first permite a um agente descobrir dependências sem executar installs silenciosamente. **Não rebrandear tudo neste PR**.

## PR-01 — `RedesignKind` no mecanismo real de Build

**Inspecionar:** `crates/ai/src/rebuild/kinds/mod.rs`, `website.rs`, `crates/ai/src/rebuild/mod.rs`, `crates/ai/src/rebuild/brief.rs`, `apps/craft-anything-cli` e registries.

**Implementar:** dispatcher `host_agent`/`craft_runner` e bloqueio `controller_conflict` para `rebuild_start` implícito; novo `TargetKind` com id `redesign`, seleção **explícita** por configuração (não apenas palavras ambíguas), default React/TS, semelhança de pixels não é gate de redesign. Integrar no `KindRegistry`; criar `RedesignBrief` mínimo que instrua preservação de capacidades conhecidas e exiba `unknowns`. Criar comando headless mínimo que crie projeto e liste etapas.

**Testes:** detecção explícita, projeto antigo continua `website`/`app`, `Study` continua igual, `kind=redesign` selecionado explicitamente, sem clone-visual checker no gate. Cobertura de erros da configuração.

**Aceite:** novo projeto chega a Study e Build com o runner do Craft sem alterar o comportamento do kind `website` existente. **Sem UI desktop:** comando CLI/fluxo MCP pode criar, inspecionar, continuar e listar status do projeto. Teste smoke no agente com checkout limpo.

## PR-02 — Workspace versionado e contratos

**Arquivos propostos:** `crates/engine/src/tools/redesign_*` ou módulo equivalente, owner de persistência na camada apropriada, `schemas/*.json` do kit (sincronizados ao schema Rust), testes de migração e I/O.

**Tarefas:** incorporar `execution`/`budgets` ao schema, contadores duráveis de chamadas no gateway, relatórios de tokens `reported/estimated/unavailable`, migração; criar diretório `redesign/`, arquivos versionados, `EvidenceRecord`, `Capability`, `Coverage`, `Decision`, `TransformationPlan`, `CheckResult`. IDs estáveis, migrações, backup e escrita atômica. Validar inputs hostis, links simbólicos, diretórios transversais e arquivos grandes.

**Testes:** roundtrip, schema inválido, v0 ausente, migração, unknowns preservados, ausência de segredos, symlink traversal recusado, restart mantém estado.

**Aceite:** criar/abrir projeto Redesign com dados tipados sem quebrar projeto Craft pré-existente; checks de camadas/wasm quando aplicável.

## PR-03 — Study adapter / capability mapper / coverage

**Reusar:** `web_site_capture`, `web_page_states_capture`, `web_walkthrough_record/check`, `core_evidence_*` e `investigation.cany`. Não criar novo browser crawler sem comprovar lacuna.

**Tarefas:** limitar payload de evidências e paginar antes da resposta ao agente, registrar custo marginal do Study; converter evidências Craft para modelo semântico; mapear rotas, ações, campos, estados e regras observáveis; mostrar ausência de perfis e lacunas de cobertura. Adicionar `redesign_capabilities_list` e `redesign_coverage_show` com paginação. Rastrear `sourceRefs` por capacidade.

**Testes:** fixture com regra visível e outra propositalmente oculta; regra oculta permanece `unknown`, capacidade conhecida preserva origem; sem `100%` quando denominador desconhecido. Login manual se necessário, sem cookies exportados.

**Aceite:** relatório Study reproduzível de uma jornada, incluindo gaps, sem alegar descoberta completa.

## PR-04 — UX Knowledge Packs + recuperação + Critic

**Tarefas:** formato de pack versionado e licenciado, retrieval local orientado a tarefa (tags + texto; 3–8 regras iniciais como heurística), sem injetar pack completo; catálogo mínimo inicial de tabelas, filtros, formulários, navegação, acessibilidade e densidade; aplicar contraindicações. Design Critic gera recomendações referenciadas; `keep_as_is` obrigatório.

**Testes:** pack sem licença/versão rejeitado; regra incompatível não aplicada; tendência a wizard/card não é invariável; prompt injection textual em pack não modifica ferramentas/autorização; fallback sem embeddings.

**Aceite:** relatório explica 3 opções (incluindo preservar) para uma tela densa, com trade-offs e fontes; sem requisitos inventados.

## PR-05 — Design System Registry strict + Layout CLI opcional

**Dependências:** modelos/schemas PR-02; critérios PR-08 podem começar como testes unitários de provider.

**Tarefas:** ingestão package.json/exports de pacote React/TS fixture; tokens DTCG; Registry com paths, versão, props, variantes e restrições; integração Layout CLI MCP por adapter; compliance próprio usando imports/tokens e typecheck, nunca confiar somente no MCP. Configurar `strict/advisory` e overrides aprovados.

**Testes:** import de componente real passa; import falso falha; prop inexistente falha; hardcoded proibido em strict falha; versão de pacote incompatível falha; Storybook ausente não bloqueia repo import válido; provider Layout fora do ar degrada sem mascarar requisito strict.

**Aceite:** gerar com DS fixture sem componentes duplicados e passar checagem de contrato real.

## PR-06 — Design Decision + Feasibility + Traceability

**Tarefas:** `diagnose/retrieve/alternatives/choose/plan`; classificação `ui_only`, `frontend_orchestration`, `backend_change_required`, `not_determined`; impacto e preservação; decisões aprováveis no Inbox nativo do Craft. Exportar plano de transformação tipado.

**Testes:** combinação de 3 telas sem endpoint atômico gera dependência de backend; cenário com backend existente admite orchestration com riscos; decisão divergente exige aprovação; capacidade crítica sem destino bloqueia check.

**Aceite:** relatório planeja nova jornada sem inventar endpoint/atomicidade e com traceability completa do escopo conhecido.

## PR-07 — Builder funcional com mock seguro

**Tarefas:** não iniciar runner aninhado no modo `host_agent`; criar brief compacto em `brief_sections` de `RedesignKind`, passar decisões/DS/capacidades por referência; construir React/TS+Vite com mocks sintéticos; preservar route/action traceability e estados. Reusar builder de Craft; não copiar design system nem logotipo privado.

**Testes:** build React/TS, eventos UI, modal, form validation, tabela, estados; CSP, secrets, rotas; projeto de origem intacto; recusa se strict DS não existe; agente sem capacidade suficiente para concluir retorna falha clara.

**Aceite:** protótipo funcional no `app/`, com dados marcados como simulados e sem chamadas de escrita à origem.

## PR-08 — Native Redesign Checks + Ready Gates

**Crítico:** não depender de `PluginJudge` WASM upstream para checks bloqueantes, pois falha do plugin pode ser `passed=true`. Criar checks Rust nativos registrados no `TargetKind` ou corrigir semântica sob teste.

**Tarefas:** build/typecheck, DS, Playwright scenarios, axe-core, traceability, segurança, impact e Criticality. Adicionar gates hard para chamadas/REA/critic e relatórios que não equiparem `unavailable` a zero tokens. Estados `pass/fail/warn/not_evaluated/blocked`; gates dos três Ready levels. Paginador de resultados; UX Critic separado.

**Testes:** check que retorna erro/timeout **não passa**; screenshot bonito mas botão não funciona → falha; caso com zero oráculos não vira `prototype_ready`; failing accessibility critical impede qualificação, mas não declarar certificação WCAG após aprovação.

**Aceite:** fixture com regressão detecta, corrige e revalida; código com fonte secreta não é exportado; relator explica o que não foi testado.

## PR-09 — Application MCP Bridge + Trust Gateway

**Reusar:** plugin MCP do Craft (stdio/HTTP, consentimento e Evidence). **Não** criar segundo cliente MCP indiscriminadamente.

**Tarefas:** perfil `application_mcp`, catalogue/hash de tool schemas, allowlist, classificação de efeitos, query `read-only`, versão e permissões; AKC opcional; confronto de informações e conflitos; aprovação para tool alterado; limites de saída/egress.

**Testes:** server com tool mentindo readOnly, tool description maliciosa, mudança de schema e token no body; todos bloqueados/sanitizados; `get_rule` seguro vira `declared`, não `observed`; sem MCP a experiência funciona.

**Aceite:** usuário conecta um MCP fixture e recebe explicação funcional contextualizada sem permissões adicionais implícitas.

## PR-10 — REA Deep Adapter + medição de valor

**Tarefas:** discovery/versão do Node/REA, iniciar apenas com consentimento, gatilho e orçamento; registrar razão de cada chamada e ganho de evidência incremental; converter Evidence, limitações e cobertura; `analyze_web_bundle`, `trace_javascript_semantics`, `inspect_web_network_capture` em primeiro lote; gated Deep. `environment-build`/stateful mock só com prova incremental e sanitização.

**Testes:** Deep permitido, Clean Room bloqueado, browser perfil dedicado, REA ausente fallback, timeout parcial, fonte incompatível, egress e token-canário; compare Craft vs Craft+REA no benchmark.

**Aceite:** integração adiciona pelo menos um fato correto e testado num fixture onde Craft-alone não o obtinha nas mesmas condições, ou continua experimental/opt-in sem impedir o checkpoint interno; porém a capacidade de investigação avançada precisa estar disponível e testada antes do release V1.

## PR-11 — Benchmark, segurança e grandes fluxos

**Tarefas:** benchmark controlado Craft base vs Redesign Lean vs Redesign Deep, medindo modelos/controladores, tokens input/output/cache (se reportados), chamadas e tamanho de payload, tempo e qualidade, inclusive caso sem telemetria; fixtures 2 perfis, múltiplas rotas, estados, docs incongruentes, API assíncrona, DS quebrado, prompt injection, locale e mobile. Suíte antes/depois, medidas de cobertura/precisão/tempo/custo.

**Testes:** `VALIDATION_MATRIX.md`; regressões repetíveis; nem fallback de agente, nem trace de REA podem causar claims mais fortes sem novos testes.

**Aceite:** resultados publicados com ambiente, versão, campos sem cobertura e limites; nenhum valor de KPI inventado.

## PR-12 — UX final, empacotamento e release

**Tarefas:** telas de seleção Redesign, DS e MCP sem novo canvas; relatório no workspace; branding e licenças; instaladores cross-platform com sidecar optional; SBOM; docs PT-BR/EN iniciais; teste de upgrade e rollback.

**Aceite:** instalação limpa em SO-alvo, first-run útil, uso CLI offline para fixture local, preview e relatório abrindo; licença e segurança documentadas.

---

## Definição de Done geral

- Rodar os comandos definidos no `AGENTS.md` real (incluindo `cargo fmt --all -- --check`, `cargo test --workspace`, clippy, `cargo xtask layers`, `cargo xtask locks`, `cargo xtask secrets`, wasm se tocar L0–L2, checks de i18n se pertinente).
- Evidência de execução real na PR, com versões de SO/toolchain/modelo e artefatos reproduzíveis.
- Provar fluxo antigo Craft sem regressão; testes unitários, contrato, negativos de segurança e pipeline P0.
- Não deixar ação/timeout/segredo em `pass`; unknowns não omitidos; arquivos novos versionados/migráveis.
- Lint/build do React gerado e testes E2E com fixture conhecida.

## Primeiro prompt para o Cursor

Copie este comando **depois de criar o fork e adicionar os arquivos deste pacote ao repositório**:

```text
Você está implementando Redesign Anything como fork de Craft Anything. Leia primeiro AGENTS.md, docs/pitfalls.md, docs/extending.md, REDESIGN_SPEC/SPEC.md, REDESIGN_SPEC/CONTRACTS.md, REDESIGN_SPEC/IMPLEMENTATION_PLAN.md, REDESIGN_SPEC/VALIDATION_MATRIX.md e o código real de crates/ai/src/rebuild/kinds. NÃO reescreva o Craft, NÃO integre REA/Studio agora, NÃO mude a UI antes de existir CLI. Defina `host_agent` como controlador desta sessão, `lean` como padrão e proíba builders aninhados. Não use um limite estimado de tokens como se fosse hard cap. Execute somente PR-00 (baseline e regras do fork) e PR-01 (novo RedesignKind mínimo com seleção explícita). Preserve Clean Room, consentimento, camadas, nunca-crash, tool/evidence contracts, código original e todos os testes upstream. Antes de codar, descreva as alterações precisas e o teste de aceite; depois implemente, rode verificações e reporte os resultados reais e limites. Não faça push, não edite repositório remoto e não declare testes não executados.
```

## Complemento v1.3.0 — tarefas bloqueantes do Council II

**PR-00:** adicionar `TRUST_BOUNDARIES.md`, `ORACLE_POLICY.md`, `SAFE_WORKSPACE_POLICY.md`, `OSS_DX_AND_UPSTREAM.md`, pin upstream SHA, licença/attribution, matriz de compatibilidade planejada, `doctor` sem side effects.

**PR-01 / primeiro vertical slice:** saída isolada; aprovação antes de aplicar patch; política de oráculo independente; cenário de fixture com expected observado/contratado; checks falham fechados; sanitização de evidências.

**PR-02:** schema para `oracle_type`, `approval_record`, `project_scope`, `evidence_retention`, `source_hash`, `workspace_output_mode`; implementar migrations e testes negativos. Não declarar campos implementados até integração Rust real.

**PR-03–08:** Task Model com unknowns, design de conteúdo, benchmark pré-registrado e avaliação cega, DS pinado, accessibility manual em amostra.

**PR-09–12:** REA/MCP somente opt-in e com controles; CI upstream, multiagente e multissistema; política de distribuição de ativos corporativos.

**Gate de revisão:** PR que altera fontes existentes, executa scripts de terceiros ou cria expected result por inferência não pode passar sem os controles correspondentes.
