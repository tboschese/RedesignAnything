# PR-10 — REA Deep adapter (opt-in)

## Tools

- `redesign_rea_discover` — probes Node + live adapter (in-repo fixture or `REDESIGN_REA_ADAPTER`); missing → lean fallback
- `redesign_rea_request` — gated call record (consent + deep + budget); Clean Room blocked; does not invoke adapter
- `redesign_rea_run` — gated **live** run against a local input file; writes `redesign/rea/`; not Clean Room allowlisted

## Live adapter

Default entry: `REDESIGN_SPEC/fixtures/rea-live/adapter.mjs` (fixture, not upstream REA).

Override: `REDESIGN_REA_ADAPTER=/path/to/node-entry`.

The fixture reads local files only (`network_egress: false`). It is for integration of the live path; it is **not** a claim of full REA semantics.

## First-batch operations

`analyze_web_bundle`, `trace_javascript_semantics`, `inspect_web_network_capture`

## Guarantees tested

- Missing REA → `fallback`
- Clean Room → blocked
- Consent + budget enforced
- Canary/secret findings refused
- Fixture adapter resolves and executes under consent + deep when Node is present
