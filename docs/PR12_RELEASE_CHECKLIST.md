# PR-12 — UX / packaging / release checklist

Binding bar: [`../REDESIGN_SPEC/V1_FIRST_RELEASE.md`](../REDESIGN_SPEC/V1_FIRST_RELEASE.md).  
Notes: [`FIRST_PUBLIC_RELEASE_NOTES.md`](FIRST_PUBLIC_RELEASE_NOTES.md). Gaps: [`V1_GAP.md`](V1_GAP.md).

## First public release package (capability)

- [x] SBOM inventory + notices (`cargo xtask sbom`)
- [x] Live REA path behind consent (fixture + `redesign_rea_run`)
- [x] Docs EN agent-first quickstart
- [x] CLI offline fixture → exportable report (`redesign/report/report.html`) + Vite preview hint
- [x] Branding pass (display name **Redesign Anything**; Craft credits preserved; machine names still `craft-anything*`)
- [x] Published benchmark record with environment + **unavailable** human scores (`REDESIGN_SPEC/benchmark/published/`)
- [x] Packaging guide for this fork (`packaging/redesign/README.md`)
- [ ] Cross-platform **signed** installers (scripts exist; need operator certs)
- [ ] Upgrade/rollback on clean OS images (needs signed artefacts)

## Impact gate (separate from capability ship)

- [ ] Blind human A/B/C evaluation with pre-registered thresholds
- [ ] Recorded gain vs arm B without inventing KPIs

Until then: `impact_verdict=unevaluated` in published RESULTS.
