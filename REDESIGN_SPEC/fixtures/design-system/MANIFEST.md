# Fixture design system (`@fixture/acme-ds`)

Minimal React/TS package used by PR-05 registry ingest and compliance tests.

- Exports: `Button`, `TextField`, `tokens`
- Tokens: DTCG JSON under `tokens/dtcg.json`
- No Storybook on purpose — missing Storybook must not block a valid package import
