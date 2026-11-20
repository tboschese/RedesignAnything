# Fixture REA live adapter

Minimal Node adapter used by Redesign Anything to exercise **opt-in live** Deep investigation without vendoring the upstream [REA](https://github.com/morluto/rea) product.

| | |
|---|---|
| Entry | `adapter.mjs` |
| Env override | `REDESIGN_REA_ADAPTER` → path to this script or a real REA CLI wrapper |
| Network | None — local file reads only |
| Operations | `analyze_web_bundle`, `trace_javascript_semantics`, `inspect_web_network_capture` |

Samples: `sample-bundle.js`, `sample-capture.har.json`.

Findings are **observed fixture scans**, not production-validated REA semantics. Lean redesign continues when Node/adapter are missing.
