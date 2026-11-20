# Upstream — Craft Anything

This repository is an **independent fork** of [Craft Anything](https://github.com/storytold/craft-anything).
It is **not** a GitHub network fork. Product work for Redesign Anything lives here; Craft remains the engineering base.

## Pinned baseline

| Field | Value |
|---|---|
| Upstream remote | `https://github.com/storytold/craft-anything.git` |
| Upstream branch at bootstrap | `main` |
| Upstream commit SHA | `827be601e781daea91ce2013fa0f64d658abd028` |
| Upstream tip message | `formats, mcp: add SQLite, GPU texture, and MCP transcript parity tests (#2233)` |
| Bootstrap date (UTC) | 2026-11-20 |
| Downstream remote | `https://github.com/tboschese/RedesignAnything.git` |
| License inherited | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `ATTRIBUTION.md`) |

Re-verify the SHA with `git rev-parse HEAD` immediately after clone if you recreate this tree from upstream alone.

## Remotes (required layout)

```text
upstream  → storytold/craft-anything   (fetch / pull only)
origin    → tboschese/RedesignAnything (push destination)
```

**Never push to `upstream`.** Sync Craft changes with:

```bash
git fetch upstream
git checkout -b sync/upstream-$(date -u +%Y%m%d)
git merge upstream/main   # or rebase; resolve conflicts; run CI / cargo test
```

## Product docs

- Redesign specification kit: [`REDESIGN_SPEC/`](../REDESIGN_SPEC/README.md)
- Slice plan: [`REDESIGN_SPEC/EXECUTION_V12.md`](../REDESIGN_SPEC/EXECUTION_V12.md)
- Binding V1 scope: [`REDESIGN_SPEC/V1_FIRST_RELEASE.md`](../REDESIGN_SPEC/V1_FIRST_RELEASE.md)
- Agent coding contract for the fork: [`REDESIGN_SPEC/AGENTS_REDESIGN.md`](../REDESIGN_SPEC/AGENTS_REDESIGN.md)

## Attribution

Preserve Craft Anything / ArtCraft notices, licenses, and [`ATTRIBUTION.md`](../ATTRIBUTION.md).
Brand assets and the Craft raven icon remain upstream material until an explicit rebrand lands.
Do not claim Craft features as Redesign-only work, and do not claim Redesign capabilities before they are implemented and tested.
