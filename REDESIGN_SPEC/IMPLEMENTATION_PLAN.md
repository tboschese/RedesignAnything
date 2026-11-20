# Redesign Anything — Executable implementation plan (detailed backlog v1.1; order superseded in v1.2)

> **DECISION v1.4:** `V1_FIRST_RELEASE.md` defines the mandatory scope of the first public release. `EXECUTION_V12.md` organizes internal checkpoints; PRs are not releases. Essential checks remain brought forward, but all fronts must converge before publication. Read `BENCHMARK_PROTOCOL.md`.

> **Read first:** `SPEC.md`, `CONTRACTS.md`, `AGENTS_REDESIGN.md`. This plan describes PRs for an **independent fork** of Craft; it is not an upstream request. No PR/branch was created in this analysis. **Existing file paths were checked against the documentation/code; new ones are proposals and require inspection of the checkout.**

## 0. Strategy and sequence

**Goal of the first cycle:** on a fixture URL, demonstrate Craft's `Study → Plan → Build → Check → Ready` with a React/TS redesign and a real DS; coverage of a known journey and limit reports. **Do not** start with REA or multiple MCPs before proving this core.

**P0 consumption control:** from PR-00 establish `host_agent` versus `craft_runner`, `lean` as default and no nested builder invocation. PR-02 incorporates `execution` fields into the contract; PR-03/04 apply selective retrieval; PR-08 creates gates and measurements. Mandatory reference: `COST_AND_EXECUTION.md`.

**Order to reach value:** PR-00 → PR-01 → PR-02 → PR-03 → PR-04 → PR-05 → PR-06 → PR-07 → PR-08. Advanced integrations come later. Each PR is small, with tests, docs, versioned schema changes and Craft regression.

### Local preparation (reference commands, validate in the checkout)

```bash
git clone https://github.com/storytold/craft-anything redesign-anything
cd redesign-anything
git remote rename origin upstream
# Create your remote fork on GitHub and add it; do NOT push to upstream.
git checkout -b feature/redesign-bootstrap
rustc --version
cargo --version
cargo test --workspace
cargo xtask layers
cargo xtask secrets
```

The README consulted requires Rust 1.97+ and OS-specific dependencies. **Do not consider the baseline green until these commands are executed and the platform/commit recorded.** Read `AGENTS.md`, `docs/pitfalls.md`, `docs/extending.md` and `docs/ROADMAP.md` before editing. The original `AGENTS.md` states that only roadmap items may be developed and that other reverse-engineered products must not be named in code; in an independent fork, **consciously update those rules and the roadmap** for the new product, respecting credits, licenses and security principles. Do not use the fork as an upstream contribution without following upstream policies.

## PR-00 — Foundation, baseline and agent-first use from the repo URL

**Files:** `AGENTS.md`, `docs/ROADMAP.md`, `docs/STATUS.md`, `README.md`, `THIRD_PARTY_NOTICES.md`, architecture documentation; existing CI. No engine changes.

**Tasks:** define a baseline test of tokens/rounds in the same agent (or `unavailable` if not measurable), make the two controller modes explicit, `lean` default and anti-loop rules; create the fork, mark the upstream commit, define independent naming/purpose and attribution; record the Rust/OS version, pass the tests; create original fixtures; define a code of conduct and distribution; preserve Craft legacies. Install the secrets hook per Craft's AGENTS. **Document the agent-first flow starting from the GitHub URL**, a quickstart without UI, its own AGENTS.md, `doctor`/prerequisites and setup security; README with prompts for agents. The repository must work as an entry point for Cursor/Claude/Codex without requiring the desktop application.

**Acceptance:** baseline compiles; upstream suite green or failures documented; no silent change to the original Craft; permissions/licenses and origin documented. The agent-first guide lets an agent discover dependencies without silently running installs. **Do not rebrand everything in this PR**.

## PR-01 — `RedesignKind` in the real Build mechanism

**Inspect:** `crates/ai/src/rebuild/kinds/mod.rs`, `website.rs`, `crates/ai/src/rebuild/mod.rs`, `crates/ai/src/rebuild/brief.rs`, `apps/craft-anything-cli` and registries.

**Implement:** `host_agent`/`craft_runner` dispatcher and `controller_conflict` blocking for implicit `rebuild_start`; a new `TargetKind` with id `redesign`, **explicit** selection by configuration (not just ambiguous words), React/TS default, pixel similarity is not a redesign gate. Integrate into `KindRegistry`; create a minimal `RedesignBrief` that instructs preservation of known capabilities and displays `unknowns`. Create a minimal headless command that creates a project and lists the steps.

**Tests:** explicit detection, an old project remains `website`/`app`, `Study` remains the same, `kind=redesign` explicitly selected, no visual-clone checker in the gate. Coverage of configuration errors.

**Acceptance:** a new project reaches Study and Build with Craft's runner without changing the behavior of the existing `website` kind. **Without desktop UI:** a CLI command/MCP flow can create, inspect, continue and list project status. Smoke test in the agent with a clean checkout.

## PR-02 — Versioned workspace and contracts

**Proposed files:** `crates/engine/src/tools/redesign_*` or an equivalent module, persistence owner in the appropriate layer, the kit's `schemas/*.json` (synchronized with the Rust schema), migration and I/O tests.

**Tasks:** incorporate `execution`/`budgets` into the schema, durable call counters at the gateway, `reported/estimated/unavailable` token reports, migration; create the `redesign/` directory, versioned files, `EvidenceRecord`, `Capability`, `Coverage`, `Decision`, `TransformationPlan`, `CheckResult`. Stable IDs, migrations, backup and atomic writes. Validate hostile inputs, symbolic links, directory traversal and large files.

**Tests:** roundtrip, invalid schema, missing v0, migration, preserved unknowns, absence of secrets, symlink traversal refused, restart keeps state.

**Acceptance:** create/open a Redesign project with typed data without breaking a pre-existing Craft project; layer/wasm checks when applicable.

## PR-03 — Study adapter / capability mapper / coverage

**Reuse:** `web_site_capture`, `web_page_states_capture`, `web_walkthrough_record/check`, `core_evidence_*` and `investigation.cany`. Do not create a new browser crawler without proving a gap.

**Tasks:** limit the evidence payload and paginate before responding to the agent, record the marginal cost of Study; convert Craft evidence to a semantic model; map routes, actions, fields, states and observable rules; show absence of profiles and coverage gaps. Add `redesign_capabilities_list` and `redesign_coverage_show` with pagination. Track `sourceRefs` per capability.

**Tests:** a fixture with a visible rule and another purposely hidden; the hidden rule stays `unknown`, the known capability preserves its origin; no `100%` when the denominator is unknown. Manual login if necessary, without exported cookies.

**Acceptance:** a reproducible Study report of a journey, including gaps, without claiming complete discovery.

## PR-04 — UX Knowledge Packs + retrieval + Critic

**Tasks:** versioned and licensed pack format, local task-oriented retrieval (tags + text; 3–8 initial rules as a heuristic), without injecting the full pack; an initial minimal catalog for tables, filters, forms, navigation, accessibility and density; apply contraindications. The Design Critic produces referenced recommendations; `keep_as_is` mandatory.

**Tests:** a pack without license/version rejected; an incompatible rule not applied; a tendency toward wizard/card is not invariable; textual prompt injection in a pack does not modify tools/authorization; fallback without embeddings.

**Acceptance:** the report explains 3 options (including preserving) for a dense screen, with trade-offs and sources; no invented requirements.

## PR-05 — Strict Design System Registry + optional Layout CLI

**Dependencies:** PR-02 models/schemas; PR-08 criteria can start as provider unit tests.

**Tasks:** ingestion of package.json/exports of a fixture React/TS package; DTCG tokens; Registry with paths, version, props, variants and constraints; Layout CLI MCP integration via adapter; own compliance using imports/tokens and typecheck, never trusting the MCP alone. Configure `strict/advisory` and approved overrides.

**Tests:** import of a real component passes; a fake import fails; a nonexistent prop fails; hardcoded values prohibited in strict fail; an incompatible package version fails; missing Storybook does not block a valid repo import; Layout provider down degrades without masking a strict requirement.

**Acceptance:** generate with the fixture DS without duplicated components and pass a real contract check.

## PR-06 — Design Decision + Feasibility + Traceability

**Tasks:** `diagnose/retrieve/alternatives/choose/plan`; classification `ui_only`, `frontend_orchestration`, `backend_change_required`, `not_determined`; impact and preservation; decisions approvable in Craft's native Inbox. Export a typed transformation plan.

**Tests:** combining 3 screens without an atomic endpoint produces a backend dependency; a scenario with an existing backend admits orchestration with risks; a divergent decision requires approval; a critical capability without a destination blocks the check.

**Acceptance:** the report plans a new journey without inventing an endpoint/atomicity and with complete traceability of the known scope.

## PR-07 — Functional builder with safe mock

**Tasks:** do not start a nested runner in `host_agent` mode; create a compact brief in `RedesignKind`'s `brief_sections`, pass decisions/DS/capabilities by reference; build React/TS+Vite with synthetic mocks; preserve route/action traceability and states. Reuse Craft's builder; do not copy a private design system or logo.

**Tests:** React/TS build, UI events, modal, form validation, table, states; CSP, secrets, routes; source project intact; refusal if strict DS does not exist; an agent without enough capability to finish returns a clear failure.

**Acceptance:** a functional prototype in `app/`, with data marked as simulated and no write calls to the origin.

## PR-08 — Native Redesign Checks + Ready Gates

**Critical:** do not depend on the upstream WASM `PluginJudge` for blocking checks, since a plugin failure can be `passed=true`. Create native Rust checks registered in the `TargetKind` or fix the semantics under test.

**Tasks:** build/typecheck, DS, Playwright scenarios, axe-core, traceability, security, impact and Criticality. Add hard gates for calls/REA/critic and reports that do not equate `unavailable` with zero tokens. States `pass/fail/warn/not_evaluated/blocked`; gates for the three Ready levels. Result paginator; UX Critic separate.

**Tests:** a check that returns an error/timeout **does not pass**; a beautiful screenshot but a non-working button → fail; a case with zero oracles does not become `prototype_ready`; a failing critical accessibility check prevents qualification, but do not declare WCAG certification after approval.

**Acceptance:** a fixture with a regression detects, fixes and revalidates; code with a secret source is not exported; the reporter explains what was not tested.

## PR-09 — Application MCP Bridge + Trust Gateway

**Reuse:** Craft's MCP plugin (stdio/HTTP, consent and Evidence). **Do not** create a second MCP client indiscriminately.

**Tasks:** `application_mcp` profile, catalog/hash of tool schemas, allowlist, effect classification, `read-only` query, version and permissions; optional AKC; reconciliation of information and conflicts; approval for a changed tool; output/egress limits.

**Tests:** a server with a tool lying about readOnly, a malicious tool description, schema change and a token in the body; all blocked/sanitized; a safe `get_rule` becomes `declared`, not `observed`; the experience works without MCP.

**Acceptance:** the user connects a fixture MCP and receives a contextualized functional explanation without additional implicit permissions.

## PR-10 — REA Deep Adapter + value measurement

**Tasks:** Node/REA discovery/version, start only with consent, trigger and budget; record the reason for each call and the incremental evidence gain; convert Evidence, limitations and coverage; `analyze_web_bundle`, `trace_javascript_semantics`, `inspect_web_network_capture` in the first batch; Deep-gated. `environment-build`/stateful mock only with incremental proof and sanitization.

**Tests:** Deep allowed, Clean Room blocked, dedicated browser profile, REA absent fallback, partial timeout, incompatible source, egress and canary token; compare Craft vs Craft+REA in the benchmark.

**Acceptance:** the integration adds at least one correct, tested fact on a fixture where Craft alone did not obtain it under the same conditions, or remains experimental/opt-in without blocking the internal checkpoint; however, the advanced investigation capability must be available and tested before the V1 release.

## PR-11 — Benchmark, security and large flows

**Tasks:** controlled benchmark of base Craft vs Redesign Lean vs Redesign Deep, measuring models/controllers, input/output/cache tokens (if reported), calls and payload size, time and quality, including the no-telemetry case; fixtures with 2 profiles, multiple routes, states, inconsistent docs, async API, broken DS, prompt injection, locale and mobile. Before/after suite, coverage/precision/time/cost measurements.

**Tests:** `VALIDATION_MATRIX.md`; repeatable regressions; neither agent fallback nor REA trace may cause stronger claims without new tests.

**Acceptance:** results published with environment, version, fields without coverage and limits; no invented KPI value.

## PR-12 — Final UX, packaging and release

**Tasks:** Redesign selection screens, DS and MCP without a new canvas; report in the workspace; branding and licenses; cross-platform installers with optional sidecar; SBOM; initial PT-BR/EN docs; upgrade and rollback test.

**Acceptance:** clean installation on the target OS, useful first run, offline CLI use for a local fixture, preview and report opening; license and security documented.

---

## General Definition of Done

- Run the commands defined in the real `AGENTS.md` (including `cargo fmt --all -- --check`, `cargo test --workspace`, clippy, `cargo xtask layers`, `cargo xtask locks`, `cargo xtask secrets`, wasm if touching L0–L2, i18n checks if pertinent).
- Evidence of real execution in the PR, with OS/toolchain/model versions and reproducible artifacts.
- Prove the old Craft flow without regression; unit tests, contract tests, security negatives and the P0 pipeline.
- Do not leave an action/timeout/secret as `pass`; unknowns not omitted; new files versioned/migratable.
- Lint/build of the generated React and E2E tests with a known fixture.

## First prompt for Cursor

Copy this command **after creating the fork and adding this package's files to the repository**:

```text
You are implementing Redesign Anything as a fork of Craft Anything. First read AGENTS.md, docs/pitfalls.md, docs/extending.md, REDESIGN_SPEC/SPEC.md, REDESIGN_SPEC/CONTRACTS.md, REDESIGN_SPEC/IMPLEMENTATION_PLAN.md, REDESIGN_SPEC/VALIDATION_MATRIX.md and the real code in crates/ai/src/rebuild/kinds. Do NOT rewrite Craft, do NOT integrate REA/Studio now, do NOT change the UI before a CLI exists. Define `host_agent` as this session's controller, `lean` as the default and prohibit nested builders. Do not use an estimated token limit as if it were a hard cap. Execute only PR-00 (baseline and fork rules) and PR-01 (new minimal RedesignKind with explicit selection). Preserve Clean Room, consent, layers, never-crash, tool/evidence contracts, original code and all upstream tests. Before coding, describe the precise changes and the acceptance test; then implement, run verifications and report the real results and limits. Do not push, do not edit the remote repository and do not claim tests that were not executed.
```

## Addendum v1.3.0 — blocking tasks from Council II

**PR-00:** add `TRUST_BOUNDARIES.md`, `ORACLE_POLICY.md`, `SAFE_WORKSPACE_POLICY.md`, `OSS_DX_AND_UPSTREAM.md`, upstream SHA pin, license/attribution, planned compatibility matrix, side-effect-free `doctor`.

**PR-01 / first vertical slice:** isolated output; approval before applying a patch; independent oracle policy; fixture scenario with observed/contracted expected result; checks fail closed; evidence sanitization.

**PR-02:** schema for `oracle_type`, `approval_record`, `project_scope`, `evidence_retention`, `source_hash`, `workspace_output_mode`; implement migrations and negative tests. Do not declare fields as implemented until real Rust integration.

**PR-03–08:** Task Model with unknowns, content design, pre-registered benchmark and blind evaluation, pinned DS, manual accessibility on a sample.

**PR-09–12:** REA/MCP only opt-in and with controls; upstream CI, multi-agent and multi-OS; corporate asset distribution policy.

**Review gate:** a PR that modifies existing sources, runs third-party scripts or creates an expected result by inference cannot pass without the corresponding controls.
