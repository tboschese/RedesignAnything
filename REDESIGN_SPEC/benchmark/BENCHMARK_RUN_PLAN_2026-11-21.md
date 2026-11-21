# BENCHMARK_RUN_PLAN — 2026-11-21 (draft, not registered)

**Status:** draft. It becomes a pre-registration only when Thiago Boschese registers it (author and
date below), before any confirmatory run. Until then nothing here may gate V1.  
**Extends:** [`BENCHMARK_RUN_PLAN.md`](BENCHMARK_RUN_PLAN.md) (2026-11-20), which stays unchanged.  
**Protocol:** [`../BENCHMARK_PROTOCOL.md`](../BENCHMARK_PROTOCOL.md)

## What changes

Adds the third fixture that `BENCHMARK_RUN_PLAN.md` and `V1_FIRST_RELEASE.md` require: an app with a
real design system.

| ID | Kind | Path | Critical scenarios (min) |
|---|---|---|---|
| `dense-form` | Dense enterprise form | `../fixtures/dense-form/` | 5 |
| `multi-step-journey` | 3-step journey + permission error | `../fixtures/multi-step-journey/` | 4 |
| `ds-claims-journey` | 3-step journey built with `@fixture/acme-ds` + approval limit + viewer role | `../fixtures/ds-claims-journey/` | 6 |

## What does not change

Arms, agent and model controls, repetitions (3 per arm × fixture), raters (≥ 2, blind), primary
metrics, rubric and every numeric threshold are those of `BENCHMARK_RUN_PLAN.md`. A confirmatory run
on three fixtures is therefore 3 fixtures × 3 arms × 3 repetitions = 27 builds.

## Before it can be registered

- The `ds-claims-journey` oracle is `observed_verified` by replay; Thiago confirms it (or not) first
  (`../fixtures/ds-claims-journey/oracle/OBSERVATIONS.md`, `ORACLE_POLICY.md` OP-07).
- An arm's design-system conformance is judged with `redesign_ds_check` against
  `@fixture/acme-ds` 1.2.0 for this fixture only.

## Registration

- Registered by: (pending)
- Registered at (UTC): (pending)
