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

## Retrieval

Local tag + text overlap only (no embeddings required). Tools return **3–8** rules with `source_ids`, never the whole pack.

## Critic

`redesign_critic_recommend` always includes a `keep_as_is` option plus contrast alternatives. Heuristics are not WCAG or compliance certificates.
