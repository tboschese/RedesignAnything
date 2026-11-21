# Changelog — Redesign Anything specification

## Documentation 1.4.8 — 2026-11-20

- Critical scenarios can be judged by a journey run in the built prototype (`redesign_journeys_run`) instead of a word hint. See `docs/PR17_JOURNEYS.md`. No contract or schema change: the gate in CONTRACTS.md §7 still counts passed scenarios.

## Documentation 1.4.7 — 2026-11-20

- Prototype Ready's `build.status === 'pass'` (CONTRACTS.md §7) now means a built site that is as new as the app's sources, not a `package.json`. No contract or schema change.

## Documentation 1.4.6 — 2026-11-20

- `SPEC.md` scope: `reimagine` may propose a new visual design as well as new flows. The intentional identity traits stay, kept or evolved.

## Documentation 1.4.5 — 2026-11-20

- Likeness to the original is measured per page and reported, not judged against a limit; variants of one study are compared by order. See `docs/PR16_LIKENESS.md`. No contract or schema change.

## Documentation 1.4.4 — 2026-11-20

- `SPEC.md` scope: one version by default; on request, one variant project per mode from the same study.

## Documentation 1.4.3 — 2026-11-20

- `CONTRACTS.md` §7: `conceptReady` requires `identity_essence` not to fail, and `prototypeReady` requires `identity_drift` not to fail. No redesign may lose the target's intentional identity traits without the operator's recorded approval.

## Documentation 1.4.2 — 2026-11-20

- `SPEC.md` scope: `refresh` now means organizing each screen with the same blocks, journeys and look (it read "visual, minimal functional change"). The plan's mode is validated and compared with the plan by the advisory `mode_scope` check.
- `CONTRACTS.md`: comment on `RedesignMode`. No schema change: the mode enum was already in the schemas.

## Documentation 1.4.1 — 2026-11-20

- Renamed product nickname **V1 Impactante** → **First public release**; binding doc is now `V1_FIRST_RELEASE.md` (removed `V1_IMPACTANTE.md`).
- Root `README.md` and entry docs (`REDESIGN_SPEC/README.md`, `AGENT_FIRST.md`, `AGENTS_REDESIGN.md`, `docs/V1_GAP.md`) rewritten in **English**. Chat with maintainers may remain in Portuguese; repository text is English.
- Cross-references updated across the kit and `docs/`.

## Documentation 1.2.0 — 2026-11-19

- `COUNCIL_ROUND_2.md` adds a second critical review with eight perspectives, divergences, conditional votes, gaps and a GO/NO-GO decision for the initial slices.
- Simulated council of software, AI, design and product consolidated; security, QA, DX/OSS and enterprise-domain perspectives added in the following review.
- `SPEC.md` receives binding addendum v1.2: Task Model, measurable improvement, design-system policies in three categories, early checks, early stopping and simplification by evidence.
- `EXECUTION_V12.md` replaces the previous PR order with vertical slices, keeping the detailed historical backlog.
- `BENCHMARK_PROTOCOL.md` defines arms A/B/C/D, blind evaluation, experimental control and stopping criteria.
- `COUNCIL_DECISIONS.md` records decisions and unproven hypotheses.
- `VALIDATION_MATRIX.md` gains T-045 to T-056.
- JSON Schemas 1.0.0 kept for compatibility; the initial Task Model is optional until implementation/formal migration.
- **No integration tests or benchmarks were executed.**

## Documentation 1.1.0 — 2026-11-19

Update motivated by the analysis of agent-execution feasibility and token estimates.

- Introduces `COST_AND_EXECUTION.md` with a single controller (`host_agent` / `craft_runner`), Lean/Deep modes, on-demand retrieval, anti-loop rules, per-adapter limits and honest telemetry.
- Adds an optional `execution` block to `redesign-config.schema.json`, without making the 1.0.0 persistence contract incompatible. Requires `allow_nested_builder=false`; in `host_agent`, `model.use_craft_selected_backend=false`.
- Adds `usage-snapshot.schema.json`, with input/output/cache data, `reported`/`estimated`/`unavailable` measurement, cost `null` when not observable, and synthetic examples.
- Updates `SPEC.md`, `CONTRACTS.md`, `AGENT_FIRST.md`, `AGENTS_REDESIGN.md`, `IMPLEMENTATION_PLAN.md` and `VALIDATION_MATRIX.md` to make cost control and a single agent P0 requirements.
- Extends the test matrix T-031 to T-044 to cover budget, restart, fallback, cache, payloads, handoff and the Craft/Lean/Deep benchmark.
- Fixes the schema generator to use a path relative to the kit instead of a temporary execution directory.
- **No real integration test with Cursor/Craft/REA in this revision.** The nine JSON Schemas and the examples were validated against JSON Schema Draft 2020-12.

## Documentation 1.0.0 — 2026-11-19

Initial specification, Craft-derived architecture, semantic contracts, UX libraries, enterprise design system, optional REA/application MCP, security, validation, agent-first and PR-based implementation.

## v1.3.0 — normative incorporation of Council II (2026-11-19)

- Four normative policies: trust boundaries, independent oracles, safe workspace/patch, DX/OSS/upstream.
- Binding addenda in SPEC, execution, backlog, benchmark, agent-first, costs and validation.
- Negative tests T-057 to T-074 defined, **not executed**.
- The `COUNCIL_ROUND_2.md` report remains as the record of the review; its critical recommendations become explicit requirements.
- No software implementation, real benchmark, GitHub creation or security approval is claimed.

## v1.4.0 — First public release

- Replaced MVP/partial release with a complete first public delivery and first public release.
- Normative document `V1_FIRST_RELEASE.md` with capabilities, impact criteria, six parallel workstreams, demo and release gate.
- Aligned README, SPEC, plan, execution, agents, benchmark, cost and matrix.
- Tests T-075..T-082 specified, not executed.
- Kept the v1.3 policies on security, parity, workspace and license.
