# Fixture design system (`@fixture/acme-ds`)

Minimal React/TS package used by PR-05 registry ingest and compliance tests.

- Exports: `Button`, `TextField`, `color` and `space` (the tokens as values), `tokens` (DTCG JSON)
- The components render real, token-styled elements (2026-11-21), so an app built on the package looks like it; before that they returned nothing
- Tokens: DTCG JSON under `tokens/dtcg.json`
- No Storybook on purpose — missing Storybook must not block a valid package import

## Provenance

Original work written for this repository (Thiago Boschese and coding agents), MIT like the package says. No third-party design system was copied.
