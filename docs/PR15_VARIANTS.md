# PR-15 — Redesign variants on request

A project holds one plan and one app. When the person asks for more than one version, each mode gets its own project, made from the same study.

## What landed

- `redesign_variants_create {project, modes}` — one sibling project per mode (all three when `modes` is empty), in `crates/ai/src/rebuild/redesign_variants.rs`.
  - Carried: `capture/`, `reference/`, and from `redesign/` the evidence, capabilities, oracle, `identity.json` and `design_system.json`. Decisions come back as proposals (choice reset to `keep_as_is`, approval cleared).
  - Not carried: the plan, `scaffold_refs.json`, `check_results.json`, `quality/`, `report/` and `app/`.
  - Refused: a project that is itself a variant, one with no capabilities yet, a mode that already has a variant, a symlinked store entry.
- `redesign_variants_show {project}` — asked on the source or on any variant: each variant's mode, whether it has a plan and an app, and what its last `redesign_checks_run` wrote (Ready gates, `identity_essence`, `identity_drift`, `mode_scope`). It runs no checks.
- A variant records `{source_project, mode}` in `redesign/variant.json`; the source lists them in `redesign/variants.json`. Folder names only. A listed folder counts only when it points back to the source.
- In a variant, `redesign_plan_build` uses the variant's mode and refuses another one. Before a plan exists, the builder brief already follows the variant's mode.
- Engine: `workspace::copy_folder`, the copy `Workspace::duplicate` uses (links skipped, build output left out).

## Why sibling projects

Every reader of a project's plan and app (scaffold, checks, quality runners, identity drift, preview, reports) works on a variant unchanged. Variants inside one project would need a variant parameter in all of them.

## Not in this slice

- Nothing builds the variants: the agent that asked plans, scaffolds and builds each one.
- The default stays one version in mode `redesign`. Variants are made only on request.
- `project_duplicate` still drops a redesign project's kind and store; it is not how variants are made.
