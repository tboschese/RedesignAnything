# Redesign Anything

**Redesign existing web applications with evidence, design systems, and journey preservation — agent-first, open source, local-first.**

Independent fork of [Craft Anything](https://github.com/storytold/craft-anything) (MIT OR Apache-2.0). This repository is the **Redesign Anything** product; Craft remains the study/rebuild engine.

| | |
|---|---|
| **Repo** | https://github.com/tboschese/RedesignAnything |
| **License** | MIT OR Apache-2.0 — see [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE), [`NOTICE`](NOTICE), [`ATTRIBUTION.md`](ATTRIBUTION.md) |
| **Upstream pin** | `827be601e781…` — [`docs/UPSTREAM.md`](docs/UPSTREAM.md) |
| **Status** | **First public release (capability)** on `main` — [`docs/FIRST_PUBLIC_RELEASE_NOTES.md`](docs/FIRST_PUBLIC_RELEASE_NOTES.md); impact gate unevaluated — [`docs/STATUS.md`](docs/STATUS.md) |
| **Spec** | [`REDESIGN_SPEC/`](REDESIGN_SPEC/README.md) |

> **Not a visual clone.** Pixel likeness is not a gate. Focus is known capability, justified UX (including `keep_as_is`), real design systems, and fail-closed checks.

---

## What it does

1. **Study** — reuses Craft capture/evidence; maps routes and oracles to capabilities with explicit *unknowns*.
2. **Plan** — local knowledge packs, Design Critic (always with `keep_as_is`), decisions, and a typed transformation plan.
3. **Build** — default React + TypeScript (Vite) stack; scaffold with simulated mocks; no nested Craft builder in `host_agent` mode.
4. **Check** — design-system registry, traceability, native Ready (`concept` / `prototype` / `engineering`); no PluginJudge WASM as a blocking gate.

Default control: **`host_agent` + `lean`**. The Cursor/Claude/Codex agent drives the CLI/MCP; it does not start a second Craft builder without an explicit handoff.

---

## For agents (start here)

Paste into Cursor / Claude Code / Codex:

```text
Use this repository: https://github.com/tboschese/RedesignAnything
Read README.md, AGENTS.md, and REDESIGN_SPEC/AGENT_FIRST.md.
Explain prerequisites before installing anything.
Do not silent-install. Check docs/BASELINE.md and docs/UPSTREAM.md.
Then prepare a kind=redesign project (CLI/MCP), preserve known journeys,
and report what was not verified. Do not modify the original system.
```

Agent checklist:

1. Read `AGENTS.md`, `REDESIGN_SPEC/AGENTS_REDESIGN.md`, `docs/pitfalls.md`
2. Confirm Rust (see Get started) and, on recent macOS, `DEVELOPER_DIR` / `SDKROOT` if the linker fails
3. Ask consent before `rustup`, brew, npm, etc.
4. Use `redesign_*` tools / CLI; do not invent endpoints or claim 100% coverage

---

## Get started

### Prerequisites

- Recent stable Rust (upstream asks for 1.97+; this fork’s baseline used 1.100)
- Git
- Optional: Node (Vite/REA), browser for web Study

**No silent installs.** Ask before installing toolchains.

```bash
git clone https://github.com/tboschese/RedesignAnything.git
cd RedesignAnything
# product remote
git remote -v   # origin → tboschese/RedesignAnything
# optional: track Craft
git remote add upstream https://github.com/storytold/craft-anything.git

cargo build -p craft-anything-cli
./target/debug/craft-anything-cli doctor --format text
./target/debug/craft-anything-cli commands --format text | head
```

On macOS, if linking fails with the Command Line Tools SDK:

```bash
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
export SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
```

### Minimal flow (host agent)

Binaries still use the machine name `craft-anything-cli` (Craft heritage). Redesign commands:

```bash
# create a redesign project without starting a nested builder
craft-anything-cli redesign-project-init \
  --request "Redesign the expense form" \
  --target "https://example.test/expenses" \
  --projects-dir "$HOME/RedesignProjects"

# list kinds / default stack
craft-anything-cli redesign-kinds-list

# ingest the fixture DS + React/TS scaffold
craft-anything-cli redesign-ds-ingest \
  --project /path/to/project \
  --package-dir REDESIGN_SPEC/fixtures/design-system \
  --policy strict

craft-anything-cli redesign-scaffold-app --project /path/to/project

# Study → store, critic, plan, ready
craft-anything-cli redesign-study-map --project /path/to/project \
  --oracle-path REDESIGN_SPEC/fixtures/dense-form/oracle/scenarios.json
craft-anything-cli redesign-knowledge-retrieve --query "dense table with filters" --tags table,filtering
craft-anything-cli redesign-critic-recommend --problem "dense expense form"
craft-anything-cli redesign-checks-run --project /path/to/project
```

Classic Craft rebuild with an explicit kind:

```bash
craft-anything-cli rebuild https://example.test "Redesign checkout" \
  --kind redesign \
  --projects-dir "$HOME/RedesignProjects"
```

MCP: the same `redesign_*` tool set via `craft-anything-cli mcp` (see `AGENTS.md`). In a `host_agent` session, `rebuild_start` without handoff returns `controller_conflict`.

---

## Status

In development; nothing is published yet. The redesign flow (Study → Critic → Plan → Build → Checks → Ready) runs end to end on fixtures and on host-agent test runs; what each area can do is in the table below, and what is still open is in [`docs/V1_GAP.md`](docs/V1_GAP.md) and [`docs/STATUS.md`](docs/STATUS.md).

---

## What already exists in this fork

| Area | State |
|---|---|
| `kind=redesign` + persistence in `craft.json` | Ready |
| Versioned `redesign/` store | Ready |
| Study → capabilities / coverage | Ready |
| Knowledge packs + Critic (`keep_as_is`) | Ready |
| Design System registry (strict/advisory) | Ready |
| Decisions / plan / traceability | Ready |
| React+TS+Vite scaffold with mocks + multi-route pages from plan | Ready |
| Native Ready (`redesign-ready` on the kind) | Ready |
| Application MCP trust gateway | Ready (catalogue/gate + fixture smoke) |
| REA Deep | Contracts + fallback + **fixture live** (`redesign_rea_run`); override with `REDESIGN_REA_ADAPTER` |
| A/B/C/D benchmark | Honest dry-run; **no invented KPIs / no published results** |
| Installers / full rebrand | Open — [`docs/PR12_RELEASE_CHECKLIST.md`](docs/PR12_RELEASE_CHECKLIST.md) |
| SBOM inventory | `cargo xtask sbom` → [`docs/sbom/`](docs/sbom/), [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) |

Per-PR docs: `docs/PR01_*.md` … `docs/PR12_*.md`.

Fixtures: `REDESIGN_SPEC/fixtures/` (dense-form, multi-step-journey, design-system).

---

## Principles

- **Explicit unknowns** — never claim 100% coverage when the denominator is incomplete
- **`keep_as_is` is valid** — change requires justification
- **Do not invent backend** — `backend_change_required` when an atomic contract is missing
- **Honest telemetry** — `unavailable` ≠ zero tokens
- **Clean room** — `redesign_*` tools are allowlisted; Deep/REA need consent
- **Do not write to the origin** — output under `app/` + `redesign/`

---

## Repository layout

```text
apps/                 craft-anything, craft-anything-cli, craft-anything-web
crates/               Craft engine (L0–L6) + AI layer with redesign_* modules
REDESIGN_SPEC/        contracts, schemas, fixtures, First public release plan
docs/                 STATUS, UPSTREAM, BASELINE, PRxx
AGENTS.md             agent rules for this repo
```

---

## Development

```bash
cargo xtask fmt
cargo test -p craft-anything-ai --lib redesign_
cargo clippy -p craft-anything-ai --all-targets -- -D warnings
cargo xtask layers
cargo xtask secrets
```

Branches: `feature/`, `bugfix/`, … + roadmap id when applicable. Do not push to `upstream`.

---

## Upstream Craft

This fork inherits Craft’s engine, CLI, MCP, and UI. Craft/ArtCraft docs and branding still appear in parts of the tree until full rebrand.

- Sync and pin: [`docs/UPSTREAM.md`](docs/UPSTREAM.md)
- Test baseline: [`docs/BASELINE.md`](docs/BASELINE.md)
- Craft upstream: https://github.com/storytold/craft-anything

**Never push to the `upstream` remote.**

---

## License and credits

Code under **MIT OR Apache-2.0**, same as Craft Anything. Preserve notices in `NOTICE` and [`ATTRIBUTION.md`](ATTRIBUTION.md). Do not redistribute private customer design systems or data.

---

## First public release

Capability release notes: [`docs/FIRST_PUBLIC_RELEASE_NOTES.md`](docs/FIRST_PUBLIC_RELEASE_NOTES.md).

Still open after the capability ship: **signed** installers, full `cargo about` HTML texts, blind human impact eval (gain vs arm B), binary rename. See [`docs/V1_GAP.md`](docs/V1_GAP.md) and [`REDESIGN_SPEC/V1_FIRST_RELEASE.md`](REDESIGN_SPEC/V1_FIRST_RELEASE.md). Do not invent KPIs.
