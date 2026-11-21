# First public release — binding strategy v1.4.0

## Product decision

The first **public** delivery is a complete, demonstrably differentiated V1 — not an MVP, single-screen demo, or artificially limited cut. Lower marginal cost of agent-written code justifies more functional ambition, but it does **not** remove integration, verification, security, maintenance, or inference cost. Slices and PRs are **internal** milestones, not public releases. The first experience should impress by outcome and be as simple as pointing an agent at this repository and an authorized application.

## Promise and boundaries

Redesign existing web applications with significant structure and experience changes, grounded in tasks and evidence, respecting design systems and preserving **known, verified behaviours**. Never claim full extraction of hidden rules, complete parity, or automatic production readiness. The product is open source, agent-first, with no paid API required; inference uses the host’s authenticated model or a supported local backend. REA and Application MCP are **V1 capabilities with selective activation**, not mandatory dependencies on every run.

## Required capabilities before first public publication

| Front | Launch criterion | Legitimate fallback |
|---|---|---|
| Core Craft | Study→Build→Check operational; upstream compatibility; headless agent execution | No fallback for the main flow |
| UX Intelligence | Versioned rich library (AI, forms, tables, density, navigation, states, content, a11y, enterprise flows); contextual retrieval; applicability/contraindication; decisions with evidence and `keep_as_is` alternatives | Local text search, no embeddings |
| Journey redesign | Structural transformation of a multi-screen journey with states; executable React/TS; responsiveness when applicable | Declare unobserved parts; do not invent backend |
| Corporate DS | Real ingestion of tokens and React components + at least one documentary path (Storybook or docs); strict/flexible policy, usage conformity, versioning | No-DS mode when the user does not provide one |
| Functional preservation | Task Model, positive/negative scenarios, independent oracles, Playwright, coverage and unknowns | Declare inconclusive; never false approval |
| Advanced investigation | REA available for JS/network/state when useful; Application MCP Bridge available with trust gateway, read-only, provenance | Flow works without those connectors |
| Quality Engine | Build, functional checks, automatic a11y + manual sample, visual evaluation, regression, security, report | Honest report when checks cannot run; critical gates do not pass |
| Agent-first / DX | README, AGENTS, CLI/MCP, doctor, assisted setup, no duplicate agent, real tests for announced agents | Do not announce untested compatibility |
| Experience | One entry instruction; progressive configuration; preview, before/after comparison, decisions, limitations, exportable artifacts | Headless CLI/report is enough; desktop not required |
| OSS / distribution | Verified licence/attribution, SBOM, pinned dependencies, CI, docs and reproducible fixtures; distributed as source code, built on the user's machine (decision 2026-11-21) | No private corporate assets distributed; no prebuilt or signed installers |

## Impact criterion (not completeness alone)

- Pre-registered benchmark on at least three applications/fixtures of distinct natures: dense enterprise form, multi-step journey, and an app with a real design system; include error and permission states. Final sample size, repetitions, and thresholds must be fixed **before** execution — not invented retrospectively.
- A/B/C comparison: original Craft; Craft + good UX instructions; Redesign Anything. Same model, permissions, task, and comparable budget. Blind human evaluation by tasks and rubric; measure effectiveness, errors, DS consistency, functional coverage, accessibility, human effort, and tokens/time. Do not use only an LLM as judge.
- *Decision 2026-11-21 (Thiago Boschese): no human evaluation; the product computes the rubric from mechanical measures and no LLM judges results. See `BENCHMARK_PROTOCOL.md`, addendum 2026-11-21.*
- V1 must show **relevant gain against B**, without critical function or security regression. If there is no gain, fix before public release; do not rebrand a demo as V1.
- Primary demo: authorized corporate web app with a full journey, corporate DS, executable before/after, decision evidence, and check report. A second demo without DS proves generality.

## Distribution decision (2026-11-21)

Thiago Boschese decided (chat, 2026-11-21): **the first public release is distributed as source
code only.** People (or their agents) clone the repository and build it on their own machine
(`cargo build` / `cargo install`), which needs no code-signing certificate: an operating system only
warns about programs downloaded already built. No prebuilt or signed installer is part of V1;
signed installers are out of its scope and its gate, and any later one is a separate decision.

## Architecture policy

**Reuse before build.** Craft remains the reverse-engineering and rebuild engine; REA is a selective extension, not a Study replacement; DS CLI/Storybook and UX libraries are sources/adapters. One active orchestrator (`host_agent` XOR `craft_runner`). Knowledge packs may be large on disk and minimal in context. Do not create a mandatory multi-agent pipeline or a central SaaS. V1 integrations must be implemented, tested, and available; activation by need is optimization, not scope deferral.

## Development organization

Six fronts may advance in parallel with continuous integration: (1) Core/Agent-first/REA/MCP; (2) Design Intelligence and Task Model; (3) Corporate DS; (4) generation and journeys; (5) Quality/security; (6) DX/distribution. Slices 0–5 in `EXECUTION_V12.md` are **internal checkpoints** to reduce rework and catch regressions. They do not authorize a partial public release. v1.3 blocking criteria and trust boundaries remain fully in force.

## Do not

- Do not ship a V1 that only redesigns one screen, without DS and without optional investigation available.
- Do not require REA, Application MCP, or DS for every task.
- Do not deepen the agent without measuring benefit.
- Do not confuse a pretty UI with productivity, usability, or parity.
- Do not hide unknowns or weaken security controls to meet a schedule.
- Do not declare an integration/compatibility implemented before real tests.

## Precedence

This document supersedes any earlier recommendation of a **reduced MVP, P0-minimal, or slice-based public release** in `SPEC.md`, `EXECUTION_V12.md`, `IMPLEMENTATION_PLAN.md`, `COUNCIL_DECISIONS.md`, and `COUNCIL_ROUND_2.md`. It fully preserves normative requirements for security, oracles, workspace, cost, and quality. On conflict, security and evidence honesty prevail over impact criteria.
