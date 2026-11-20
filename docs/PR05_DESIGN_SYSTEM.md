# PR-05 — Design System Registry (strict/advisory)

## Tools

- `redesign_ds_ingest` — ingest React/TS `package.json` exports + optional DTCG tokens
- `redesign_ds_show` — read `redesign/design_system.json`
- `redesign_ds_check` — local import/prop/hardcoded-color compliance

## Guarantees tested

- Fixture package ingests without Storybook
- Real `@fixture/acme-ds` import passes; unknown package fails
- Unknown prop fails; hardcoded hex fails in `strict`, warns in `advisory`
- Package version mismatch fails
- Layout CLI degraded never masks a strict failure

## Fixture

`REDESIGN_SPEC/fixtures/design-system/` (`@fixture/acme-ds@1.2.0`).
