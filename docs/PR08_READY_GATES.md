# PR-08 — Native Redesign Checks + Ready Gates

## Tool + kind check

- `redesign_checks_run` — fail-closed native Rust checks written to `redesign/check_results.json`
- `RedesignReadyCheck` (`redesign-ready`) — registered on `RedesignKind` next to `BuildCheck` (no `WebVisualCheck` / no `similarity`)

## Gates

- **concept_ready** — security ok + preview + build not failed/errored + `identity_essence` not failed (PR-14)
- **prototype_ready** — concept + build/DS pass + ≥1 critical oracle passed + button works + no unmapped critical + no critical a11y fail + `identity_drift` not failed (PR-14)
- **engineering_ready** — prototype + approved backend deps + no secrets

## Guarantees tested

- Error/timeout statuses never equal `pass`
- Zero oracles ⇒ not prototype_ready
- Broken button ⇒ not prototype_ready
- Critic alone never promotes Ready
- No WCAG certification claim from native gate
