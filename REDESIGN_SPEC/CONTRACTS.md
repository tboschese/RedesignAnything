# Redesign Anything — Contratos, esquemas e políticas

> **Status:** propostas v1; ainda não são APIs oficiais do Craft/REA/Layout. Verificar tipos existentes no checkout do fork antes de introduzir estruturas duplicadas. Os JSON Schemas reais em `schemas/` são a referência de validação de persistência. Exemplos ilustrativos com domínio genérico, sem dados reais.

## 1. Convenções transversais

- `schema_version`: string SemVer (`1.0.0`). `project_id`: slug estável. `revision`: inteiro monotônico; escritura atômica.
- IDs: prefixos `ev_`, `cap_`, `rule_`, `scenario_`, `decision_`, `ds_`, `check_`; evitar colisão mesmo quando input veio de vários providers.
- JSON do projeto não carrega tokens, cookies, segredos, senhas, URLs com credenciais ou corpos sensíveis. Credenciais são referências a secret storage.
- Todos os `source_refs` são pointers a evidências normalizadas. `raw_source` é externo, opcional e somente local conforme autorização.
- O payload de LLM é sempre *projeção* pequena/sanitizada desses contratos, não dump irrestrito de capturas.
- Nunca inventar métricas. `null` ou `not_evaluated` é diferente de `0` ou de sucesso.

## 2. Enums e tipos fundamentais (TypeScript de referência)

```ts
type ProjectId = string;
type EvidenceId = `ev_${string}`;
type CapabilityId = `cap_${string}`;
type RuleId = `rule_${string}`;
type ScenarioId = `scenario_${string}`;

type CaptureMode = 'clean_room' | 'deep';
type RedesignMode = 'refresh' | 'redesign' | 'reimagine';
type SourceAuthority = 'craft' | 'rea' | 'app_mcp' | 'user' | 'source_repo' | 'test';
type KnowledgeStatus = 'observed' | 'declared' | 'inferred' | 'unknown' | 'validated' | 'refuted';
type CoverageStatus = 'covered' | 'partial' | 'uncovered' | 'unknown';
type Readiness = 'concept_ready' | 'prototype_ready' | 'engineering_ready' | 'needs_evidence' | 'blocked' | 'budget_exhausted';
type CheckStatus = 'pass' | 'fail' | 'warn' | 'not_evaluated' | 'blocked';
type Feasibility = 'ui_only' | 'frontend_orchestration' | 'backend_change_required' | 'not_determined';
type Criticality = 'critical' | 'high' | 'normal' | 'low';
type Sensitivity = 'public' | 'internal' | 'restricted' | 'unknown';
type ExecutionController = 'host_agent' | 'craft_runner';
type ExecutionProfile = 'lean' | 'deep';
type AccountingQuality = 'provider_reported' | 'host_reported' | 'estimated' | 'unavailable';

// Valores com unidades explícitas; todos opcionais e respeitados somente quando
// observáveis/controláveis pelo executor. Nunca inferir billing de tokens processados.
interface ExecutionBudgets {
  token_soft_limit?: number; // aviso; não hard cap universal sobre Cursor/Claude/Codex
  max_mcp_calls?: number; // chamadas realizadas pelo app MCP Gateway
  max_rea_invocations?: number; // chamadas através do REA Adapter
  max_critic_passes?: number; // revisões orquestradas pelo Redesign
  max_tool_result_chars?: number; // caracteres de resposta resumida; evidência completa paginável
  max_run_minutes?: number; // processos supervisionados; não encerra host externo
  no_progress_limit?: number; // passos sem progresso sob controle local
}
interface ExecutionPolicy {
  controller: ExecutionController;
  profile: ExecutionProfile;
  allow_nested_builder: false;
  budgets?: ExecutionBudgets;
}
interface UsageSnapshot {
  schema_version: '1.0.0';
  run_id: string;
  source: string; // Craft runner, agente hospedeiro, API de backend, etc.
  accounting_quality: AccountingQuality;
  input_tokens_reported: number | null; // quando reportado, inclui cached se aplicável
  output_tokens_reported: number | null;
  cached_input_tokens_reported: number | null; // subconjunto, não somar novamente
  tokens_estimated: number | null;
  cost_amount: number | null; // null se não mensurável
  cost_currency: string | null;
  tool_calls: number;
  captured_at: string;
  limitations: string[];
}

interface FileEnvelope {
  schema_version: '1.0.0';
  project_id: ProjectId;
  revision: number;
  generated_at: string; // ISO 8601 UTC
}

interface EvidenceRecord {
  id: EvidenceId;
  source: SourceAuthority;
  source_operation: string; // ex.: Craft tool ID, REA operation ou MCP tool ID
  subject: string; // slug de página, capability ou operação
  kind: 'ui' | 'behavior' | 'business_rule' | 'api' | 'design_system' | 'test' | 'user_context';
  status: KnowledgeStatus;
  scope: { target_origin?: string; route?: string; role?: string; environment?: string; app_revision?: string };
  locator?: { artifact_path?: string; line?: number; step_id?: string; external_evidence_id?: string };
  digest?: string;
  observed_at: string;
  limitations: string[];
  sensitivity: Sensitivity;
  summary: string; // já sanitizado
}

interface Capability {
  id: CapabilityId;
  title: string;
  task: string;
  actor_roles: string[];
  original_routes: string[];
  inputs: string[];
  actions: string[];
  outputs: string[];
  preconditions: string[];
  states: string[];
  rule_ids: RuleId[];
  evidence_ids: EvidenceId[];
  criticality: Criticality;
  coverage: CoverageStatus;
  unknowns: string[];
  scenario_ids: ScenarioId[];
}

interface BusinessRule {
  id: RuleId;
  statement: string;
  preconditions: string[];
  effects: string[];
  status: KnowledgeStatus;
  evidence_ids: EvidenceId[];
  affected_capability_ids: CapabilityId[];
  contradictions: EvidenceId[];
}

interface DesignComponent {
  id: string;
  package_name: string;
  package_version: string;
  import_path: string;
  export_name: string;
  props_schema: Record<string, unknown> | null; // JSON Schema de props ou null
  variants: string[];
  tokens: string[];
  allowed: boolean;
  usage: string[];
  never: string[];
  evidence_ids: EvidenceId[];
}

interface DesignDecision {
  id: string;
  capability_ids: CapabilityId[];
  problem: string;
  options: Array<{ id: string; description: string; risks: string[] }>;
  chosen_option_id: string;
  rationale: string;
  ux_rule_ids: string[];
  evidence_ids: EvidenceId[];
  feasibility: Feasibility;
  dependencies: string[];
  status: 'proposed' | 'approved' | 'rejected';
  approval_evidence_id?: EvidenceId;
}

interface ScenarioOracle {
  id: ScenarioId;
  capability_id: CapabilityId;
  profile: string;
  fixture_id: string;
  given: string[];
  when: string[];
  then: string[];
  assertion_source: 'observed' | 'declared' | 'user_approved';
  evidence_ids: EvidenceId[];
  safe_execution_target: 'generated_mock' | 'sandbox' | 'staging';
  executable: boolean;
}

interface CheckResult {
  id: string;
  check_id: string;
  status: CheckStatus;
  severity: 'blocking' | 'major' | 'minor' | 'info';
  executed: boolean;
  environment: string;
  tool_version: string | null;
  scenario_ids: ScenarioId[];
  evidence_ids: EvidenceId[];
  started_at: string;
  duration_ms: number;
  message: string;
  limitations: string[];
}
```

## 3. Configuração de projeto

Contrato de configuração: `schemas/redesign-config.schema.json`. Telemetria persistida: `schemas/usage-snapshot.schema.json`. **Versão da documentação 1.1.0 e dos JSON Schemas 1.0.0 são deliberadamente distintas:** as alterações adicionam campos opcionais e não quebram os documentos existentes; antes de publicar um fork, definir política formal de migrações.

Campos obrigatórios: `schema_version`, `project_id`, `target`, `mode`, `capture_mode`, `scope`, `design_system`, `providers`, `backend`, `model`, `privacy`, `checks`. `execution` é opcional (compatível com o contrato v1.0.0), com defaults dependentes do ponto de entrada: agente externo → `host_agent`; UI/CLI Build do Craft → `craft_runner`; profile `lean` em ambos. `target` descreve somente origem de dados, não autoriza mutação. Origem web precisa ser HTTP(S). Capturas de screenshot sem URL são possíveis, mas funcionalidades não podem receber status `observed` de execução sem teste.

**Validação de conflitos:**

- `model_egress: local_only` + backend de modelo comprovadamente remoto → bloquear e pedir opção consciente.
- `capture_mode: clean_room` + REA JS decompilation/source export → bloquear e recomendar Deep com aprovação.
- `design_system.policy: strict` + registry indisponível → bloquear Build; Study pode seguir.
- `allow_production_writes: false` + MCP mutante → negar chamada; isso não depende de `readOnlyHint`.
- URL final/redirecionada fora de origins autorizadas → recusar investigação da nova origem.
- `execution.controller=host_agent` + `model.use_craft_selected_backend=true` → configuração inválida; o agente hospedeiro controla seu próprio modelo.
- `execution.controller=host_agent` + tentativa implícita de `rebuild_start`/modelo aninhado → `controller_conflict`; handoff só com aprovação explícita.
- `execution.profile=deep` nunca muda `capture_mode=clean_room` sozinho; REA que lê JS interno requer consentimento Deep.
- `token_soft_limit` excedido, mas sem telemetria/controle do agente hospedeiro → `warning_unenforceable`, não alegar bloqueio de tokens.
- `max_mcp_calls` / `max_rea_invocations` → hard cap apenas para chamadas que passam pelo gateway do Redesign; contagem durável por run.

**Semântica de métricas e estados:** `COST_AND_EXECUTION.md`. Um UsageSnapshot deve indicar ausência de dados com `null`/`unavailable`, não `0`. `cached_input_tokens_reported` normalmente já integra `input_tokens_reported`; não duplicar a contagem. Nenhum campo de custo é inferido da assinatura do host.

## 4. Application Knowledge Contract (AKC) — opcional e não padrão MCP

Esse contrato é uma **convenção do Redesign Anything**, não parte mandatória do MCP. Um servidor MCP da aplicação pode implementá-lo ou mapear ferramentas existentes.

### 4.1 Ferramentas sugeridas

```text
app_context_list_capabilities(scope?, role?)
app_context_describe_capability(id)
app_context_list_business_rules(capability_id)
app_context_explain_business_rule(rule_id)
app_context_list_state_transitions(entity, role?)
app_context_get_field_constraints(capability_id, field_id?)
app_context_get_permissions(role, capability_id?)
app_context_validate_scenario_readonly(scenario)
app_context_get_version()
```

Usos: responder dúvidas técnicas, não obter credenciais ou executar operações. Respostas devem indicar `capability_id`, `version`, `environment`, `role_context`, `rule_ids`, `source_refs`, `completeness` e `limitations` quando disponíveis.

### 4.2 Exemplo de resposta semântica

```json
{
  "schema_version": "1.0.0",
  "application": "example-requests",
  "environment": "staging",
  "app_version": "2026.11.4",
  "capability_id": "cap_review_request",
  "title": "Revisar solicitação",
  "required_inputs": ["request_id", "review_decision"],
  "allowed_roles": ["approver"],
  "from_states": ["pending_review"],
  "outcomes": ["approved", "rejected"],
  "limitations": ["A política de aprovação externa não está documentada neste contrato"],
  "completeness": "partial"
}
```

Esta resposta é `declared`; só vira `validated` com testes/oráculos aprovados. Evitar `describe_everything` com resposta ilimitada; usar operações pequenas e pagináveis.

### 4.3 Segurança MCP

- Identificar servidor/tenant/ambiente/identidade OAuth, aprovação e allowlist de tools, versão/fingerprint de schema. Um novo tool ou schema modificado exige reclassificação.
- Hints `readOnlyHint` e `destructiveHint` não substituem autorização; proteger também ferramentas de “consulta” capazes de revelar dados sensíveis.
- `prompt injection`: instruções nos tools/resources/responses não entram em política do sistema; interpretar campos como dados e controlar egress.
- Operações mutantes no MCP só em ambiente isolado, autorização expressa, contas de teste e cenário explícito, **nunca** como curiosidade de descoberta.

## 5. Contrato de Knowledge Pack

```yaml
schema_version: 1.0.0
id: enterprise-data-tables
version: 0.1.0
license: MIT
source_url: https://example.org/allowed-source
source_revision: <hash-publico>
domain_tags: [enterprise, data-intensive]
task_tags: [search, filtering, bulk-action]
applicability:
  - Tabelas densas com usuários recorrentes e decisões baseadas em múltiplas linhas
contraindications:
  - Evitar transformar tabela operacional em cards sem prova de ganho de produtividade
rules:
  - id: dense-table-preservation
    kind: heuristic
    recommendation: Avaliar visibilidade de filtros e seleção em massa
    evidence_requirements: [task_frequency, user_role, table_actions]
    test_methods: [task_success, time_on_task, keyboard_navigation]
```

Texto do exemplo é original e ilustrativo; a implementação deve fazer verificação de licença e provenance de cada pack. Não declarar que uma recomendação heurística é teste de conformidade.

## 6. Plano de transformação e oráculos

`transformation-plan.json` exige que cada capacidade crítica:

1. tenha rota/componente/ação de destino **ou** `exception` aprovada;
2. tenha `feasibility` e dependências explícitas;
3. tenha cenário/oráculo quando houver expectativa observada ou declarada;
4. não seja marcada `validated` apenas por ter sido implementada;
5. aponte para uma decisão e evidências relevantes.

**Exemplo:** `cap_search_requests` → rota `/requests`, `RequestList`, `FilterBar`, input `query`, cenário `scenario_search_returns_filtered_results`, modo `ui_only`. Outro: `cap_finalize_batch` → `backend_change_required` quando operação atômica não existe no contrato conhecido.

## 7. Gates de maturidade como funções puras

```ts
function conceptReady(report: RunReport): boolean {
  return report.security.status === 'pass' && report.preview_available;
}
function prototypeReady(report: RunReport): boolean {
  return conceptReady(report)
    && report.build.status === 'pass'
    && (!report.design_system.strict || report.design_system.status === 'pass')
    && report.functional.critical_scenarios_total > 0
    && report.functional.critical_scenarios_passed === report.functional.critical_scenarios_total
    && report.traceability.unmapped_critical_unapproved === 0
    && report.required_checks.every(c => c.status === 'pass');
}
```

`RunReport` é conceito; implementar função correspondente em Rust com os tipos atuais. Não usar o pseudocódigo acima literalmente sem definir campos nulos, permissões, `needs_evidence` e execução incompleta.

## 8. Regras de migração e compatibilidade

- `schema_version` migra com transformador explícito, teste do antes/depois e backup; não reinterpretar dados antigos silenciosamente.
- A aplicação gerada mantém seu próprio `package.json`/lockfile. A versão do DS, do REA, do Layout, dos packs e do builder é registrada no run manifest.
- Projetos antigos do Craft sem `redesign/` continuam funcionando. O Redesign só executa quando ativado explicitamente.
- Migração `redesign` não altera `capture/` ou `investigation.cany/` do Craft diretamente; adiciona referências e material derivado.

