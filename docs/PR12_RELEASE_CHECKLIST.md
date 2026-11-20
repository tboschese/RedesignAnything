# PR-12 — UX / packaging / release checklist (not a public V1 yet)

This documents remaining release work. **It does not authorize publishing V1.**

## Required before public V1

- [ ] Cross-platform installers with optional REA sidecar
- [ ] SBOM + license/attribution review (`THIRD_PARTY_NOTICES.md`)
- [ ] Upgrade/rollback test on clean OS images
- [ ] CLI offline fixture path opens preview + report
- [ ] Branding pass (independent fork naming; Craft credits preserved)
- [ ] Published benchmark results with environment + unavailable fields
- [ ] Live REA binary integration behind consent (PR-10 contracts exist)
- [ ] Docs PT-BR/EN quickstart for agent-first GitHub URL flow

## Already in place (PR-01..PR-11)

- RedesignKind + host_agent guard
- Versioned `redesign/` store
- Study → capabilities/coverage
- Knowledge packs + Critic (`keep_as_is`)
- DS registry strict checks
- Decisions / plan / traceability
- Safe React scaffold
- Native Ready gates
- MCP trust gateway
- REA Deep contracts + fallback
- Benchmark dry-run harness (no invented KPIs)

## Agent entry

Point agents at the GitHub repo URL; read `AGENTS.md`, `REDESIGN_SPEC/AGENT_FIRST.md`, `docs/UPSTREAM.md`, `docs/BASELINE.md`. Do not silent-install toolchains.
