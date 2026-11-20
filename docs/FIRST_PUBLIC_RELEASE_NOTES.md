# First public release — notes (2026-11-20)

**Product:** Redesign Anything  
**Repo:** https://github.com/tboschese/RedesignAnything  
**Engine:** Craft Anything fork (MIT OR Apache-2.0) — credits in `NOTICE`, `ATTRIBUTION.md`

## What this release is

An **agent-first, local-first capability release** of Redesign Anything on `main`: Study→Plan→Build→Check for `kind=redesign`, design-system registry, multi-route scaffold, quality runners, fixture REA live, Application MCP fixture, SBOM inventory, and exportable offline reports.

## What this release is not

- Not a claim of **measured UX gain vs Craft + UX prompts (arm B)**. Human blind evaluation remains **unavailable** — see `REDESIGN_SPEC/benchmark/published/`.
- Not a claim of production parity, complete hidden-rule extraction, or WCAG certification.
- Not a full binary rebrand: machine names remain `craft-anything*`; window title and docs say **Redesign Anything**.
- Not signed cross-platform installers (packaging scripts exist; signing keys are operator-owned).

## How to use (agent)

```text
Use https://github.com/tboschese/RedesignAnything
Read README.md, AGENTS.md, REDESIGN_SPEC/AGENT_FIRST.md.
No silent installs. Prefer host_agent + lean. Preserve known journeys; report unknowns.
```

Offline fixture smoke + report:

```bash
craft-anything-cli redesign-fixture-smoke --fixture-id dense-form
# opens paths under the smoke project: redesign/report/report.html and app/
```

## Binding docs

- Strategy: `REDESIGN_SPEC/V1_FIRST_RELEASE.md`
- Gaps: `docs/V1_GAP.md`
- Checklist: `docs/PR12_RELEASE_CHECKLIST.md`

## Impact gate

`impact_verdict=unevaluated` until a pre-registered blind human study is completed and recorded. Do not invent KPIs.
