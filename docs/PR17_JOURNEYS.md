# PR-17 — Journeys run in the built prototype

## What landed

- `web_site_journey_run {steps, width?, restart?, wait_ms?, output_directory?}` (`crates/engine/src/tools/website_tools/journey.rs`) does a visitor's steps on the open website and says which one failed.
  - Steps: `{"click": name}`, `{"type": text, "into": field}`, `{"press": key}`, `{"open": path}`, `{"expect": text}`.
  - A control is found by its visible or accessible name, a field by its label, placeholder or name. Open shadow roots are searched.
  - A click is a real pointer click at the control's centre: a covered or disabled control fails the step.
  - The page is loaded again before the first step unless `restart` is false. The run stops at the first failed step.
  - It runs the site's code, so it needs approval, like `web_page_states_capture`.
- `redesign_journeys_run {project, journeys?, width?}` (`crates/ai/src/rebuild/redesign_journeys.rs`) serves the built prototype and runs one journey per oracle scenario.
  - A journey names a scenario of `redesign/oracle/scenarios.json` and must end with an `expect`.
  - `journeys` replaces the ones the project keeps in `redesign/quality/journeys.json`; without it the kept ones run again.
  - It writes `redesign/quality/journeys-report.json` (every step, with what happened) and a picture of where each journey ended in `redesign/quality/journeys/<scenario>/end.png`.
- `functional_scenarios` judges a scenario by its journey when the report is about the build on disk, and by a word hint otherwise. Each scenario says which (`method`: `journey_run` or `word_hint`), and the summary counts both.
  - A failed journey fails a scenario whose word is in the sources; a passed journey passes one whose word is not.
  - A report written for an older build is not used, and the summary says to run the journeys again.

## Why journeys belong to the project, not to the oracle

The oracle says what must be true. How a prototype is driven to show it depends on that prototype: two variants of one study name their controls differently. So the steps are kept beside the prototype, and the oracle is not edited.

## Measured on a real site

Three prototypes of the same study, five scenarios each, before (word hints) and after (journeys):

| Prototype | Word hints | Journeys |
| --- | --- | --- |
| Organize screens | 4 pass, 1 fail | 2 pass, 3 fail |
| Redesign journeys | 5 pass | 5 pass |
| New flows | 5 pass | 4 pass by journey, 1 still by word hint |

Organize screens keeps the original's one page, so the two scenarios about an item page were passing on a word in its sources. In New flows one scenario asks for two separate search fields, which that variant replaces with one by an approved decision: no journey can show it, so it has none.

## Retiring a scenario a decision replaces

An approved decision can replace exactly what a scenario asks for (two search fields become one). No honest journey can show that scenario in that variant. `redesign_scenario_retire {project, scenario_id, decision_id, approval_evidence_id, reason}` sets it aside:

- the decision must decide the scenario's capability, change the original and be approved;
- the operator must have approved the retirement itself, recorded as evidence with `source: "user"`;
- it is kept in `redesign/retired_scenarios.json` of this project only, so another variant of the same study still has the scenario;
- it stops counting when the decision is no longer the approved one that replaces it, and `restore: true` brings the scenario back;
- a retired scenario is listed as `retired` with the decision and the reason. It is not counted as passed, failed or total, and a project with no scenario left is still not ready.

This is a stopgap the operator asked for. The aim is prototypes complete enough that few scenarios need it.

## Guarantees tested

- A journey types, clicks, presses Enter and reads what shows; it finds a control inside an open shadow root; a covered control and a text that never shows fail their own step; each run starts from a page loaded again (live browser test).
- An address on another site is refused by `open`.
- A journey must name a scenario of the oracle, one per scenario, and end with an `expect`.
- A report is read only while the build it ran in is on disk.
- A scenario whose journey ran is judged by it in both directions.
- Running journeys is refused until the person approves it.
- A scenario is retired only by the approved decision that replaces it and with the operator's yes; the retirement lapses when the decision changes; a retired scenario is not owed and is not a pass.

## Limits

- The builder writes the steps. A passed journey shows that these steps passed, not that they are the right steps for the scenario; the report keeps them for a person to read.
- A scenario without a journey is still judged by a word hint, and the check says how many were. Since 1.4.9 a prototype is not ready while one is.
- An `expect` looks for its text anywhere on the page. Controls inside frames are not found.
- When several controls share a name, the first one that shows is used, and the step's note says so.
