# PR-06 — Design Decision + Feasibility + Traceability

## Tools

- `redesign_diagnose` — feasibility + critic (always includes `keep_as_is`)
- `redesign_decision_propose` / `_choose` / `_approve`
- `redesign_plan_build` / `_show`
- `redesign_traceability_check`

## Feasibility classes

`ui_only` | `frontend_orchestration` | `backend_change_required` | `not_determined`

Missing atomic backend contracts never invent endpoints.

## Guarantees tested

- Three screens without atomic endpoint → `backend_change_required`
- Known backend → orchestration with explicit risks
- Divergent choice → `needs_approval` until approved
- Critical capability without destination → `blocked`
