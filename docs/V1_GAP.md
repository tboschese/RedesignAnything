# Gap for the First public release (2026-11-20)

Binding criterion: [`REDESIGN_SPEC/V1_FIRST_RELEASE.md`](../REDESIGN_SPEC/V1_FIRST_RELEASE.md).  
PR-01…11 checkpoints do **not** authorize a public release.

## Summary

| V1 front | Status today | Still needed for publication |
|---|---|---|
| Core Craft / redesign lean | Headless operational (`redesign_*`, kind, store, scaffold, native Ready) | Announced multi-agent smoke; documented continuous regression |
| UX Intelligence | Minimal packs + Critic + `keep_as_is` | **Rich** library (more domains: journey, states, enterprise content); retrieval already exists |
| Multi-screen journeys | Study/plan/traceability; scaffold emits **multiple hash routes + UI states** from plan/capabilities | Full corporate demo; richer page bodies beyond simulated states |
| Corporate DS | Ingest package.json + DTCG + strict check | Storybook/docs path; optional real typecheck; richer “corporate” DS fixture |
| Functional preservation | Fixture oracles + proxy smoke | **Playwright** (or equivalent) wired to oracles; executable positive/negative scenarios |
| Advanced investigation | MCP trust + REA fixture live + **Application MCP fixture** (`redesign_mcp_fixture_smoke`) | Wire `REDESIGN_REA_ADAPTER` to real upstream REA; full MCP Spec bridge when needed |
| Quality Engine | Native Ready, secrets, DS | **axe-core** (or similar); documented manual a11y sample; unified exportable report |
| Agent-first / DX | README + CLI/MCP | Cursor/Claude/Codex matrix executed and published; redesign-aware `doctor` |
| Experience | CLI/report | Before/after preview/comparison in the fixture flow; optional redesign UI |
| OSS / distribution | Inherited licences + `cargo xtask sbom` inventory | Full `cargo about` HTML at release; installers; binary rebrand; fork CI; upgrade/rollback |
| Impact (benchmark) | A/B/C/D dry-run + pre-registered fixtures | **Measured run** + blind human evaluation; gain vs arm B; **do not invent KPIs** |

## Suggested priority (code before release)

1. Scenario/oracle runner + Playwright (fail-closed / `not_evaluated`)
2. axe-core in Ready (no false WCAG certification)
3. Multi-route scaffold from the transformation plan
4. Richer UX packs + “corporate” DS fixture
5. Fixture E2E smoke → `check_results.json`
6. ~~Live REA (opt-in)~~ fixture path landed; optional real REA CLI via `REDESIGN_REA_ADAPTER`
7. Published benchmark + packaging (PR-12)

## Explicitly out of scope right now

- Declaring V1 shipped
- Inventing benchmark results
- Requiring REA/MCP/DS on every task
