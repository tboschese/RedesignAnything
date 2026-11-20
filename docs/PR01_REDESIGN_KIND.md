# PR-01 — RedesignKind (Slice 1 start)

Implemented on this fork. Not a public V1 release.

## What landed

- `TargetKind` id `redesign` in `crates/ai/src/rebuild/kinds/redesign.rs`
- Registered in `KindRegistry::built_in()` after `game`, before `app`
- Explicit selection only: `craft.json` field `kind`, CLI `--kind redesign`, MCP `kind`, or request markers `kind=redesign` / `mode=redesign`
- Default stack: `React + TypeScript (Vite)`
- Checks: `build` only (no `web_visual` / pixel gate)
- Brief instructs capability preservation, `keep_as_is`, unknowns, no inventing backends
- Headless tools (Clean room allowed): `redesign_project_init`, `redesign_project_show`, `redesign_kinds_list`
- `ExecutionPolicy` + `controller_conflict` when MCP inserts `host_agent` and `rebuild_start` is called without `handoff_to_craft_runner=true`

## How to try

```bash
craft-anything-cli redesign-kinds-list --format json
craft-anything-cli redesign-project-init \
  --request "Redesign the expense form" \
  --target "https://example.test" \
  --projects-dir /tmp/ra-projects \
  --format json
craft-anything-cli redesign-project-show --project /tmp/ra-projects/<folder> --format json
```

## Tests run (this machine)

- `cargo test -p craft-anything-ai --lib redesign` — pass
- `host_agent_blocks_nested_rebuild_without_handoff` — pass
- `rebuild_tools_are_registered_from_the_ai_layer_and_check_their_input` — pass (includes redesign tools + conflict)
- `a_registered_kind_is_chosen_from_its_words_and_brings_its_checks` — pass
- `*_requests_keep_their_project_kind` (8) — pass
- CLI smoke for the three redesign tools — pass
