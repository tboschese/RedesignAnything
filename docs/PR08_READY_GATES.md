# PR-08 — Native Redesign Checks + Ready Gates

## Tool

`redesign_checks_run` — fail-closed native Rust checks written to `redesign/check_results.json`.

## Gates

- **concept_ready** — security ok + preview + build not failed/errored
- **prototype_ready** — concept + build/DS pass + ≥1 critical oracle passed + button works + no unmapped critical + no critical a11y fail
- **engineering_ready** — prototype + approved backend deps + no secrets

## Guarantees tested

- Error/timeout statuses never equal `pass`
- Zero oracles ⇒ not prototype_ready
- Broken button ⇒ not prototype_ready
- Critic alone never promotes Ready
- No WCAG certification claim from native gate
