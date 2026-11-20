# PR-04 — UX Knowledge Packs + Critic

## Tools

- `redesign_knowledge_retrieve` — local tag+text retrieval (3–8 rules, source ids, no embeddings)
- `redesign_critic_recommend` — ≥3 options including mandatory `keep_as_is`

## Guarantees tested

- Pack without license/version rejected
- Disallowed license rejected
- Retrieval never dumps a full pack
- `keep_as_is` always present
- Wizard/cards/sidebar are not universal invariants
- Prompt-injection text in a pack does not grant tools or override authorization

## Bundled domains

tables, filters, forms, navigation, accessibility, density — see `REDESIGN_SPEC/knowledge_packs/`.
