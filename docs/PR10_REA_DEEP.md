# PR-10 — REA Deep adapter (opt-in)

## Tools

- `redesign_rea_discover` — Node/REA probe; missing REA → lean fallback
- `redesign_rea_request` — gated call (consent + deep + budget); Clean Room blocked

## First-batch operations

`analyze_web_bundle`, `trace_javascript_semantics`, `inspect_web_network_capture`

## Guarantees tested

- Missing REA → `fallback`
- Clean Room → blocked
- Consent + budget enforced
- Canary/secret findings refused
