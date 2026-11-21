# Exploratory run — dense-form, 2026-11-21

**Exploratory, one repetition. Never used for the V1 gate.** It tested the benchmark process end
to end before the confirmatory run; what it found changed the product and the plan.

## Setup

- Fixture `dense-form`; arms in the shuffled order A, B, C (seed 1795278867).
- Same builder for all arms: Claude Code, model Sonnet 5.5, in an isolated configuration.
- Request, B's instructions and arm definitions: `BENCHMARK_RUN_PLAN_2026-11-21.md`.
- A and B: the fixture app copied and opened as an own project (`existing: edit`). C: a redesign
  project of the served fixture with its oracle imported (`redesign_study_map`).

## What the arms did

| Arm | Wall time | Builder's end state | What changed |
|---|---|---|---|
| A | 45 s | ready (build, own tests) | `styles.css` only: cards, larger fields, Submit and Approve pinned to the bottom |
| B | 90 s | ready (build, own tests) | markup, script and styles: grouped fields, hints, a line saying why Submit is disabled, errors under their field with focus, darker text, focus outlines |
| C | 34 min | needs you (Prototype Ready false) | a React rewrite: one column of cards, a "Still required" line, a typed date box with a calendar button, a sticky action bar it said keeps Submit in view |

## Defects it found (fixed)

1. **The keyboard walk stopped inside a native date field** (its day, month and year take a Tab
   each with the focus on the same input). It gave the original, A and B the same low keyboard
   score; C scored well only because its builder replaced the date field to get past the walk.
   Fixed by #67.
2. **An imported oracle was not kept where the builder's checks read it**, so C's builder had no
   scenarios: no journey ran and Prototype Ready could not pass. Fixed by #68.
3. **A journey could not fill a date field** (typing sends characters a date field does not take),
   so two scenarios failed on the original itself. Fixed by #65.

Also seen, not fixed: C's builder waited about 30 minutes for an approval nobody would give, then
went on with "keep as is" (unattended runs cost C time A and B do not spend).

## Scores (`redesign_benchmark_evaluate`, the same reference journeys for every build)

| Rubric | Original | A | B | C |
|---|---|---|---|---|
| First plan, keyboard walk defective | 3.68 | 4.48 | 4.48 | 5.00 |
| First plan, walk fixed | 4.20 | 5.00 | 5.00 | 5.00 |
| v2 (D6–D8 added) | 3.86 | 4.71 | 4.14 | 4.14 |

With the walk fixed, every arm passes all five critical scenarios and scores 5.0 on D1–D5: the
first rubric cannot tell the arms apart. In v2, D6 (interaction cost) is 3.0 for all (same
structure, same actions), D8 is not measurable on `dense-form` (no announced field error), and D7
(main action in the first view) separates them: A pinned Submit to the bottom (5.0); B and C did not
show it in the first window (1.0). C's builder said its sticky bar keeps Submit in view; a picture
of the first window at 390 px confirms it does not.

## What changed because of it

- #65, #67, #68 (the defects above).
- Thiago Boschese's decision (chat, 2026-11-21): finer measures and harder fixtures before the
  confirmatory run. `BENCHMARK_RUN_PLAN_2026-11-21-v2.md` registers D6–D8; harder fixtures follow.
