# BENCHMARK_RUN_PLAN — pre-registration (Slice 0)

**Status:** pre-registered, **not executed**.  
**Registered at (UTC):** 2026-11-20  
**Author:** Thiago Boschese  
**Protocol:** [`../BENCHMARK_PROTOCOL.md`](../BENCHMARK_PROTOCOL.md)  
**Upstream pin:** `827be601e781daea91ce2013fa0f64d658abd028` (see `docs/UPSTREAM.md`)

This document freezes inclusion criteria, sample size, repetitions, budgets, rubrics, and **numeric thresholds before any A/B/C run**. Changing a threshold after seeing scores invalidates confirmatory use; create a new dated plan instead.

## Fixtures included

| ID | Kind | Path | Critical scenarios (min) |
|---|---|---|---|
| `dense-form` | Dense enterprise form | `../fixtures/dense-form/` | 5 in `oracle/scenarios.json` |
| `multi-step-journey` | 3-step journey + permission error | `../fixtures/multi-step-journey/` | 4 in `oracle/scenarios.json` |

**Future V1 gate** still requires a third fixture with a real design-system package (`V1_FIRST_RELEASE.md`). That fixture is **out of Slice 0** and must be added in a later dated plan before confirmatory V1 benchmark.

## Arms

| Arm | Definition | Notes |
|---|---|---|
| A | Craft common | Simple redesign prompt; no UX pack |
| B | Craft + strong UX instructions | Detailed UX prompt; no Redesign tools |
| C | Redesign Anything Lean | Task model + packs + checks when implemented; no REA |
| D | Optional REA Deep | Only if C leaves measured gaps |

Slice 0 records the plan only. Arms C/D are **not runnable** until Slice 1+ lands.

## Agent / model controls (to fill at execution time)

Record before the first trial of a confirmatory run:

- Host agent product + version
- Model id + temperature (if controllable) or `unavailable`
- `lean` mode; single controller (`host_agent`); no nested builder
- Equal tool/time budget across arms
- Hardware / OS fingerprint
- Randomization: shuffle arm order per fixture when multiple arms run in one session

## Repetitions and sample

| Parameter | Pre-registered value |
|---|---|
| Fixtures per confirmatory run | 2 (Slice 0 set) |
| Tasks per fixture | All oracle scenarios marked `executable: true` |
| Repetitions per arm × fixture | 3 |
| Blind human raters | ≥ 2 |
| Exploratory mini-runs | Allowed; must be labeled exploratory and not used for V1 gate |

## Primary metrics

1. **Critical functional pass rate** — fraction of critical oracle scenarios with gate `passed` (not `inconclusive` / `not_run`).
2. **False-approval rate** — checks marked passed without allowed oracle → must be 0.
3. **Blind UX rubric mean** — see rubric below (1–5 scales).
4. **Cost** — wall time, tool calls, tokens `reported|estimated|unavailable` (never invent 0).

## Blind UX rubric (1–5)

- Task success clarity
- Error prevention / recovery
- Information density appropriateness
- Consistency of controls/labels
- Accessibility of primary path (keyboard + visible focus at minimum)

Raters must not know arm identity. LLM-as-judge alone is **not** sufficient.

## Numeric thresholds (frozen)

Defined **before** observing Redesign outputs:

| Gate | Threshold |
|---|---|
| Blocking functional regressions vs arm B | Critical pass rate(C) ≥ critical pass rate(B) − **0.00** (no drop allowed on critical scenarios) |
| UX differentiation vs B | Blind rubric mean(C) − mean(B) ≥ **+0.40** on the 1–5 scale, **or** ≥ **2** of 5 rubric dimensions improve by ≥ **+0.50** with none worse by ≥ **0.50** |
| Safety | Secret export incidents = **0**; nested builder starts = **0**; fail-open `passed` on timeout = **0** |
| Cost soft budget (equalized) | Per-arm wall time soft cap **45 min** per fixture repetition; soft call cap **80** tool calls; tokens recorded honestly |
| Decision if C ≉ B | Prefer simpler architecture (document and stop expanding Design Engine) |

V1 public release additionally requires the third DS fixture and the full `V1_FIRST_RELEASE.md` gates; this Slice 0 plan alone does **not** authorize release.

## Tie-breakers

1. Safety violations disqualify an arm regardless of UX score.
2. If UX delta < +0.40 and functional parity holds, prefer fewer moving parts (B over C).
3. Arm D only enters default path if it adds ≥ **+10** percentage points coverage of previously `unknown` critical behaviors without safety regressions.

## Execution log

| Field | Value |
|---|---|
| Confirmatory execution started | _not started_ |
| Results location | _n/a_ |
| Deviations from this plan | _none yet_ |
