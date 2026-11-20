# Redesign Anything — Agent-first, MCP, and CLI

> **Status:** `redesign_*` operations are in this fork’s CLI/MCP (see root README). Binary names are still `craft-anything-cli`. The “CLI — target interface” section below describes a future `redesign-anything-cli` wrapper; use the `redesign-*` commands listed in the README until rebrand.

## Compatibility goal

**Default execution:** `host_agent` + `lean`. The agent that received the link keeps control and calls Redesign CLI/MCP tools. It **does not start** another Craft/Claude/Codex builder without an explicit request/handoff. See `COST_AND_EXECUTION.md` for budget and telemetry mechanisms.

The user should be able to start in Cursor, Claude Code, Codex, or another coding agent by providing **only the Redesign Anything repository URL** and the intent to install/use. The agent reads instructions, checks out, checks prerequisites, asks consent for changes, and uses CLI/MCP without opening the desktop. Do not assume GitHub link = instant install, or that every agent has the same browser/Git/MCP tooling.

## Flow A — User pastes the repo URL

Expected user prompt after the fork is published:

```text
Use this Redesign Anything repository: https://github.com/tboschese/RedesignAnything
Read README.md, AGENTS.md, and REDESIGN_SPEC/AGENT_FIRST.md. Explain prerequisites before
installing anything. Check the repository version and run diagnostics.
Configure CLI/MCP when allowed, without replacing my existing configuration.
Then redesign https://app.example.com in redesign mode, preserving known journeys
and explicitly reporting unverified capabilities.
Do not modify original systems or send private application data to external providers
without my authorization. Show where artifacts and tests landed.
```

## Flow B — Agent is already in the project

```text
In this Redesign Anything checkout, do a functional redesign of the URL I will
provide. Use Study → Build → Check → Ready, prefer the local corporate design system
in strict mode, and do not invent backend contracts.
```

## Flow C — MCP already configured

The agent lists Redesign MCP operations, presents scope and permissions, starts `redesign_project_*`, queries `redesign_study_*`, reads decisions/DS, builds, and runs `redesign_check_*`. Real names are defined in PR-01/PR-03 per contracts in SPEC.md. External remote MCP needs consent and a fixed identity; do not assume `readOnlyHint` guarantees safety.

## CLI — target interface (PROPOSAL ONLY)

```sh
redesign-anything-cli doctor --json
redesign-anything-cli redesign init --url https://app.example.com --mode redesign --config redesign.json
redesign-anything-cli redesign run --project /path/to/project --json
redesign-anything-cli redesign status --project /path/to/project --json
redesign-anything-cli redesign report --project /path/to/project --json
```

**These flags have not been implemented or verified on the binary.** Before documenting them as real, validate against the CLI this fork actually produces, keeping interoperability with upstream commands.

## Mandatory technical requirements

- **Headless parity:** Study, decisions, Build, checks, report, pause/resume available without egui.
- **Single controller:** `host_agent` and `craft_runner` are exclusive modes. Internal `rebuild_start` is not invoked in `host_agent` mode; avoid spending tokens on a second agent doing the same steps.
- **Lean by default:** local Design Intelligence via selective retrieval; extra REA/critic only with evidence or approval that justifies it.
- **Honest telemetry:** do not claim Cursor/Claude/Codex tokens when the host does not expose counters; use product-controlled tool/time limits as fallback.
- **Do not fork the workflow:** reuse Craft `RunSet`, `Workspace`, `ToolSpec`, MCP, approvals, and `TargetKind`/`Check`.
- **Self-diagnosis:** check Rust, CLI/build, optional browser, optional Node/REA/Layout, ports and permissions, listing fix commands — no silent install.
- **Progressive enhancement:** without REA or Layout, base functionality remains; if strict DS was required and is invalid, block qualification.
- **Do not write to the original project:** output under Craft-controlled `app/`, with an explicit diff for edits to an existing repository.
- **Credentials:** only authorized secret store/execution environment; never write tokens into config, logs, prompts, or artifacts.
- **Provenance:** every decision and test cites its source; application MCP data is `declared`, not `observed`, by default.
- **Idempotency:** re-running the same diagnosis does not create new files; restart resumes the versioned project.
- **Compatibility:** Cursor, Claude Code, Codex tested via a capability matrix (shell, Git, MCP, approvals); none is exclusively required.
- **Network:** downloading dependencies, cloning sources, opening the target app, and using remote models must be explicit operations under an egress policy.

## Agent-first P0 acceptance criteria

1. On an agent with authorized shell/Git, repo URL + prompt alone leads to checkout and reading instructions without unnecessary questions beyond essential consents.
2. Diagnosis returns typed JSON without opening a window.
3. Absence of REA, Layout, Storybook, or application MCP appears as unavailable optional capacity, not a core error.
4. A public/local fixture ends with a React/TS build, versioned report, and justified `concept_ready` or `prototype_ready`; output lists checks actually run.
5. A prompt-injection test hidden in the site cannot change instructions, read secrets, or authorize writes.
6. Cursor, Claude Code, and Codex each complete a smoke test at least through Study/Check before compatibility is published.
7. Docs include a minimal example without a paid API, but do not promise a small local model will finish a complex task.
8. A host-agent MCP session does not start `rebuild_start`, a second builder, or a model on its own; transferring control needs explicit confirmation.
9. Without host-agent token metrics, reports show `unavailable`, never zero consumption, and do not claim a hard cap on the subscription plan.
10. REA/MCP/critic limiting instruments and response size are verifiable; budget regression is tested.

## Adoption guide

1. Point the agent at this repository URL and the root README prompt.
2. Confirm Rust and optional Node; consent before installs.
3. Run `craft-anything-cli doctor` and list `redesign-*` commands.
4. Init a `kind=redesign` project; ingest DS when available; Study → Critic → Plan → Scaffold → Checks.
5. Report unknowns, Ready maturity, and artifacts under `redesign/` + `app/`.

See also: `AGENTS_REDESIGN.md`, root `AGENTS.md`, `docs/pitfalls.md`, `V1_FIRST_RELEASE.md`.
