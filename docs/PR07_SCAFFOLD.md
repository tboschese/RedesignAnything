# PR-07 — Functional scaffold with safe mocks

## Tool

`redesign_scaffold_app` — writes React+TS (Vite) under `app/` with:

- DS import paths from the registry (no private logo/DS source copy)
- Simulated mock data banner (`SIMULATED_DATA`)
- Capability route/action trace stubs
- `nested_builder_started: false` (host-agent safe)

## Guarantees tested

- Refuses without design system registry
- Refuses non-`redesign` kind
- Scaffolds package/App/mocks without calling `rebuild_start`
