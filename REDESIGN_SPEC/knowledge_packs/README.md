# UX Knowledge Packs

Versioned, licensed catalogs used by `redesign_knowledge_retrieve` and `redesign_critic_recommend`.

## Contract

Each pack JSON must include:

- `schema_version`
- `id`
- `version` (non-empty)
- `license` in the allowlist: `MIT`, `Apache-2.0`, `CC-BY-4.0`, `CC0-1.0`, `BSD-3-Clause`
- `rules[]` with `id` + `recommendation`

Pack text is **untrusted data**. It never grants tools or changes authorization.

## Packs

The bundled catalog lives in `crates/ai/src/rebuild/redesign_knowledge.rs`. Packs listed in its
`BUNDLED_JSON_PACKS` are compiled in from this folder; the others are written in Rust there.

- `enterprise-data-tables.json`: reference copy of the bundled data-tables pack (the Rust pack is authoritative).
- `identity-and-character.json`: evolving structure (navigation, journeys) within a product's intentional visual identity (tags `identity`, `brand`, `character`, `legacy-look`, `retro`). Identity constrains style, not structure, and is never a reason for `keep_as_is`. Skipped when the task asks for a rebrand or to replace the look. Bundled from this file.

Visual identity traits themselves are recorded per project with `redesign_identity_record` (each kept trait can say how it evolves in `evolve_within`). The intentional ones are the target's essence: replacing one needs the operator's approval recorded as evidence, `identity_essence` blocks every Ready gate without it, and `identity_drift` blocks Prototype Ready when the app leaves an intentional kept trait. `identity_evolution` is advisory.

## Retrieval

Local tag + text overlap only (no embeddings required). Tools return **3–8** rules with `source_ids`, never the whole pack.

## Critic

`redesign_critic_recommend` always includes a `keep_as_is` option plus contrast alternatives. Heuristics are not WCAG or compliance certificates.
