# BENCHMARK_RUN_PLAN — 2026-11-21 (judged by the tool)

**Status:** registered, **not executed**.
**Registered at (UTC):** 2026-11-21, before any run of this plan.
**Registered by:** the coding agent (Claude Code) under Thiago Boschese's decision of 2026-11-21:
"não tem que ter humano, a ferramenta tem que conseguir resolver" (no human raters; the tool must
settle it).
**Supersedes for confirmatory use:** [`BENCHMARK_RUN_PLAN.md`](BENCHMARK_RUN_PLAN.md) (2026-11-20),
which stays as written. Changing anything below after a confirmatory trial starts invalidates it;
write a new dated plan instead.
**Protocol:** [`../BENCHMARK_PROTOCOL.md`](../BENCHMARK_PROTOCOL.md)

## Decision this plan records

- **No human raters and no human oracle confirmation.** The product runs every arm and scores it
  with mechanical measures. Oracles are `observed_verified` by replay or `human_confirmed` where a
  fixture already has that.
- **No LLM judges.** An agent may *drive* a build (write the clicks and typing that reach each
  check) but never decides a result: every check is a fixed, mechanical observation in the
  product's browser, and the texts it looks for come from the oracle.
- **Arms A and B run in plain Craft as the person's own project**: the fixture's code is the app,
  edited in place (`rebuild_start` with `existing: edit`), so no "match the original" instruction
  and no likeness score pushes a redesign back to a copy.

## Fixtures

| ID | Kind | Path | Critical scenarios |
|---|---|---|---|
| `dense-form` | Dense enterprise form | `../fixtures/dense-form/` | 5 |
| `multi-step-journey` | 3-step journey + permission error | `../fixtures/multi-step-journey/` | 4 |
| `ds-claims-journey` | 3-step journey built with `@fixture/acme-ds` | `../fixtures/ds-claims-journey/` | 6 |

## Arms

The same request opens every arm; A and C get nothing more, B gets the strong UX instructions
below. All arms: the same builder (Claude Code, model Sonnet 5.5, recorded with its version at
run time), the same machine, one controller (`host_agent`), no nested builder, the same budget.

- **Request (A, B and C):** "Redesign this app's interface so its tasks are easier to do. Keep
  everything it does working, and keep its messages word for word."
- **A — Craft, own project:** the fixture app copied into a new folder, opened with
  `rebuild_start` (`project_folder`, `existing: edit`), with the request.
- **B — Craft, own project + strong UX instructions:** as A, with the request followed by:
  1. Start from the tasks: what each screen is for; put the main action of each step where the eye lands first.
  2. Label every field and button with what it does; keep labels short and consistent across steps.
  3. Show each error next to its field, say how to fix it, and announce it (role="alert").
  4. Prevent errors before they happen: disable or hide actions that can't be taken yet, and say why.
  5. Show where the person is in a multi-step flow and let them go back without losing what they entered.
  6. Group related fields; keep one column on a phone; no sideways scrolling at 390 px.
  7. Every control reachable by keyboard in a sensible order, with a visible focus.
  8. Text contrast at least 4.5:1 (3:1 for large text); never colour alone to carry meaning.
  9. Use the project's design system components and tokens when it has one; no hard-coded colours then.
  10. Keep every behaviour, rule, role and message of the app; change how it looks and flows, not what it does.
- **C — Redesign Anything:** a redesign project of the served fixture (`rebuild_start`, kind
  `redesign`, default mode Redesign journeys), with the fixture's oracle imported and, for
  `ds-claims-journey`, the design system ingested (`redesign_ds_ingest`, strict), with the request.

## Sample

| Parameter | Value |
|---|---|
| Fixtures | 3 |
| Repetitions per arm × fixture | 3 (27 builds) |
| Order | per fixture and repetition, arms in a shuffled order recorded before the builds |
| Exploratory runs | allowed, labelled exploratory, never used for the gate |
| A failed build (no built site after the builder ends) | counts as all checks failed for that trial; it is not re-run |

## Measures (all mechanical, per trial)

For each build, served locally, the product runs:

1. **Critical scenarios** — for each scenario, a journey written by the evaluating agent that must
   contain, in order, the fixed checks in [`expectations/2026-11-21.json`](expectations/2026-11-21.json).
   A scenario passes only if every check passes. Report the control and field names used.
2. **Capture** of every page the journeys visit, at 1440 and 390 px, with `web_site_capture`
   (contrast, keyboard walk, names, alt text, language, landmarks, page width).
3. **Design-system check** (`redesign_ds_check`, strict, `@fixture/acme-ds` 1.2.0) on the build's
   sources, for `ds-claims-journey` only.

## Rubric (1–5 per dimension, computed)

| Dimension | Score |
|---|---|
| D1 Task success | 1 + 4 · (critical scenarios passed ÷ critical scenarios) |
| D2 Error prevention and recovery | 1 + 4 · (passed ÷ total) over the `error_scenarios` of the fixture |
| D3 Fits the screen | 5 if no captured page is wider than its window at 390 px, else 1 + 4 · (pages that fit ÷ pages) |
| D4 Consistency | `ds-claims-journey`: 5 if the DS check passes, 3 if it warns, 1 if it fails. Others: 5 − min(4, controls without a name) |
| D5 Keyboard and contrast | 1 + 4 · r · v · (1 − f): r = reached ÷ controls (1 when the walk hit its press limit), v = 1 − controls without visible focus ÷ reached, f = texts too faint ÷ texts checked, each averaged over the captured pages |

An arm's rubric mean is the mean of D1–D5 over its trials.

## Thresholds (frozen)

| Gate | Threshold |
|---|---|
| Functional | Critical pass rate (D1 basis) of C ≥ that of B (no drop allowed) |
| Gain over B | mean(C) − mean(B) ≥ **+0.40**, **or** at least 2 of D1–D5 improve by ≥ **+0.50** with none worse by ≥ **0.50** |
| Safety | secret exports = 0; nested builder starts = 0; a check that timed out or errored counted as passed = 0 |
| Cost (recorded, soft) | wall time ≤ 45 min per trial; tokens `reported`, `estimated` or `unavailable`, never 0 by default |
| If C does not beat B | prefer the simpler architecture and say so (`BENCHMARK_PROTOCOL.md`) |

## Known limits

- The agent that writes the journeys knows which arm built each app. The checks it must reach are
  fixed and mechanical, so it cannot make a scenario pass, but it could fail to find a working
  path; every journey and the names it used are kept with the results for audit.
- D3–D5 measure the pages the journeys visit, not every state of every page.
- No measure stands for taste or brand fit; the rubric is about tasks, errors, fit, consistency
  and access, as the protocol's rubric was.

## Execution log

| Field | Value |
|---|---|
| Exploratory run | _not started_ |
| Confirmatory run | _not started_ |
| Results | _n/a_ |
| Deviations | _none yet_ |
