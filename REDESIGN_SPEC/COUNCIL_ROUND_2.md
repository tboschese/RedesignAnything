# Council II — Adversarial review of the v1.2.0 spec

**Date:** 2026-11-19. **Nature:** structured simulation of eight professional perspectives; not the participation of real experts or independent models. **Available evidence:** documentation/spec and references to the repositories; no integrated benchmark was executed.

## Participants

1. Software architecture: modularity, upstream compatibility, fork correctness.
2. AI engineering: baselines, inference, evaluations and context control.
3. Design/HCI: tasks, accessibility, content, research and operational quality.
4. Product Management: positioning, scope, adoption, metrics and strategy.
5. Security and privacy (new): credentials, isolation, supply chain, MCP and authorization.
6. QA and reliability (new): oracles, false positives, regressions and reproducibility.
7. Developer Experience and OSS (new): installation, CI, support, documentation, licenses and distribution.
8. Enterprise operations specialist (new): roles, transactions, processes, data and real-world impact.

## Adversarial round — critiques and responses

### Architecture
**Objection:** Even v1.2 keeps contracts and module names that may not map stably onto Craft's extension points. `TargetKind` alone does not guarantee CLI/MCP/desktop, persistence and gating. **Response:** Slice 0 must pin the SHA and map real integrations; Slice 1 passes only with a headless E2E. **Veto:** deep changes to the core without an ADR comparing fork and extension.

### AI
**Objection:** A/B/C are not comparable if B gets worse instructions than C, if the model changes, or if evaluators know the origin. **Response:** pre-registered rubric, same budget and a strong B prompt, blind evaluation, multiple runs and variance recording. **Veto:** a superiority claim based only on LLM-as-judge.

### Design/HCI
**Objection:** `Task Model` is necessary, but frequency/criticality are often unknown. A cleaner screen can be worse for experienced users. **Response:** `unknown` is legitimate; preserve the original alternative; include keyboard, density, batch tasks, states and content. **Veto:** approving a redesign based on aesthetics, screenshots or an isolated heuristic score.

### Product
**Objection:** "Any web application" is too ambitious; value for a designer and for a modernizer are distinct jobs. **Response:** first target user = a developer or technical designer who needs to redesign an existing web journey with verifiable code. A full modernization product is a horizon, not a P0 promise. **Veto:** scope expansion before proof of differentiation.

### Security and privacy
**Objection:** `host_agent` is controlled by another process; REA cannot by itself guarantee that the external agent will not run commands or send captures to a remote model. **Response:** separate controls *enforceable by REA* from guidance to the host; do not claim a complete sandbox; per-project isolation, explicit authorization and data review. **Veto:** executing scripts from untrusted sources, exporting cookies/tokens, or writing to production without approval.

### QA and reliability
**Objection:** `Prototype Ready` can be falsely approved if tests cover only the known flows and the report omits the unknowns. **Response:** the gate requires an explicit denominator and scope; a check result with error/timeout = failure/inconclusive; oracles independent of the generator's own hypotheses. **Veto:** any `passed=true` without verifiable execution.

### DX/OSS
**Objection:** The "pass the URL" experience will not be the same in Cursor, Claude and Codex if it requires Rust, Node, browsers, permissions and MCP configuration. **Response:** staged bootstrap with `doctor`, alternative paths and explicit setup; test a clean environment per agent/OS; distinguish the core license from each integration's license. **Veto:** claiming universal automatic installation or zero inference cost.

### Enterprise operations
**Objection:** Consolidating five screens may require atomicity, auditing and specific permissions. A mock prototype does not demonstrate business equivalence. **Response:** classify changes as UI-only/orchestration/backend-required; use semantic MCP only as a source, with cross-verification; surface unknown rules and forbid inferring transactions. **Veto:** `Engineering Ready` for critical flows without a contract or a trustworthy oracle.

## Debates between members

**Product vs. HCI:** Product argues for shipping a single screen quickly; HCI requires at least one complete task. **Reconciliation:** one small journey, not an isolated screenshot.

**AI vs. Architecture:** AI wants to keep the Design Engine flexible; Architecture wants stable contracts. **Reconciliation:** contracts for evidence and results, not for every internal reasoning step.

**DX vs. Security:** DX wants frictionless bootstrap; Security vetoes automatic scripts. **Reconciliation:** automatic diagnosis; installations run only with consent and logging.

**QA vs. Product:** Product wants a fast release; QA requires independent oracles. **Reconciliation:** Concept Ready may ship with hypotheses; Prototype Ready requires the known critical scenarios to be tested; Engineering Ready requires integration and explicit scope.

## New findings — gaps still open in v1.2.0

1. **Out-of-process enforcement:** the tool does not control all commands of the host agent; it needs trust-boundary documentation and host-specific tests.
2. **Benchmark without a closed sampling protocol:** the number of fixtures/repetitions and thresholds must be pre-registered before declaring a gain.
3. **Source of truth for oracles:** do not generate tests and expected results exclusively from the redesign's own hypothesis; prefer observed behavior, a contract or human confirmation.
4. **Editing an existing project vs. new generation:** risk of overwriting the client's code and routes; P0 must generate into a separate workspace and require approval to apply patches.
5. **Reproducibility of private design systems:** packages, tokens and Storybook can change; record hash/version, and avoid redistributing proprietary assets.
6. **Fork compatibility with upstream:** establish a cherry-pick/merge policy, regression CI and an ADR if the delta grows.
7. **Accessibility beyond axe:** keyboard-only, focus, screen reader on a sample and high zoom; declare limitations.
8. **Prompt and evidence security:** untrusted sources, retention, TTL, redaction and a ban on exporting sessions.

## Simulated vote

- Architecture: **conditionally approves**, subject to the spike and core isolation.
- AI: **conditionally approves**, subject to a benchmark against a strong B baseline.
- Design/HCI: **conditionally approves**, subject to the Task Model and per-journey testing.
- Product: **conditionally approves**, subject to positioning and a restricted P0.
- Security: **conditionally approves**, subject to authorization, data and supply-chain controls.
- QA: **conditionally approves**, subject to independent oracles and fail-closed behavior.
- DX/OSS: **conditionally approves**, subject to honest bootstrap and multi-environment CI.
- Enterprise domain: **conditionally approves**, subject to explicit transactional limits.

**Decision: GO for Slice 0 and Slice 1; NO-GO for claims of full parity, sensitive enterprise use without tested controls, or an Engineering Ready release without evidence.** There is no empirical quality score and no reliable estimate of UX gain.

## Recommended changes for the next revision

1. Create `TRUST_BOUNDARIES.md` with the real controls of the REA process vs. the host agent's responsibilities.
2. Create `ORACLE_POLICY.md` and require independence of expected results.
3. Record a policy of separate output and patch application upon approval.
4. Fix the benchmark protocol before execution, including sampling and thresholds.
5. Create a DX and license checklist per operating system and agent.

These changes are recommendations; they were not implemented or tested in this round.
