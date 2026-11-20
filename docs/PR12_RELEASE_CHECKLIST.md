# PR-12 — UX / packaging / release checklist (not a public V1 yet)

This documents remaining release work. **It does not authorize publishing the First public release.**

Binding bar: [`../REDESIGN_SPEC/V1_FIRST_RELEASE.md`](../REDESIGN_SPEC/V1_FIRST_RELEASE.md). Gaps: [`V1_GAP.md`](V1_GAP.md).

## Required before First public release

- [ ] Cross-platform installers with optional REA sidecar
- [x] SBOM inventory + notices scaffold (`cargo xtask sbom` → `docs/sbom/cargo-cyclonedx.json`, `THIRD_PARTY_NOTICES.md`); still run `cargo about` for full HTML licence texts at release
- [ ] Upgrade/rollback test on clean OS images
- [ ] CLI offline fixture path opens preview + report
- [ ] Branding pass (independent fork naming; Craft credits preserved)
- [ ] Published benchmark results with environment + unavailable fields
- [x] Live REA path behind consent (fixture adapter + `redesign_rea_run`; real REA via `REDESIGN_REA_ADAPTER`)
- [x] Docs EN quickstart for agent-first GitHub URL flow (root README + `REDESIGN_SPEC/AGENT_FIRST.md`); optional PT-BR companion can follow

## Already in place (PR-01..PR-11 + quality + REA live)

- RedesignKind + host_agent guard
- Versioned `redesign/` store
- Study → capabilities/coverage
- Knowledge packs + Critic (`keep_as_is`) + journey/empty-state packs
- DS registry strict checks
- Decisions / plan / traceability
- Safe React scaffold with **multi-route** pages from plan/capabilities
- Native Ready gates
- Oracle scenario static eval + Playwright/axe report ingestion (`redesign_quality_run`)
- Fixture smoke (`redesign_fixture_smoke`)
- MCP trust gateway
- REA Deep contracts + fallback + fixture live run
- Benchmark dry-run harness (no invented KPIs)
- English product docs / First public release naming
- Gap map: [`V1_GAP.md`](V1_GAP.md)

## Agent entry

Point agents at the GitHub repo URL; read `AGENTS.md`, `REDESIGN_SPEC/AGENT_FIRST.md`, `docs/UPSTREAM.md`, `docs/BASELINE.md`. Do not silent-install toolchains.

## SBOM

```bash
cargo xtask sbom
# optional full texts (asks to install cargo-about once):
# cargo install cargo-about --locked
# cargo about generate about.hbs -o THIRD-PARTY-LICENSES.html
```
