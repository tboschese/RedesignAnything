# PR-08 — Native Redesign Checks + Ready Gates

## Tool + kind check

- `redesign_checks_run` — fail-closed native Rust checks written to `redesign/check_results.json`
- `redesign_report_write` — runs the same checks and writes `redesign/report/report.md` and `report.html` for any project: what waits for the person, what is not passing, each scenario with how it was judged, likeness to the original, then every check
- `RedesignReadyCheck` (`redesign-ready`) — registered on `RedesignKind` next to `BuildCheck` (no `WebVisualCheck` / no `similarity`)

## Gates

- **concept_ready** — security ok + preview + build not failed/errored + `identity_essence` not failed (PR-14)
- **prototype_ready** — concept + build pass (`build_scaffold`: a built site in `app/dist`, `build` or `out` that is as new as the sources; the check runs no build, and a scaffold never built or edited since is `not_evaluated`) + design system pass (an advisory warning does not block; a strict one fails) + ≥1 critical oracle passed, each by a journey run in the built prototype + button works + no unmapped critical + no critical a11y fail + `identity_drift` not failed (PR-14)
- **engineering_ready** — prototype + approved backend deps + no secrets

## Guarantees tested

- Error/timeout statuses never equal `pass`
- Zero oracles ⇒ not prototype_ready
- Broken button ⇒ not prototype_ready
- Critic alone never promotes Ready
- No WCAG certification claim from native gate

## Scenarios (PR-17)

`functional_scenarios` judges a scenario by its journey run in the built prototype when there is one for the build on disk (`redesign_journeys_run`), and by a word hint in the sources otherwise. It says which for each scenario. Prototype Ready needs every scenario judged by a journey: when all pass but one rests on a word hint, the row is a warning and the prototype is not ready. See `docs/PR17_JOURNEYS.md`.
