# Redesign Anything — Contracts, schemas and policies

> **Status:** v1 proposals; they are not yet official Craft/REA/Layout APIs. Check the types that already exist in the fork's checkout before introducing duplicate structures. The real JSON Schemas in `schemas/` are the reference for persistence validation. Illustrative examples use a generic domain, with no real data.

## 1. Cross-cutting conventions

- `schema_version`: SemVer string (`1.0.0`). `project_id`: stable slug. `revision`: monotonic integer; atomic writes.
- IDs: prefixes `ev_`, `cap_`, `rule_`, `scenario_`, `decision_`, `ds_`, `check_`; avoid collisions even when input came from several providers.
- Project JSON carries no tokens, cookies, secrets, passwords, URLs with credentials or sensitive bodies. Credentials are references to secret storage.
- All `source_refs` are pointers to normalized evidence. `raw_source` is external, optional and local-only as authorized.
- The LLM payload is always a small/sanitized *projection* of these contracts, not an unrestricted dump of captures.
- Never invent metrics. `null` or `not_evaluated` is different from `0` or from success.

## 2. Fundamental enums and types (reference TypeScript)

```ts
type ProjectId = string;
type EvidenceId = `ev_${string}`;
type CapabilityId = `cap_${string}`;
type RuleId = `rule_${string}`;
type ScenarioId = `scenario_${string}`;

type CaptureMode = 'clean_room' | 'deep';
// refresh: organize each screen, same blocks/journeys/look · redesign (default): reorganize journeys · reimagine: new flows.
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

// Values with explicit units; all optional and honored only when
// observable/controllable by the executor. Never infer token billing from processed tokens.
interface ExecutionBudgets {
  token_soft_limit?: number; // warning; not a universal hard cap over Cursor/Claude/Codex
  max_mcp_calls?: number; // calls made by the app MCP Gateway
  max_rea_invocations?: number; // calls through the REA Adapter
  max_critic_passes?: number; // reviews orchestrated by Redesign
  max_tool_result_chars?: number; // characters of summarized response; full evidence is paginable
  max_run_minutes?: number; // supervised processes; does not terminate an external host
  no_progress_limit?: number; // steps without progress under local control
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
  source: string; // Craft runner, host agent, backend API, etc.
  accounting_quality: AccountingQuality;
  input_tokens_reported: number | null; // when reported, includes cached if applicable
  output_tokens_reported: number | null;
  cached_input_tokens_reported: number | null; // subset, do not add again
  tokens_estimated: number | null;
  cost_amount: number | null; // null if not measurable
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
  source_operation: string; // e.g. Craft tool ID, REA operation or MCP tool ID
  subject: string; // page slug, capability or operation
  kind: 'ui' | 'behavior' | 'business_rule' | 'api' | 'design_system' | 'test' | 'user_context';
  status: KnowledgeStatus;
  scope: { target_origin?: string; route?: string; role?: string; environment?: string; app_revision?: string };
  locator?: { artifact_path?: string; line?: number; step_id?: string; external_evidence_id?: string };
  digest?: string;
  observed_at: string;
  limitations: string[];
  sensitivity: Sensitivity;
  summary: string; // already sanitized
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
  props_schema: Record<string, unknown> | null; // JSON Schema of props or null
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

## 3. Project configuration

Configuration contract: `schemas/redesign-config.schema.json`. Persisted telemetry: `schemas/usage-snapshot.schema.json`. **The documentation version 1.1.0 and the JSON Schemas version 1.0.0 are deliberately distinct:** the changes add optional fields and do not break existing documents; before publishing a fork, define a formal migration policy.

Required fields: `schema_version`, `project_id`, `target`, `mode`, `capture_mode`, `scope`, `design_system`, `providers`, `backend`, `model`, `privacy`, `checks`. `execution` is optional (compatible with the v1.0.0 contract), with defaults that depend on the entry point: external agent → `host_agent`; Craft UI/CLI Build → `craft_runner`; profile `lean` in both. `target` describes only the data origin and does not authorize mutation. A web origin must be HTTP(S). Screenshot captures without a URL are possible, but functionality cannot receive an `observed` status from execution without a test.

**Conflict validation:**

- `model_egress: local_only` + a demonstrably remote model backend → block and ask for a conscious choice.
- `capture_mode: clean_room` + REA JS decompilation/source export → block and recommend Deep with approval.
- `design_system.policy: strict` + registry unavailable → block Build; Study may proceed.
- `allow_production_writes: false` + mutating MCP → deny the call; this does not depend on `readOnlyHint`.
- Final/redirected URL outside authorized origins → refuse investigation of the new origin.
- `execution.controller=host_agent` + `model.use_craft_selected_backend=true` → invalid configuration; the host agent controls its own model.
- `execution.controller=host_agent` + implicit attempt at `rebuild_start`/nested model → `controller_conflict`; handoff only with explicit approval.
- `execution.profile=deep` never changes `capture_mode=clean_room` on its own; REA that reads internal JS requires Deep consent.
- `token_soft_limit` exceeded, but without telemetry/control of the host agent → `warning_unenforceable`; do not claim a token block.
- `max_mcp_calls` / `max_rea_invocations` → hard cap only for calls that pass through the Redesign gateway; durable count per run.

**Semantics of metrics and states:** `COST_AND_EXECUTION.md`. A UsageSnapshot must indicate missing data with `null`/`unavailable`, not `0`. `cached_input_tokens_reported` is normally already part of `input_tokens_reported`; do not double the count. No cost field is inferred from the host's subscription.

## 4. Application Knowledge Contract (AKC) — optional and not an MCP standard

This contract is a **Redesign Anything convention**, not a mandatory part of MCP. An application's MCP server may implement it or map existing tools.

### 4.1 Suggested tools

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

Uses: answer technical questions, not obtain credentials or execute operations. Responses must indicate `capability_id`, `version`, `environment`, `role_context`, `rule_ids`, `source_refs`, `completeness` and `limitations` when available.

### 4.2 Example of a semantic response

```json
{
  "schema_version": "1.0.0",
  "application": "example-requests",
  "environment": "staging",
  "app_version": "2026.11.4",
  "capability_id": "cap_review_request",
  "title": "Review request",
  "required_inputs": ["request_id", "review_decision"],
  "allowed_roles": ["approver"],
  "from_states": ["pending_review"],
  "outcomes": ["approved", "rejected"],
  "limitations": ["The external approval policy is not documented in this contract"],
  "completeness": "partial"
}
```

This response is `declared`; it becomes `validated` only with approved tests/oracles. Avoid `describe_everything` with an unbounded response; use small, paginable operations.

### 4.3 MCP security

- Identify the server/tenant/environment/OAuth identity, tool approval and allowlist, schema version/fingerprint. A new tool or a modified schema requires reclassification.
- The `readOnlyHint` and `destructiveHint` hints do not replace authorization; also protect "query" tools capable of revealing sensitive data.
- `prompt injection`: instructions in tools/resources/responses do not enter system policy; interpret fields as data and control egress.
- Mutating operations in MCP only in an isolated environment, with express authorization, test accounts and an explicit scenario, **never** as discovery curiosity.

## 5. Knowledge Pack contract

```yaml
schema_version: 1.0.0
id: enterprise-data-tables
version: 0.1.0
license: MIT
source_url: https://example.org/allowed-source
source_revision: <public-hash>
domain_tags: [enterprise, data-intensive]
task_tags: [search, filtering, bulk-action]
applicability:
  - Dense tables with recurring users and decisions based on multiple rows
contraindications:
  - Avoid turning an operational table into cards without proof of a productivity gain
rules:
  - id: dense-table-preservation
    kind: heuristic
    recommendation: Evaluate the visibility of filters and bulk selection
    evidence_requirements: [task_frequency, user_role, table_actions]
    test_methods: [task_success, time_on_task, keyboard_navigation]
```

The example text is original and illustrative; the implementation must verify the license and provenance of each pack. Do not claim that a heuristic recommendation is a conformance test.

## 6. Transformation plan and oracles

`transformation-plan.json` requires that each critical capability:

1. has a target route/component/action **or** an approved `exception`;
2. has `feasibility` and explicit dependencies;
3. has a scenario/oracle when there is an observed or declared expectation;
4. is not marked `validated` merely for having been implemented;
5. points to a decision and relevant evidence.

**Example:** `cap_search_requests` → route `/requests`, `RequestList`, `FilterBar`, input `query`, scenario `scenario_search_returns_filtered_results`, mode `ui_only`. Another: `cap_finalize_batch` → `backend_change_required` when an atomic operation does not exist in the known contract.

## 7. Maturity gates as pure functions

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

`RunReport` is a concept; implement the corresponding function in Rust with the current types. Do not use the pseudocode above literally without defining null fields, permissions, `needs_evidence` and incomplete execution.

## 8. Migration and compatibility rules

- `schema_version` migrates with an explicit transformer, a before/after test and a backup; do not silently reinterpret old data.
- The generated application keeps its own `package.json`/lockfile. The version of the DS, REA, Layout, packs and builder is recorded in the run manifest.
- Old Craft projects without `redesign/` keep working. Redesign runs only when explicitly activated.
- A `redesign` migration does not directly alter Craft's `capture/` or `investigation.cany/`; it adds references and derived material.
