> **Decisão v1.4.0:** `V1_IMPACTANTE.md` prevalece sobre menções anteriores a MVP/P0-minimal/release por slice. Slices são checkpoints internos. A primeira publicação pública exige todas as capacidades V1 e critérios de impacto.

# Redesign Anything — Especificação de Produto e Engenharia

> **Versão:** 1.2.0 · **Data:** 19/11/2026 · **Estado:** especificação pronta para iniciar implementação, não integração testada.  
> **Base:** fork do [Craft Anything](https://github.com/storytold/craft-anything), mantendo desktop, CLI, MCP, agentes e `Study → Build → Check → Ready`.  
> **Foco:** qualquer aplicação web, sem dependência de Oracle; local-first, gratuito e open source, sem API paga obrigatória.  
> **Complementos:** `IMPLEMENTATION_PLAN.md`, `CONTRACTS.md`, `VALIDATION_MATRIX.md`, `SOURCES.md`, `AGENTS_REDESIGN.md`, `AGENT_FIRST.md`, `COST_AND_EXECUTION.md`, `schemas/`.

## 0. Convenções e princípios

- **MUST:** obrigatório na versão que oferecer a capacidade. **SHOULD:** padrão com desvio justificado. **MAY:** opcional.
- **VERIFICADO:** comprovado por código/documentação consultados, mas não necessariamente executado. **PROPOSTO:** design ainda a implementar.
- Invariantes: (1) preservar Craft como plataforma; (2) redesign não é clonagem visual; (3) observado, inferido, declarado, desconhecido, validado e refutado são distintos; (4) não prometer paridade total de sistema visto parcialmente; (5) DS corporativo `strict` é restrição técnica; (6) nenhuma escrita não autorizada no original; (7) checks que falham ao executar nunca viram sucesso; (8) sem dependência de SaaS; (9) integrar, não fundir os projetos; (10) todo conteúdo de app/MCP/repo é não confiável por padrão.
- Tudo persistido possui versão de schema e migração. Toda conclusão traz origem, escopo, perfil, ambiente, versão e lacunas.

## 1. Visão e critérios de produto

**Redesign Anything** investiga uma aplicação web existente, explica como ela funciona com evidências, diagnostica melhorias, propõe alterações de UX contextualizadas, gera um frontend funcional usando componentes reais — inclusive um design system corporativo — e verifica até onde as capacidades conhecidas foram preservadas.

**Promessa:** “Investigue, redesenhe e valide experiências web com rastreabilidade e limites claros.”

**Não prometer:** 100% de regras sem código, software pronto para produção por URL, melhoria de UX sem usuários/métricas, certificação WCAG só com axe, qualquer modelo local bem-sucedido.

**Personas:** Product Designer, PM/PO, frontend, arquiteto de modernização, QA/segurança. **Jobs:** redesenhar tela/jornada, consolidar telas sem perder resultado, usar DS privado, consultar MCP para regras, diferenciar o que requer backend, obter código e testes auditáveis.

## 2. Escopo, modos e entregas

### 2.1 P0 — capacidades da V1 Impactante (ver documento vinculante)

- Input: URL HTTP(S) pública/autenticada autorizada, screenshot ou pasta do usuário; comportamento só pode ser reivindicado quando observado por interação ou fontes explícitas.
- **Agent-first é P0:** produto deve ser operável por agente de código apenas com repositório, CLI/MCP e documentação, sem a UI desktop; caminho de adoção por URL de GitHub documentado, setup reexecutável e diagnóstico de ambiente. Não assumir que fornecer URL instala automaticamente ferramentas.
- **Execução econômica é P0:** modo `lean` default, orquestração única (host agent **ou** runner Craft), conhecimento/DS sob demanda, ferramentas Deep/REA opt-in, limites verificáveis de chamadas e telemetria honesta. Ver `COST_AND_EXECUTION.md`.
- Captura/estudo, evidência, jornadas, estados, regras inferidas e lacunas usando Craft; REA e MCP da aplicação opcionais.
- Modos: `refresh` (visual, mínima mudança funcional), `redesign` (reorganiza jornada com rastreabilidade), `reimagine` (propõe novos fluxos e sinaliza dependências de backend).
- Conhecimento de UX local e versionado; Design Decision Engine, Critic; Corporate DS Registry com `strict` e `advisory`.
- Geração React + TypeScript, Vite por default, mocks/dados sintéticos e componentes DS reais.
- Verificações objetivas: build, DS, jornadas conhecidas, acessibilidade automatizada, risco técnico/segurança e relatório de cobertura.
- UI praticamente igual à do Craft: uma opção Redesign, parâmetros progressivos, etapas e preview; sem exigir canvas próprio na V1.

### 2.2 P1 / P2

P1: importação Storybook/TypeScript mais profunda, REA semântico + tráfego, MCP da aplicação, múltiplos papéis, simulação com estado e análise de impacto, edição de projeto existente. P2: vídeo como input completo, comparador visual colaborativo, Figma opcional, outros frameworks, migração backend, colaboração remota.

**Fora do P0:** backend produtivo autogerado; alterações destrutivas de produção; clonagem de marcas/ativos sem direitos; descoberta garantida de lógica servidor-side não exposta; hospedagem SaaS; inclusão do Layout Studio AGPL no núcleo sem avaliação legal.

### 2.3 Níveis de Ready

- `concept_ready`: interface navegável com hipóteses, mocks e limitações explícitas; sem promessa de paridade.
- `prototype_ready`: build executável, cenários **críticos conhecidos do escopo** aprovados, DS válido e dados simulados claramente identificados.
- `engineering_ready`: verificações exigidas pelo escopo, contratos e testes satisfatórios; **pronto para revisão/integração, não certificado para produção**.
- `blocked`: erro de segurança, permissão ou requisito obrigatório. `needs_evidence`: ausência de evidência impede alegação solicitada, mas não necessariamente protótipo limitado.

## 3. Baseline do Craft: reaproveitamento obrigatório

O código/doc do Craft confirmam: Rust/egui + CLI/MCP; `TargetKind`, `Check` e `ToolSpec`; plugins MCP locais/remotos; WASM opcional; `Workspace` com `capture/`, `app/`, `checks/`, `investigation.cany/`, `inbox/`; browser capture, estados, walkthrough, JS e stand-in de tráfego; runners e revisões iterativas. **Não reimplementar essas capacidades.**

Código-base de extensão:

- `crates/ai/src/rebuild/kinds/`: `TargetKind`, `KindRegistry`, `Check`, `CheckCx`; novo tipo de projeto `redesign` [NOVO].
- `crates/ai/src/rebuild/brief.rs`: brief contextual, sem prompt monolítico [EXTENDER].
- `crates/ai/src/rebuild/mod.rs` e workflow `study/check`: orquestração existente [EXTENDER ATRAVÉS DOS SEAMS].
- `crates/engine/src/tools/<area>.rs` + `tools/mod.rs`: novos `ToolSpec` e registro para CLI/MCP [NOVO].
- `crates/ai/src/rebuild/stacks.toml`: biblioteca/stack se realmente necessária; reutilizar React/TS existente quando disponível.
- `crates/ui-egui/src/`: seleção Redesign, DS e relatórios, mantendo thin UI.
- `crates/engine/src/workspace*` ou owner equivalente: arquivos versionados, migrations e diretório de projeto.

**Cuidados confirmados:** (a) site check padrão examina start page e até 12 páginas vinculadas, não aplicação inteira; (b) no código de `plugin_check.rs`, uma falha de execução de plugin WASM é reportada como `passed=true`. Checks obrigatórios do Redesign DEVEM ser nativos/fail-closed ou exigir alteração explícita dessa semântica; (c) Clean Room do Craft não é sandbox de SO; (d) uso de ferramentas Deep exige consentimento; (e) `AGENTS.md` upstream impõe padrões de originalidade, Rust seguro, camadas e roadmap — adaptar ao fork antes de codar, preservando os controles de segurança.

## 4. Arquitetura lógica e responsabilidades

```mermaid
flowchart TD
 U[Craft UI / CLI / MCP] --> O[Redesign Orchestrator]
 O --> S[Craft Study]
 S --> X[REA Adapter opcional]
 S --> M[Application MCP Gateway opcional]
 X --> E[Unified Evidence + Coverage]
 M --> E
 S --> E
 E --> C[Capabilities & Experience Context]
 C --> D[Design Decision + Feasibility]
 K[UX Knowledge Packs] --> D
 R[Corporate Design System Registry] --> D
 D --> B[Craft Builder / React TS]
 R --> B
 B --> Q[Redesign Native Checks]
 Q --> G[Ready Gates + Report]
 Q -.feedback.-> B
```

**Craft:** lifecycle, browser, CLI/MCP, UI, agente, build runner, pause/resume. **Redesign:** modelos semânticos, decisão UX, DS, gating, rastreabilidade, instrumentos de confiança. **REA:** estudo Deep seletivo de JS/rotas/endpoints/capturas/ambientes. **Layout CLI:** fornece DS e checks auxiliares, não decide sozinho a conformidade. **App MCP:** declara informações funcionais conforme permissões; não é verdade absoluta.

**Um controlador por run:** em `host_agent`, Cursor/Claude/Codex já controla os raciocínios, edição e ferramentas; a integração MCP/CLI apenas fornece capacidades e **não inicia automaticamente** o builder interno do Craft. Em `craft_runner`, o runner do Craft é controlador único, e REA/Layout/app MCP são providers passivos. `controller_conflict` impede uma chamada de Build aninhada; handoff requer aprovação explícita e checkpoint. `lean` é default independente de `clean_room`; `deep` exige autorização separada. O roteamento e limites detalhados estão em `COST_AND_EXECUTION.md`.

**Processos:** núcleo Rust; sidecars Node somente quando ativados, subprocessos sempre via `craft_anything_platform::process` com deadlines, cancelamento, limites de output e isolamento; nenhum serviço cloud central obrigatório. Runtime Node compatível com REA selecionado e versões pinadas; Rust segue baseline do checkout (README consultado: 1.97+). Não adicionar banco servidor central obrigatório à V1. `model.use_craft_selected_backend` só se aplica a `craft_runner`; em `host_agent` deve ser `false` e o modelo do host é o único ativo.

**Modos de investigação:** `clean_room` usa somente superfície de visitante/operador; `deep` pode inspecionar JS, rede e estruturas internas com aprovação. Consultar MCP que revela regras internas deve ser consentimento separado e ter proveniência, não um bypass silencioso do modo Clean Room.

## 5. Persistência, contratos e modelo semântico

### 5.1 Organização do projeto [PROPOSTA]

Preservar os diretórios nativos do Craft. Adicionar apenas `redesign/`:

```text
<workspace>/
  craft.json                  # existente; adicionar campos versionados somente se necessário
  capture/                    # Craft, sem duplicar
  app/                        # projeto gerado ou projeto autorizado para edição
  checks/                     # Craft
  investigation.cany/         # Craft / evidências
  inbox/                      # Craft / pedidos de aprovação
  redesign/
    config.json               # política e contexto, sem credenciais
    study-snapshot.json       # target/ambiente/perfil/versão/scope
    capabilities.json         # capacidades conhecidas e unknowns
    rules.json                # regras observadas/declaradas/inferidas
    evidence-index.json       # refs para provas no Craft e em adaptadores
    coverage.json             # denominadores explícitos
    ds/registry.json          # componentes/tokens/contratos/imports aprovados
    knowledge-lock.json       # packs, versões, licenças e hashes
    decisions.json            # tradeoffs e justificativas
    transformation-plan.json  # origem -> redesign + viabilidade + testes
    traceability.json         # requisitos/capacidades -> artefatos -> checks
    scenarios.json            # scripts e oráculos de testes
    check-results.json        # resultados determinísticos e do Critic separados
    usage.json                # snapshots versionados de consumo (nullable onde não observável)
    report.md                 # relatório de engenharia
    private/                  # APENAS artefatos locais sensíveis, gitignored
```

Cada artefato possui `schema_version`, `project_id`, `revision`, `generated_at`, `source_refs` e limites de tamanho. Escrita atômica, `tmp → fsync → rename`, migração e falha explicável. Caminhos relativos são restritos à raiz do workspace. `redesign/private/` não é exportado por padrão; não armazenar credenciais lá. Artefatos individuais podem usar JSONL paginado quando volumosos, mas sem expor milhões de linhas no contexto do agente.

### 5.2 Entidades principais

Os contratos completos estão em `CONTRACTS.md` e `schemas/`.

- **EvidenceRecord:** id estável, origem (`craft`, `rea`, `app_mcp`, `user`, `source_repo`, `test`), tipo, localização, identidade da captura, digest, escopo, classe de dado, estado (`observed`, `declared`, `inferred`, `unknown`, `validated`, `refuted`), completude, timestamp.
- **Capability:** objetivo/tarefa; ator/perfil; pré-condições; inputs; ações; resultados; estados; telas originais; evidence_ids; criticidade; teste/oráculo; status de cobertura.
- **BusinessRule:** texto, expressão formal opcional, condições e transições, origem, status, conflitos e restrições de aplicação.
- **ExperienceContext:** atores, frequência e criticidade das tarefas, volume de dados, restrições operacionais, métricas existentes; `unknown` quando não informado.
- **DesignSystemRegistry:** componentes reais com pacote/import path, versão, props e variantes, estados, exemplos, regras de uso, tokens e proveniência.
- **DesignDecision:** problema, alternativas incluindo `keep_as_is`, evidências, regras UX utilizadas, escolha, riscos, impacto, critérios de teste e confirmação humana.
- **TransformationPlan:** capacidades originais → rotas/componentes/ações geradas, tipo de viabilidade, dependências de backend, preservações obrigatórias, check IDs.
- **ScenarioOracle:** ação/entrada/estado anterior/resultado esperado, provedores de evidência e restrições da simulação.
- **CheckResult:** `pass`, `fail`, `warn`, `blocked`, `not_evaluated`; severidade, execução, logs sanitizados, evidence_ids, gate aplicável.

**Proibição:** gravar no mesmo campo uma observação de comportamento e uma conclusão de LLM sem origem explícita. Não usar confiança numérica autoatribuída como prova.

### 5.3 Confidence & Coverage Engine

- Denominadores necessários: `known_capabilities`, `scoped_capabilities`, `scenarios_with_oracle`, `scenarios_executed`, `scenarios_passed`, `roles_examined`, `states_examined`, `routes_examined`.
- `tested_coverage = passed_known_scenarios / scenarios_with_oracle`, nunca “% da aplicação”. Se denominador zero → `unknown`, não `100%`.
- `discovery_coverage` = capacidades enumeradas no escopo + estados/jornadas inspecionados; denominador precisa ser declarado, não extrapolado.
- Confiança qualitativa: `observed`, `inferred`, `declared`, `corroborated`, `contradicted`, `unknown`; combinar fontes independentes sem apagar conflitos.
- Quando Craft/REA/MCP discordarem, criar `conflict` ligado às versões/perfis e abrir questão. A fonte MCP não vence automaticamente a observação e vice-versa.
- Regras não verificadas devem estar visíveis nos relatórios e nos briefings do builder, sem afirmar que são invariantes comprovados.

## 6. Corporate Design System Registry

### 6.1 Entradas aceitas

Prioridade: (a) repositório/pacote React real com permissões; (b) Storybook; (c) tokens DTCG/CSS e documentação; (d) Layout CLI e importação de kit; (e) UI kit open source se não houver corporativo. Figma é provider opcional, não pré-requisito.

### 6.2 Compilação do Registry

1. Localizar `package.json`, lockfile, exports, componentes e versão; nunca assumir o nome da API pela aparência.
2. Extrair assinaturas/props/variantes quando possível por TypeScript AST ou metadados do Storybook, sem executar código desconhecido.
3. Ler tokens DTCG 2025.10 ou converter fontes suportadas; registrar tema, densidade, contraste e aliases.
4. Ingerir documentação com fonte, versão, licença e `usage`, `never`, `a11y`, `patterns`.
5. Resolver componentes permitidos via `ComponentId`, `importPath`, `packageVersion`, `propsSchema` e `evidenceRefs`.
6. Produzir `registry.json` assinado por digest de conteúdo; mudanças invalidam a verificação da versão anterior.
7. Expor via provider interno `redesign_ds_list_components`, `redesign_ds_get_component`, `redesign_ds_get_tokens`.

### 6.3 Política de aplicação

- `strict`: componentes permitidos **somente**; componentes inexistentes → `blocked` ou solicitação de extensão aprovada. Não criar HTML genérico visualmente parecido com componente corporativo quando existe componente oficial. Imports, props, eventos, tokens, tema e versão conferidos por análise de código e execução de build/testes.
- `advisory`: sugestões, com exceções explicitamente registradas e justificadas.
- Hierarquia: integridade, segurança e acessibilidade obrigatórias > políticas corporativas > objetivo de produto > padrões genéricos. Conflitos não solucionáveis exigem aprovação humana.
- `Layout check_compliance` é evidência auxiliar, **não** certificado. Nosso checker adicional identifica importações proibidas, tokens/cores hardcoded, props incompatíveis, componentes duplicados e versões erradas.
- Testes de componente em contexto: estados vazio, loading, erro, disabled, foco/teclado, densidade de dados e responsividade.

### 6.4 Exemplo

Em uma tarefa de listagem com filtros, o agente recebe o contexto de que `@corp/ui` contém `DataTable`, `FilterBar` e `BulkActions`, com import paths e props reais. Deve reutilizá-los, não criar `MyTable` usando a mesma aparência. Se a versão corporativa não suportar seleção em massa, registrar limitação e propor alternativa; não inventar uma propriedade inexistente.

## 7. Design Intelligence Engine

### 7.1 Knowledge Packs

- Fonte primária: bases abertas e licenciadas (UI/UX Pro Max, PatternFly/Carbon conforme licença específica, Open UI, DTCG, normas WCAG públicas); **não copiar integralmente materiais proprietários**.
- Pack versionado com `id`, `version`, `license`, `source_url`, `retrieved_at`, `hash`, `domains`, `tasks`, `applicability`, `contraindications`, `examples`, `tests`, `reliability`.
- Estrutura: `ux-core`, `enterprise-data`, `forms-validation`, `navigation-ia`, `dashboards`, `accessibility`, `density-productivity`, `mobile-responsive`, `content-localization`, `industry-patterns`.
- Seleção de conhecimento: regras/tags e busca textual local (SQLite FTS5 ou equivalente) como baseline; embeddings são opcionais e não dependência da V1. Contexto pequeno e direcionado.
- Regra de ouro: nenhum pack converte uma heurística em obrigação legal; o relatório distingue padrão, restrição corporativa e hipótese.

### 7.2 Design Decision Engine

Pipeline tipado: `diagnose` → `retrieve_rules` → `generate_alternatives` → `evaluate_tradeoffs` → `check_feasibility` → `select_or_ask` → `record_decision` → `build_guide`.

A avaliação considera: tarefa, perfil e experiência, frequência, densidade, custo cognitivo, risco de erro, interoperabilidade, acessibilidade, componente disponível, efeitos técnicos e métricas fornecidas. Gera pelo menos a opção `keep_as_is` e, quando solicitado, alternativas contrastantes. Não forçar wizard, cards ou menu lateral como template universal.

### 7.3 Design Critic

- Agente ou papel separado do gerador, com inputs: evidências, transformação, DS, screenshots do novo, testes executados.
- Crítica estrutural sobre tarefas, hierarquia, densidade, atalhos, estados, erros e navegação; cada finding contém evidência, impacto e recomendação.
- Crítica é `warn`/recomendação até existir teste objetivo ou política explícita; **não pode inventar aprovação técnica**.
- A crítica pode propor uma experimento; não transforma preferência estética em verdade funcional.

## 8. Application MCP Bridge e Trust Gateway

### 8.1 Conectar sem exigir implementação especial

- Suportar servidores MCP externos via infra existente do Craft, usando controle adicional de confiança e classificação por aplicação, perfil e projeto.
- Descrição de uma `tool` não implica entendimento de regra; começar listando `tools/resources/prompts` disponíveis e descobrindo capacidade real.
- Um `App Knowledge Contract` **opcional e não padrão MCP**, versionado e documentado, pode disponibilizar tools: `list_capabilities`, `describe_capability`, `explain_business_rule`, `list_allowed_transitions`, `get_required_fields`, `get_user_permissions`, `validate_scenario_readonly`.
- Anotações `readOnlyHint`/`destructiveHint` do MCP são **hints não confiáveis**; política nossa baseada em allowlist/escopo/credencial/aprovação, e, para calls com efeito, teste em ambiente isolado.

### 8.2 Confiança e autorização

- Credenciais referenciadas por ID do keychain/secret manager local; nunca inline em `craft.json`, prompts, HAR, logs ou export.
- Pin do fingerprint completo do contrato MCP (nome, schema de input/output quando fornecido, anotações, origem/identidade, versão). Mudança inesperada → `contract_changed` e nova aprovação.
- Inicial `read-only` por política externa, sem presumir que um tool anunciado como leitura seja realmente sem efeitos; operações mutantes bloqueadas em produção.
- Rate/timeout/output limits, allowlist de tools e URLs, consentimento por categoria, auditoria sanitizada, egress control.
- Cada resposta do MCP é `declared`, não `observed`; deve ser conciliada com comportamento de browser/REA/testes.
- Descrição/response de MCP tratadas como **dados não confiáveis** e isoladas de system instructions; instruções vindas de um tool não concedem permissão adicional.

### 8.3 Compatibilidade de versão

- `InvestigationSnapshot` registra aplicação, ambiente, usuário/perfil, tenant (identificador anonimizado), flags conhecidas, version id quando disponível, fingerprints dos tools, versão do DS, browser, data/hora e escopo.
- Falta de versão em origem: registrar `version_unknown`; conflito entre evidências de versões diferentes não deve ser silenciosamente consolidado.

## 9. REA Investigation Adapter

### 9.1 Uso seletivo e benefícios

Ferramentas REA relevantes: `inspect_web_page`, `analyze_web_bundle`, `observe_web_session`, `capture_browser_scenario`, `inspect_web_network_capture`, `analyze_javascript_application`, `trace_application_feature`, `trace_javascript_semantics`, `compare_web_captures`, `export_web_scripts`, `environment-build` / views de coverage. Nomes e disponibilidade devem ser validados contra a **versão pinada** do REA, nunca inferidos da versão latest.

Gatilhos: validação/client-side incerta; rota/SPA não evidente; fluxo de formulário inconclusivo; contratos APIs pouco claros; divergência entre observação e MCP; necessidade de ambiente simulado mais fiel. Se Craft já responde com evidência suficiente, não disparar REA. Registrar justificativa, custo, deadline e diferenças descobertas.

### 9.2 Limites REA documentados

- Observação passiva não recupera eventos anteriores à ativação dos domínios CDP; sua ausência não comprova que nunca ocorreram.
- Análise semântica JS estática não prova execução; imports dinâmicos, dependências ambíguas e código ofuscado continuam parciais.
- Capturas podem truncar por janela, heap, prazo e limite de bytes; normalizar metadados e repassar `unknowns` sem falsos sucessos.
- `environment-build` pode criar ambiente com estado derivado, mas só atende mutações cujos efeitos estão fundamentados pelas capturas; writes não modeladas são `withheld`.
- CDP dá acesso poderoso ao perfil; usar perfil dedicado, portas locais, origens controladas, consentimento e isolamento. Não anexar simultaneamente várias ferramentas ao mesmo browser sem coordenador de sessão.

### 9.3 REA como provider de evidências

Adaptador transforma `Evidence` externa em `EvidenceRecord` com `external_evidence_id`, `producer_version`, `capture_digest`, `coverage`, `limitations`, `authority`, `evidence_ref`. Preservar bruto apenas quando explicitamente permitido; exibir projeções/sumarização para agente.

### 9.4 Ambiente simulado

Política escalonada: (1) mocks sintéticos controlados; (2) stand-in do Craft quando registros suficientes; (3) REA Agent Environments se sua cobertura de reads/writes confirmadas aumentar a fidelidade. Nunca usar replay de credenciais/reais como fixture exportável. Se não houver provas de efeitos de escrita, marcar `not_modeled` e permitir apenas demonstração isolada.

## 10. Feasibility & Transaction Semantics

Antes de construir, cada mudança recebe classificação:

- `ui_only`: aparência/navegação/apresentação sem novos efeitos de backend.
- `frontend_orchestration`: novas sequências de calls existentes; exige teste de estado, idempotência, erros parciais, retries, race conditions e permissões.
- `backend_change_required`: requer novo endpoint, regra, transação, autorização ou capacidade não exposta; produzir proposta e contrato pendente, não fingir implementação completa.
- `not_determined`: informações insuficientes; só protótipo com hipótese rotulada ou bloqueio de engenharia.

Para fluxos complexos, registrar operações, dependências, pré-condições, efeitos, garantias de atomicidade **conhecidas**, idempotência, compensação/rollback e eventos assíncronos. Não inferir atomicidade olhando apenas várias telas. Em integração live, requests mutantes só para staging/teste com credenciais restritas e permissão.

## 11. Builder, gerador e proteção de código

- Usar executor dos agentes do Craft. O Brief contém contexto compacto e versões: capability map, UX plan, registry DS, invariantes, desconhecidos, critérios executáveis. Limitar contexto e recuperar referências por demanda; não enviar a biblioteca/Storybook inteiro ao prompt. Em `host_agent` não iniciar novo modelo automaticamente.
- React + TypeScript por default. Se o DS exigir uma estrutura suportada diferente, agente deve justificar ou registrar incompatibilidade. Em projeto existente, usar branch/worktree e staging autorizado, nunca alterar fonte sem confirmação.
- Estrutura sugerida do frontend gerado: `src/app`, `src/routes`, `src/features/<capability>`, `src/components` (somente se aprovado), `src/lib/api`, `src/lib/state`, `src/mocks`, `tests/e2e`.
- Componentes corporativos importados por caminhos reais. Proibido copiar seu código sem direito ou replicar sua aparência em wrappers genéricos para satisfazer compliance falso.
- Simulação: dados sintéticos e banner de ambiente; desabilitar endpoints produtivos. `src/mocks` nunca inclui dados pessoais extraídos.
- Gerar estados: loading, empty, invalid, error, success, disabled, permission-denied e timeout nos fluxos aplicáveis. Não inventar regras; desconhecidos visíveis em config/teste.
- Toda decisão relevante registra onde foi implementada e quais testes a cobrem.
- `Design Critic` separado do `Build Check`: opinião heurística não faz build passar.
- Tratar código gerado como não confiável: comandos de pacote/build em ambiente controlado, sem rede/segredos por default, scripts de instalação auditados, limites de processo.

## 12. Validação e gates nativos de redesign

### 12.1 Checks implementados no tipo `redesign`

| Check ID | Tipo | Condição de bloqueio |
|---|---|---|
| `redesign_build_check` | Determinístico | build/typecheck obrigatório falhou ou não executou |
| `redesign_ds_check` | Determinístico | `strict`: import/token/props/componente não autorizado |
| `redesign_functional_check` | Browser/testes | cenários críticos com oráculo não passam; execução faltante = not_evaluated |
| `redesign_accessibility_check` | Automático + revisão | novas violações críticas identificadas; ausência de revisão humana permanece qualificada |
| `redesign_security_check` | Política e artefatos | segredo exportado, egress não autorizado, uso de ação mutante proibida |
| `redesign_traceability_check` | Estrutural | capacidade crítica conhecida sem destino ou justificativa |
| `redesign_impact_check` | Diferencial | regressão comprovada em rota protegida de projeto existente |
| `redesign_ux_critique` | Agente, consultivo | finding heurístico não é gate técnico até aceito como requisito |

Status são `pass`, `fail`, `warn`, `blocked`, `not_evaluated`. Cada check inclui `executed`, `environment`, `tool_version`, `scenario_ids`, `evidence_ids`, `started_at`, `duration_ms`, `limitations`, severidade e rota de reparo. **Nunca mapear falha de plugin para sucesso.**

### 12.2 Critérios para os níveis Ready

- Concept: compila/renderiza ou sinaliza que é mock visual; fornece decisões e limites. Bloqueio de segurança ainda impede entrega não sanitizada.
- Prototype: `build=pass`, DS strict se configurado `pass`, segurança `pass`, **100% dos cenários críticos com oráculo do escopo passaram**, nenhum comportamento conhecido crítico omitido sem aprovação, relatório de cobertura válido. Outros cenários não medidos continuam visíveis. Se não existem cenários executáveis, não qualificar como Prototype Ready sem aprovação explícita do escopo e denominação alternativa.
- Engineering: condições Prototype + contratos de integração definidos para partes reais, suíte requerida aprovada, regressão de rotas protegidas aprovada, teste de acessibilidade relevante com revisão humana pendente **não** tratado como certificado. Necessita revisão de engenharia externa antes de produção.
- `blocked` sempre vence `ready`. `not_evaluated` obrigatório bloqueia qualificação; apenas checks opcionais podem ficar qualificados como não avaliados.

### 12.3 Performance e qualidade

- Build com TypeScript estrito e testes executáveis; lint, imports consistentes, sem hardcoded de DS strict; assert de bundle e avisos.
- Playwright: navegação, tecla Tab/Enter/Escape, mensagens de erro, estados vazios, responsividade, teclado e foco.
- axe-core: reportar violações com escopo e versão. WCAG 2.2 AA é **alvo de avaliação**, não certificação automatizada.
- Perf: Web Vitals/Lighthouse em ambiente comparável, com dados representativos e logs de hardware/rede; não declarar melhora com amostras incomparáveis.
- UX: teste humano de tarefas, tempo, sucesso, erros, satisfação; se não houver, `ux_improvement=not_measured`, mesmo quando critic considera melhor.

## 13. Ameaças e controles de segurança (STRIDE resumido)

**Ativos:** browser autenticado, cookies, dados de clientes, código corporativo, tokens MCP, design system privado, registros de tráfego, arquivos gerados.

**Ameaças principais:** SSRF via URL/endpoint, execução arbitrária de install scripts, prompt injection em páginas/MCP/README, tool poisoning, troca de contrato MCP, exfiltração em logs ou prompts, egress de dados para IA externa, mutação acidental em produção, supply chain de pacotes, vazamento de HAR, cross-tenant, navegação para domínio não autorizado, manipulação de artefatos e paths.

**MUST:**

1. Permissões mínimas, env/test-first, allowlist de origens e métodos, bloqueio de mutações na origem sem autorização. O navegador pode disparar requisições normais — usar ambiente de teste isolado e não afirmar contenção impossível em browser comum.
2. MCP por servidor + catálogo completo fixado por digest; anotações não confiáveis; mudanças exigem revisão.
3. Secrets no OS keychain/variáveis de ambiente controladas; nunca persistir tokens, IDs pessoais ou corpos de resposta sensíveis em artefatos compartilhados.
4. Dados destinados ao modelo passam por política de egress: `local_only`, `allow_remote_redacted`, `allow_remote_explicit`. O usuário deve saber quando conteúdo sai do dispositivo.
5. Sanitização deny-by-default de HAR/JSON, URLs, headers e corpos. Detectores são auxiliares: capturas brutas não são exportadas automaticamente, mesmo se scanners não acharem nada.
6. Sandboxing/isolamento dos builds e subprocessos quando disponível; sem segredos montados no processo de geração; fail-closed se controles obrigatórios não disponíveis.
7. Logs de auditoria sem conteúdo sensível; identificadores de evidência, usuário/ambiente pseudonimizados, retentativa rastreável.
8. Cancelamento e budgets obrigatórios por job; subprocessos sempre via process manager do Craft; política de limpeza e retenção configurável.
9. Hash/versionamento de dependências, lockfile e SBOM no release. CI testa licenças da árvore real; pin de REA/Layout.
10. Tratamento de conteúdo de site/MCP como **dados não confiáveis**. Instruções externas não podem mudar políticas, sistema de prompts ou autorizações.

### 13.1 Acesso ao browser / CDP

- Perfil dedicado quando REA precisa de CDP; porta apenas loopback. Preferir a sessão coordenada pelo Craft, sem dois controladores simultâneos em execução conflitante.
- Login interativo humano e perfis temporários; não exportar storage state, cookies ou HAR. Limpar no encerramento; falha de cleanup registrada.
- Em aplicações SSO/MFA, nunca fazer bypass. Se automação for bloqueada, solicitar sessão ou evidência autorizada.

### 13.2 Direitos de uso e licenciamento

- Craft: MIT OR Apache-2.0. REA, Layout CLI/Layout UI: MIT; Velloo Apache-2.0; Studio Layout AGPL-3.0 (não integrar código no core sem revisão). Carbon MCP hosted ≠ software local completo open source.
- Dados/documentação externos têm licença própria, mesmo quando pacote de código tem MIT. Não redistribuir conteúdo sem permissão. Registrar autoria, URL, licença, versão, hash e avisos em `THIRD_PARTY_NOTICES.md`/SBOM.
- Usar apenas aplicações que usuário possua ou tenha autorização para inspecionar; não copiar marca/logo/assets de terceiros automaticamente.

## 14. CLI, MCP e configuração [APIs PROPOSTAS]

### 14.1 Ferramentas públicas, headless-first

Seguir a regra do Craft: uma capacidade = `ToolSpec`, acessível por engine, CLI e MCP; GUI somente cliente. Novos IDs propostos:

| ToolSpec ID | Efeito esperado |
|---|---|
| `redesign_project_configure` | persistir config validada, sem secrets |
| `redesign_study_run` | iniciar investigação autorizada, cria evidência |
| `redesign_capabilities_list` | paginar capacidades e desconhecidos |
| `redesign_coverage_show` | mostrar denominadores, escopo e lacunas |
| `redesign_ds_import` | importar registry com aprovação de fonte |
| `redesign_ds_check` | verificar conformidade de código |
| `redesign_app_context_query` | consultar MCP autorizado através de gateway |
| `redesign_rea_probe` | probe REA opcional, autorizado, devolver evidência |
| `redesign_plan_generate` | criar alternativas/decisões/viabilidade |
| `redesign_plan_approve` | registrar aprovação explícita para uma mudança |
| `redesign_build_run` | direcionar ao runner existente do Craft |
| `redesign_checks_run` | executar checks nativos, sem aprovação textual |
| `redesign_report_export` | gerar report sanitizado e lista de artefatos |

O conjunto final de `ToolSpec` deve evitar wrappers artificiais quando o comando já existir no Craft. Não duplicar a API do runner; criar *facade* apenas para coordenação e contracts. Ferramentas `list` devem oferecer `offset`, `limit`, filtros e limites de payload.

### 14.2 CLI proposta

Comandos **a implementar no fork**, não afirmar que já existem:

```bash
# No primeiro PR, manter binário do Craft; renomear/alias na fase de distribuição.
craft-anything-cli redesign init https://app.example.test \
  --mode redesign --capture clean-room --project-folder ./project
craft-anything-cli redesign study --project-folder ./project
craft-anything-cli redesign ds import --project-folder ./project \
  --source ./corporate-design-system --policy strict
craft-anything-cli redesign plan --project-folder ./project
craft-anything-cli redesign build --project-folder ./project
craft-anything-cli redesign check --project-folder ./project
craft-anything-cli redesign report --project-folder ./project
```

Revalidar convenções da CLI existente e preferir subcomandos que usem `ToolSpec`; não inserir flags arbitrárias no core sem adaptação de parser e testes. A embalagem final poderá expor `redesign-anything` como alias/documentação sem substituir uma ferramenta pronta antes da hora.

### 14.3 Exemplo de configuração

```json
{
  "schema_version": "1.0.0",
  "project_id": "redesign-demo",
  "target": {"kind": "website", "url": "https://app.example.test"},
  "mode": "redesign",
  "capture_mode": "clean_room",
  "scope": {"max_routes": 12, "max_journeys": 5, "roles": ["operator"]},
  "design_system": {"kind": "local_repo", "source": "./corporate-ui", "policy": "strict"},
  "providers": {"rea": "disabled", "application_mcp": "disabled"},
  "backend": {"mode": "mock", "allow_production_writes": false},
  "model": {"use_craft_selected_backend": false},
  "execution": {"controller": "host_agent", "profile": "lean", "allow_nested_builder": false,
    "budgets": {"token_soft_limit": 750000, "max_mcp_calls": 30, "max_rea_invocations": 2,
      "max_critic_passes": 1, "max_tool_result_chars": 12000, "max_run_minutes": 90, "no_progress_limit": 2}},
  "privacy": {"model_egress": "local_only", "export_raw_captures": false},
  "checks": {"max_refine_rounds": 3, "require_critical_scenarios": true}
}
```

Esses defaults são **propostos**, não números observados de performance. Os schemas reais devem validar URL, escopo, enumerations e referências; `local_only` com agente remoto é incompatibilidade e deve bloquear até ajuste explícito.

### 14.4 Uso agent-first e bootstrap por repositório GitHub — requisito obrigatório P0

**Caso de uso central:** alguém cola a URL do repositório Redesign Anything no Cursor/Claude Code/Codex/ACP e pede ao agente para usá-lo, como já faz com o Craft. Um GitHub URL é uma entrada de descoberta, não uma instalação executável. O agente deve obter o repositório por mecanismos permitidos, ler documentos de confiança e completar pré-requisitos com consentimento.

**O pacote de distribuição DEVE conter** README com quickstart de agente, `AGENTS.md` do produto, `docs/AGENT_FIRST.md` (este kit inclui `AGENT_FIRST.md`), `docs/extending.md` adaptada, CLI e MCP documentados, exemplos de configuração, fixtures offline e `doctor` headless. MAY oferecer `skills/redesign-anything/SKILL.md` portátil, desde que não quebre agentes que não implementam Skills. Nenhuma capacidade dependerá de janela egui.

**Contrato do fluxo:**

1. Agente recebe URL do repositório Redesign Anything; valida origem/commit e obtém o código por Git/checkout autorizado.
2. Lê README/`AGENTS.md`/guia agent-first, executa diagnóstico **sem alterar sistema** (`doctor --json`, sintaxe exata após implementar).
3. Informa pré-requisitos ausentes e solicita consentimento antes de instalar dependências, executar programas terceiros, acessar browser autenticado ou configurar MCP.
4. Permite invocar redesign pelo CLI ou expor as mesmas operações via MCP; lista e chama `redesign_*` sem UI. Evita duplicar o workflow Craft.
5. Retorna artefatos locais no projeto, status, evidências, cenário de teste, relatório de cobertura e limitações, inclusive quando o processo é interrompido.
6. Em retomada, preserva projeto, versão e evidências em vez de repetir captura e alterar respostas sem registrar.

**Comando futuro (PROPOSTO, não implementado):** `redesign-anything-cli redesign --url <url> --mode redesign --config <path>`; o nome executável e flags DEVEM ser confirmados contra os binários reais do fork no PR-01. As ferramentas MCP previstas estão na seção 14.

**Critérios de aceite:** uma máquina limpa com Git, Rust e dependências do SO documentadas; agente consegue seguir apenas instruções do repositório, preparar o CLI, identificar ausência de browsers/Node/REA/Layout e executar o fixture P0 sem abrir desktop. O fluxo também funciona quando REA/Layout/MCP da aplicação não estão instalados. Todas as instalações e ações com efeito são explícitas; nenhum script obtido da web é executado implicitamente.

**Camada de ergonomia:** `README` oferece três prompts copiados: (a) preparar instalação; (b) redesenhar URL + DS; (c) validar projeto salvo. Devemos medir desde o início a fricção de copiar uma URL no agente versus iniciar por MCP pré-instalado.

## 15. Observabilidade, controle de custo e erro

**Contrato canônico:** `COST_AND_EXECUTION.md`. Orçamento de tokens do agente hospedeiro é apenas **advisory** quando seu cliente não fornece telemetria de uso. Limites efetivamente aplicáveis sobre ferramentas/sidecars executados pelo Redesign são hard; nenhum relatório pode tratar tokens não observáveis como zero.

- Evento estruturado por etapa (`study_started`, `capture_completed`, `mcp_query_approved`, `rea_probe_completed`, `decision_recorded`, `build_started`, `check_completed`, `ready_gate_evaluated`) sem dados sensíveis.
- Métricas: duração por etapa, chamadas REA/MCP, tokens de entrada/saída/cache **quando reportados**, qualidade da contabilidade, custo apenas quando comprovável, operações não executadas, chamadas e cache hits, cobertura do escopo, rodadas, checks e unknowns. Não somar cached input duas vezes.
- Limites configuráveis de rotas, profundidade, cenários, artefatos, chamadas MCP/REA, número de críticas, tempo por processo, refinamentos e payload das ferramentas. `token_soft_limit` é **aviso**: hard cap só quando houver contador e controle de parada do modelo; caso contrário `warning_unenforceable`. UI e CLI mostram limites antes de uma investigação potencialmente longa.
- `budget_exhausted`: preservar progresso parcial e retornar estado próprio `budget_exhausted` com `needs_evidence` quando pertinente; não converter checks desconhecidos em `pass` nem tratar uso de tokens não observável como limite cumprido.
- Retry apenas para falha transitória/idempotente; chamadas com efeito nunca retry automático sem idempotency key e autorização.
- Determinismo: versões pinadas, datas de captura, seeds, screen sizes, fixture, ambiente e locale. Modelo pode variar, mas toda verificação objetiva é reexecutável.

**Estimativas de planejamento, não medições:** tela simples 150–400 mil tokens processados; tela corporativa 400 mil–1,2 milhão; jornada de 3–5 telas 1–3 milhões; módulo 10–20 telas 3–10 milhões; aplicativo extenso acima disso. Incluem contexto repetido e podem variar amplamente. Não equivalem à janela de contexto nem ao custo cobrado. Os *defaults* experimentais, sem garantia de enforcement para agente externo, constam em `COST_AND_EXECUTION.md`. O Craft documentou testes de uso de cache e loops caros em `docs/ROADMAP.md` (R5.3/R5.4): registrar o baseline antes de estabelecer critérios numéricos.

## 16. E2E de referência e jornada de aceite

**Teste adicional obrigatório:** executar o mesmo fixture com o agente hospedeiro como controlador único, validar que `rebuild_start` não é iniciado por padrão e reportar tokens do host como `unavailable` quando não houver metadados. A comparação Craft vs Lean vs Deep está em `COST_AND_EXECUTION.md` e `VALIDATION_MATRIX.md`.

Aplicação fixture open source construída para o teste, com regras publicadas separadamente da versão inspecionada:

- Gestão genérica de solicitações: listagem densa, filtros, paginação, edição, detalhe, workflow de estados, validações dependentes, permissões (operator/approver), erros e efeitos persistidos no backend de testes.
- Design system corporativo fixture: pacote React/TS com DataTable, FilterBar, FormField, StatusBadge, Dialog, Toast, tokens DTCG e Storybook, incluindo variantes/props incompatíveis deliberadas.
- Application MCP fixture: duas ferramentas read-only com regras declaradas e uma ferramenta mutante bloqueada em produção; um tool malicioso para teste de prompt injection.
- REA fixture: JavaScript distribuído com rotas SPA e validação; HAR sintético com token-canário (não real) e cenários de escrita confirmada/não confirmada.

**Jornada de aceite do P0:**

1. Usuário abre Craft fork → seleciona Redesign, URL fixture, modo Redesign, DS corporativo estrito.
2. Study mapeia 1 jornada crítica, evidências, unknowns e erros, sem acesso a código fonte.
3. Plan propõe reorganização justificada, `keep_as_is`, preservações e viabilidade.
4. Build gera React/TS utilizando imports reais do DS, com dados sintéticos e fluxo funcional.
5. Check detecta uma regressão intencional, refina uma vez e só aprova após testes passarem.
6. Export inclui projeto, decisões, coverage com denominador, riscos, limitations, licenças e logs sanitizados.
7. Teste de segurança confirma que não houve escrita na aplicação original nem segredo nos artefatos.
8. Com REA desligado, o P0 continua; com REA habilitado, só registra ganho se houver evidência nova.

**Aceite P0:** build e testes passam; zero violações bloqueantes de DS/safety; nenhuma chamada mutante para original; 100% dos cenários críticos **definidos e executáveis do escopo** passam; toda capacidade crítica conhecida tem destino ou exceção aprovada; unknowns e fontes visíveis. Não exigir que a IA descubra regras desconhecidas.

## 17. Plano de entrega e dependências

O `IMPLEMENTATION_PLAN.md` contém PRs pequenos, arquivos, testes e critérios de aceite. Resumo:

1. PR-00: fork, licenças, baseline testado e adaptações de `AGENTS.md`/roadmap.
2. PR-01: `RedesignKind`, defaults e comando headless mínimo.
3. PR-02: workspace `redesign/`, schemas, migrações, status e source refs.
4. PR-03: Study adapter + Capability/Evidence/Coverage.
5. PR-04: Knowledge Packs + retrieval tipado, critic básico.
6. PR-05: DS Registry + integração Layout CLI + compliance estrito.
7. PR-06: Plan/Feasibility/Traceability + aprovação de decisões.
8. PR-07: React/TS Build + briefing + mock/dados sintéticos.
9. PR-08: checks nativos fail-closed + Playwright/axe + Ready gates.
10. PR-09: Application MCP Bridge e Trust Gateway.
11. PR-10: REA deep adapter e avaliação incremental; ambientes simulados se comprovado ganho.
12. PR-11: E2E benchmark multi-perfil + segurança + scaling.
13. PR-12: polish da UI Craft, distribuição/empacotamento, SBOM, documentação e release.

Ordem importa: **PR-00 → PR-03 → PR-05 → PR-08** fecha o caminho de valor antes de adicionar várias integrações. Se uma dependência adicional não elevar métricas frente à base, manter como plugin opcional.

## 18. Definição de pronto, incertezas e decisão final

### 18.1 Para cada PR

- Código compila; testes unitários/integração adicionados; novo arquivo versionado tem migração/validação.
- Comandos Craft apropriados passam (`cargo fmt`, clippy, test, `cargo xtask layers`, locks, wasm quando tocar L0–L2, secrets e checks de i18n quando relevante). Revalidar sintaxe no checkout.
- Nenhum novo caminho de execução bypassa consentimento, cancela indevidamente jobs ou invade Clean Room.
- Documentação `AGENTS.md`, roadmap/status/tools atualizada no fork; teste live descrito como executado ou não.
- Fallbacks e erros exercitados; checks de segurança falham fechado.
- O PR não altera funcionalidade de clonagem tradicional do Craft sem teste de regressão.

### 18.2 Decisões já tomadas

- Arquitetura **fork do Craft**, não plataforma independente.
- Diferencial: Design Intelligence, DS corporativo, transformação e verificação — não engenharia reversa duplicada.
- REA/MCP da aplicação opcionais, conhecimento por fontes, egress controlado.
- React/TypeScript e UX web como primeiro alvo; futuras plataformas podem reaproveitar a engine.
- Três níveis Ready e `Confidence & Coverage` obrigatórios, com epistemologia honesta.
- Custo baixo, free/open source, sem API paga obrigatória; inferência local pode ser limitada.

### 18.3 Questões para **medir**, não para decidir por palpite

- Qual modelo/agente, local ou autenticado, consegue executar uma cadeia útil de Redesign + Check em hardware real?
- Quanto REA acrescenta à cobertura de regras em relação ao Craft sozinho, por unidade de esforço/tempo?
- Quais limitações de Storybook/TypeScript o Registry não consegue captar para o DS-alvo?
- Em quais padrões de aplicações o consolidar fluxos piora produtividade?
- Qual taxa de falso positivo/falso negativo dos gates de paridade e DS?
- Qual tempo/custo aceitável da descoberta com 5, 20, 100 rotas e vários perfis?
- Como empacotar sidecars assinados e verificados em Windows/macOS/Linux sem instalar pacotes silenciosamente?

### 18.4 Limites não resolvíveis genericamente

Nenhuma observação finita garante descobrir regras nunca exercitadas; nenhum MCP necessariamente sabe explicar regras internas; nenhuma simulação reproduz garantidamente efeitos de backend não observados; não há comprovação de “UX melhor” sem evidência/avaliação de uso; nenhuma suíte automatizada prova ausência universal de bugs, falhas de segurança ou WCAG completa. O produto expõe essas fronteiras como parte da entrega.

## 19. Referências canônicas e leitura antes da implementação

- Craft [`README`](https://github.com/storytold/craft-anything/blob/main/README.md), [`AGENTS.md`](https://github.com/storytold/craft-anything/blob/main/AGENTS.md), [`docs/extending.md`](https://github.com/storytold/craft-anything/blob/main/docs/extending.md), [`kinds/mod.rs`](https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/mod.rs), [`plugin_check.rs`](https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/plugin_check.rs), [`docs/STATUS.md`](https://github.com/storytold/craft-anything/blob/main/docs/STATUS.md), [`docs/ROADMAP.md`](https://github.com/storytold/craft-anything/blob/main/docs/ROADMAP.md).
- REA [`README`](https://github.com/morluto/rea/blob/main/README.md), [`browser observation`](https://github.com/morluto/rea/blob/main/docs/guides/web/browser-observation.md), [`JS semantics`](https://github.com/morluto/rea/blob/main/docs/guides/javascript/javascript-application-workflows.md), [`agent environments`](https://github.com/morluto/rea/blob/main/docs/agent-environments.md).
- Layout CLI [`README`](https://github.com/uselayout/cli/blob/main/README.md), [`LICENSE`](https://github.com/uselayout/cli/blob/main/LICENSE); Layout UI [`README`](https://github.com/uselayout/ui/blob/main/README.md).
- W3C [Design Tokens 2025.10](https://www.w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/); [WCAG 2.2](https://www.w3.org/TR/WCAG22/); [Understanding Conformance](https://www.w3.org/WAI/WCAG22/Understanding/conformance).
- MCP [tool annotations and risk](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/); OWASP [MCP Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/MCP_Security_Cheat_Sheet.html).



## 20. Adendo vinculante v1.2.0 — revisão crítica do council

> **Precedência:** esta seção e `COUNCIL_DECISIONS.md` prevalecem sobre descrições de escopo/ordem v1.1.0 que as contradigam. Os schemas 1.0.0 continuam compatíveis; mudanças de Task Model são inicialmente arquivos opcionais e NÃO tornam o formato de configuração anterior inválido.

### 20.1 Tese e critério de diferenciação

O produto não é uma promessa de modernização integral de sistemas a partir de URL. É um **agente open source de redesign de interfaces e jornadas web baseado em evidências, que produz código verificável e limitações explícitas**. A hipótese a demonstrar é que, sob a mesma aplicação, agente, modelo, orçamento e critérios, o REA supera (A) Craft com prompt comum e (B) Craft com instruções estruturadas de UX em pelo menos um resultado relevante sem regressões críticas. **Se B alcançar desempenho equivalente, simplificar para skill + checks em vez de manter subsistemas desnecessários.**

### 20.2 Primeiro vertical slice interno obrigatório (não release público)

O primeiro checkpoint técnico interno é: URL/fixture autorizada → Study reutilizado do Craft → Task Model mínimo → diagnóstico com evidências → proposta incluindo `keep_as_is` → build React/TS → checks essenciais (build, smoke funcional, proveniência e honestidade de cobertura) → relatório Concept/Prototype Ready. CLI/MCP no `host_agent`, sem desktop obrigatório. **Este marco não autoriza publicação da V1.** DS corporativo, inteligência rica, REA, Application MCP, jornadas e qualidade são capacidades exigidas antes do primeiro release público, com ativação seletiva conforme `V1_IMPACTANTE.md`.

### 20.3 Task Model e conteúdo

Cada jornada sob avaliação DEVE registrar `task_id`, objetivo, persona/papel quando conhecido, frequência (`unknown` é válido), criticidade, volume, precondições, resultados esperados, dependências, fonte e métricas antes/depois quando disponíveis. Campos sem evidência ficam `unknown`; não inferir métricas. A qualidade do conteúdo inclui rótulos, terminologia de domínio, microcopy, mensagens de erro, estados vazios e linguagem clara; decisões textuais devem referenciar evidências ou hipótese.

### 20.4 Design System Policy com três categorias

`required_component`: componente oficial obrigatório para função coberta; `allowed_composition`: composição estrutural autorizada, sem duplicar componentes oficiais; `extension_requires_approval`: lacuna com proposta explícita e aprovação. A política `strict` aplica estas categorias e **não proíbe automaticamente toda marcação HTML sem equivalente no registry**. Testes verificam import, props, versão, tokens, semântica de estados e justificativa de exceção. Configuração legada `strict/advisory` permanece válida.

### 20.5 Evidência de melhoria e criticidade

Checks determinísticos provam somente seus critérios; Design Critic/LLM judge fornecem recomendações, não certificação. O protocolo de avaliação deve incluir tarefas representativas, preservação de resultados, sucesso/tempo/erros quando mensuráveis, acessibilidade manual quando aplicável, conteúdo, densidade e a opção `keep_as_is`. Não declarar ganho de UX sem benchmark ou estudo de usuários. Reprovação de cenário funcional crítico conhecido impede `prototype_ready`, independentemente da aparência.

### 20.6 Governança de execução e early stopping

Manter controlador único `host_agent` XOR `craft_runner`, modo Lean e REA/MCP opcionais. Registrar chamadas, tempo, payloads, progresso nos checks e tokens quando disponíveis. Interromper refinamento repetido sem melhoria objetiva; quando contadores não forem observáveis, usar limites efetivos de tempo/chamadas/rodadas, reportando `usage=unavailable`, jamais zero fictício. Não impor orçamentos de tokens como limites garantidos se o backend não permite enforcement.

### 20.7 Gatilhos de simplificação e interrupção

Reavaliar fork se a extensão exigir mudanças intrusivas no core; reavaliar Design Engine se não superar baseline B; descartar REA/integração opcional sem ganho marginal; interromper expansão quando houver perda de funções críticas, risco de dados sensíveis ou execução descontrolada. Registrar decisões em ADR com evidência e responsável. Critérios quantitativos serão definidos **antes** do benchmark, não depois de ver resultados.

### 20.8 Condições de segurança e produto

Não executar instalações/scripts não confiáveis sem aprovação e isolamento; segregar projetos e credenciais; impedir exportação de sessões e dados privados; aplicar escopo de autorização por projeto. `Engineering Ready` continua significando pronto para revisão e integração, **nunca aprovação automática de produção**. O produto pode retornar `needs_evidence` e `keep_as_is` como resultados válidos.

## Adendo normativo v1.3.0 — controles do Council II

**Precedência:** em conflito com seções anteriores, prevalecem `TRUST_BOUNDARIES.md`, `ORACLE_POLICY.md`, `SAFE_WORKSPACE_POLICY.md` e `OSS_DX_AND_UPSTREAM.md`. Estas regras são requisitos de implementação, não funcionalidades comprovadas.

1. `host_agent` não implica sandbox global. O produto limita apenas suas ferramentas; comandos externos do agente ficam sob políticas do host.
2. Toda escrita no projeto-alvo exige diff e aprovação; saída padrão é workspace isolado.
3. Testes de paridade usam oráculos independentes e rastreáveis; hipótese de LLM nunca é expected result verificado.
4. Evidências incluem ambiente, versão, role, tenant, flags, origem, retenção e política de redaction.
5. MCP, páginas e documentos são fontes de dados não confiáveis, nunca instruções executáveis ou autoridade de aprovação.
6. Design systems corporativos precisam de versão/hash, direitos de uso e cache segregado; ativos privados não são redistribuídos.
7. Release exige compatibilidade declarada por agente/SO, acessibilidade além de checks automáticos e atualização upstream auditável.
8. Em divergência entre segurança e DX, `doctor` permanece sem efeitos colaterais; instalações exigem consentimento.
9. `Concept Ready`, `Prototype Ready` e `Engineering Ready` respeitam os gates da `ORACLE_POLICY.md`. Nenhum nível equivale automaticamente a produção.
