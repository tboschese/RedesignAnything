# PR-13 — Redesign modes

## What landed

- `RedesignMode` (`refresh` | `redesign` | `reimagine`, default `redesign`) in `crates/ai/src/rebuild/redesign_mode.rs`. `redesign_plan_build` refuses an unknown mode.
- `redesign_modes_list` — read-only description of the three scopes (allowed in Clean room).
- `mode_scope` — a row in `redesign_checks_run`:
  - `refresh` warns when a capability moves to a route it did not have, a mapping needs backend changes, or an identity trait is replaced.
  - `redesign` reports route moves and divergent decisions.
  - A route is compared as written, except the start page: the path of the project's target address is the prototype's `/`, and `%C3%A9` equals `é`. A page other than the start page that the prototype names differently (`/wiki/<name>` as `/article`) still counts as moved.
  - `reimagine` warns when the plan proposes nothing new.
- The builder brief: a redesign project no longer gets the website kind's "Match it exactly" paragraph. `TargetKind::capture_guidance` lets a kind say what to do with the capture; the redesign kind answers by mode (`refresh`: keep the layout; `redesign`: the capture is evidence, not a layout to copy; `reimagine`: propose a new visual design too, keeping or evolving every intentional identity trait).
- `redesign/scaffold_refs.json` records the plan's `mode` and the scaffold's `router`.

## Guarantees tested

- An out-of-scope plan is a warning and never fails the `redesign-ready` check.
- A redesign brief never asks for an exact match, and follows the plan's mode.
- An unknown mode is refused.

## Not in this slice

- Losing the target's identity blocking Ready, measuring how much each page changed, and producing one variant per mode on request.
- `mode_scope` reads the plan only. It does not look at the built app.

User-visible labels are recorded in [feature-names.md](feature-names.md).
