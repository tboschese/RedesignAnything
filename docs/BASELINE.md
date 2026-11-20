# Baseline — Slice 0 / PR-00

Honest record of the first local baseline on this fork. **Do not treat this as a claim that Redesign features work.** Engine code is still Craft Anything at the pinned upstream SHA plus documentation/spec only.

## Identity

| Field | Value |
|---|---|
| Date (UTC) | 2026-11-20 |
| Fork HEAD | `63bf1b4b37c7cdd685e90021b5481384ea7a3e09` |
| Upstream pin | `827be601e781daea91ce2013fa0f64d658abd028` |
| Host | Darwin 27.2.0, macOS 27.2.1, aarch64 (Apple Silicon) |
| rustc / cargo | 1.100.0 (satisfies `rust-version = "1.97"`) |
| SDK used | `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer`; `SDKROOT=…/MacOSX.sdk` (Xcode). Default Command Line Tools SDK (`MacOSX27.2.sdk`) failed to link (`tapi` / unknown `arm64e.x1`). |
| Secrets hook | `git config core.hooksPath .githooks` → `cargo xtask secrets` |

## Commands run

```bash
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
export SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk
cargo xtask secrets          # exit 0 — secrets: ok
cargo xtask layers           # exit 0 — layers: ok
cargo test --workspace       # stopped after first failing package (ai)
cargo test --workspace --no-fail-fast   # completed; 2 failing targets
```

Raw logs retained locally under `/tmp/redesign-baseline/` on the machine that ran them (not committed).

## Results summary (`--no-fail-fast`)

Aggregated from every `test result:` line in the no-fail-fast log:

| Metric | Count |
|---|---|
| Suite results ok | 423 |
| Suite results FAILED | 2 |
| Tests passed (sum of suite lines) | 4986 |
| Tests failed | 2 |
| Tests ignored | 38 |

Cargo final report: **2 targets failed** — `-p craft-anything-ai --lib` and `-p craft-anything-ui-egui --lib`.

### Failures (environment-sensitive; not fixed in Slice 0)

1. **`cli_agents::tests::a_chosen_program_that_moved_with_an_update_is_found_again`** (`craft-anything-ai`)  
   Expected chosen program `"codex"`, got the real ChatGPT.app bundled Codex CLI path:  
   `/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`.  
   Likely interacts with a local ChatGPT/Codex install on this host.

2. **`game_files::tests::a_non_text_folder_is_refused_before_a_look_starts`** (`craft-anything-ui-egui`)  
   Panicked with `Os { code: 92, message: "Illegal byte sequence" }` while refusing a non-text folder. Host locale/filesystem quirk; not investigated further in Slice 0.

## Not run / unavailable

- Full CI matrix (Linux/Windows/FreeBSD) — not executed here.
- Live redesign / Study→Build against fixtures — RedesignKind not implemented yet.
- Token accounting for host agents — `unavailable`.
- `cargo xtask secrets` / `layers` required the Xcode SDK env vars above on this Mac.

## Agent-first note

Point agents at https://github.com/tboschese/RedesignAnything — read `README.md`, `AGENTS.md`, `REDESIGN_SPEC/AGENT_FIRST.md`, `docs/UPSTREAM.md`, this file. Do not install toolchains or dependencies silently; request consent. On macOS 27 with CLT SDK issues, set `DEVELOPER_DIR` / `SDKROOT` to Xcode before building.

## Verdict

Baseline is **reproducible with known failures documented**. Slice 0 does **not** claim a fully green upstream suite on this host. Proceed to Slice 1 / PR-01 only with these limitations visible.
