> **v1.4.0 — mandatory interpretation:** slices 0–5 are internal integration checkpoints, not MVPs or public releases. The V1 release requires the complete set and the gates in `V1_FIRST_RELEASE.md`. Working on parallel fronts is allowed; integrate and test continuously.

# Prioritized execution plan — v1.2.0

> **Takes precedence over the historical PR order in `IMPLEMENTATION_PLAN.md`**. PR-00..PR-12 remain as the technical backlog, but the first cycle is reorganized to prove value and checks before advanced infrastructure.

## Slice 0 — Baseline and minimal fork

- Pin the upstream SHA, licenses, Rust/Node requirements and `cargo test --workspace`/smokes as local support allows.
- Assisted installation from the repo URL; no third-party setup or script is executed silently.
- Create original fixtures: one dense screen and one journey with 2–3 steps, with known rules and oracles.
- Establish baselines A=plain Craft, B=Craft+UX instructions and C=REA; freeze the rubric and conditions before execution.
- Acceptance: reproducible environment, documented baseline, no claim of improvement.

## Slice 1 — First end-to-end redesign

- `RedesignKind` with CLI/MCP integration in host_agent, without a nested runner.
- Reuse Craft's Study; persist a minimal Task Model and evidence/unknowns.
- Generate a diagnosis and options including `keep_as_is`; build React/TS.
- **Bring forward minimal checks:** build/typecheck, critical-task smoke, evidence references, fail-closed on timeout, Concept/Prototype Ready report.
- Acceptance: the fixture task works and has not regressed; logs contain no secrets; `unknown` does not become `passed`.

## Slice 2 — UX differentiation

- Local Knowledge Packs with rules/contraindications and lean retrieval.
- Track decisions and content quality (labels, errors, states).
- Run the A/B/C benchmark with blind review and pre-registered metrics; account for calls, duration and tokens when available.
- Acceptance: demonstrable gain in at least one relevant dimension with no critical regression; otherwise, simplify to skills/checks.

## Slice 3 — Real corporate DS

- Integrate a fixture React/TS package with tokens and components.
- `required_component`, `allowed_composition`, `extension_requires_approval`.
- Validate imports/props/version, tokens and state semantics.
- Acceptance: build and positive/negative checks with a consistent strict policy.

## Slice 4 — Selective deep investigation

- First measure the gaps in native Study. Only then test REA or Application MCP in isolation.
- Record marginal coverage, false positives, latency, cost and risk.
- Acceptance: integrate only if the incremental benefit is reproducible.

## Slice 5 — Robustness, desktop and release

- Upstream regression, adversarial security, cross-platform installation, manual accessibility and OSS governance.
- Acceptance: reproducible release, agent-first docs, explicit limitations, no promise of automatic production.

## Change rule

No new module enters the critical path without a documented hypothesis, test, budget and fallback. Each slice requires a residual-risk report. There are no invented dates or percentages.

## Mandatory addendum v1.3.0 — entry conditions per slice

**Slice 0:** record `TRUST_BOUNDARIES.md`, `ORACLE_POLICY.md`, license/attribution baseline, upstream SHA, side-effect-free `doctor`, a fixture with independent oracles and a pre-registered benchmark plan.

**Slice 1:** isolated output by default, prohibit patching without approval, oracles whose origin is external to generation, fail-closed on timeout, redaction before persisting, path/origin scope and prohibition of a nested builder.

**Slice 2:** blind evaluation with sampling and thresholds defined before execution; do not rely on LLM-as-judge alone; task metrics and functional results.

**Slice 3:** DS versioned by hash/lockfile; do not redistribute private assets; validate component semantics beyond import/props.

**Slice 4:** REA/MCP opt-in; new evidence, permissions and data policy demonstrated; no automatic mutating call.

**Slice 5:** CI/upstream regression, real smoke per agent/OS, keyboard/focus/zoom/screen reader on a sample, retention and purge tested.

**Blocking rule:** no slice may declare `Ready` if there are critical tests not executed or oracles that are only inferred. Scope and unknowns remain visible.
