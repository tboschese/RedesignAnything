# PR-02 — Redesign workspace store

## Layout

```text
<project>/redesign/
  store.json
  evidence/<ev_*.json>
  capabilities/<cap_*.json>
```

## Tools

- `redesign_store_show`
- `redesign_evidence_put`
- `redesign_capability_put`

## Guarantees tested

- Atomic writes via `write_atomic`
- Id prefixes `ev_` / `cap_`
- Secret-looking summaries refused
- Symlink / path escape refused
- Unknowns preserved on capabilities
- `redesign/` listed in workspace `OWN_NAMES`

## Not yet

- Full Decision / TransformationPlan / CheckResult files
- Migration from v0 envelopes
- Token budget counters wired into Craft usage.json
