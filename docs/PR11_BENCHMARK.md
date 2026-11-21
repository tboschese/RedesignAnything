# PR-11 — Benchmark harness

## Tool

`redesign_benchmark_dry_run` — arms A/B/C/D with `usage.quality=unavailable` (never coerced to zero).

## Rules

- No invented KPIs
- Arm D cannot claim superiority without measured notes
- See `REDESIGN_SPEC/BENCHMARK_PROTOCOL.md` and `benchmark/BENCHMARK_RUN_PLAN.md`

## Judging a trial: `redesign_benchmark_evaluate` (2026-11-21)

Under the 2026-11-21 plan (`REDESIGN_SPEC/benchmark/BENCHMARK_RUN_PLAN_2026-11-21.md`) the product judges each trial itself; no human rater and no LLM decides a result.

- It serves the arm's built site and runs the evaluating agent's journey for each critical scenario, but only if the journey reaches the scenario's fixed checks in order (`benchmark/expectations/2026-11-21.json`). A missing, reworded or reordered check fails the scenario before anything runs. The controls and fields keep the build's own names, which the record lists for audit.
- It captures the start page and every address a journey opened, at 1440 and 390 px. For `ds-claims-journey` it checks the sources against `@fixture/acme-ds` 1.2.0, strict, with the registry kept in the output folder rather than in the arm's project.
- It computes D1–D5 and their mean as the plan registers them. A measure that no page recorded is listed under `measures_missing`.
- It writes `evaluation.json`, the journeys' pictures and the captures to a new `output_directory`.

Judged as arm D on the unchanged originals, all 15 critical scenarios pass. `multi-step-journey` and `ds-claims-journey` score 5.0 on every dimension. `dense-form` scores a mean of 3.68: it is wider than a 390 px window (D3 = 1) and its keyboard and contrast measure 2.39.
