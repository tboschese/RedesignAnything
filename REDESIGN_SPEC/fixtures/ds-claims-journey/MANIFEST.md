# Fixture: ds-claims-journey

## Intent
A three-step expense claim built with the fixture design system `@fixture/acme-ds`
(`../design-system`): Claim details → Approval → Review. It is the benchmark's third fixture, the
one with a real design system (`BENCHMARK_RUN_PLAN.md`, `V1_FIRST_RELEASE.md`). Every control,
field and colour on the page comes from the package's components and tokens.

## How to serve
```bash
cd REDESIGN_SPEC/fixtures/ds-claims-journey/app
npm install
npm run build      # type-checks, then builds into dist/
npm run preview    # http://127.0.0.1:8767/
```
`node_modules/` and `dist/` are not committed.

## Steps
1. **Claim details** — Description and Amount (EUR) are required; the amount must be greater than 0.
2. **Approval** — a claim over 500 EUR needs an approver code; up to 500 needs none. Back returns to Claim details with the entries kept.
3. **Review** — lists the claim; Submit claim shows "Claim submitted." A viewer (`?role=viewer`) sees "Viewers cannot submit claims." and Submit claim is disabled.

## Visible rules (oracle-backed, see `oracle/`)
1. Cannot continue from Claim details without a description and an amount greater than 0.
2. Over 500 EUR, Approval cannot continue without an approver code.
3. Up to 500 EUR, no approver code is asked for.
4. Back keeps what was entered.
5. Submitting shows "Claim submitted."
6. A viewer cannot submit.

## Intentional unknowns
- What happens to a claim after Submit (approval workflow, email).
- Other currencies; validation of the approver code itself.

## Oracle
`observed_verified` by replay on 2026-11-21 (`oracle/OBSERVATIONS.md`); used without human
confirmation by decision of 2026-11-21.

## Provenance
Original work written for this repository (Thiago Boschese and coding agents, 2026-11-21). No
third-party app, design or wording was copied. Dependencies: React, Vite, TypeScript (MIT), pinned
in `app/package.json`.
