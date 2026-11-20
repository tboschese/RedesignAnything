# Safe Workspace & Patch Policy — v1.3.0

## Default rule
Generate the redesign in a **separate** workspace (`output_mode=isolated`) and preserve the target project's source code, routes, configuration files and secrets. Do not use `git reset --hard`, destructive cleanup, overwriting or auto-merge.

## Apply flow
1. Detect repository state, branch, uncommitted changes and affected files.
2. Generate artifacts in an isolated directory and record hashes of inputs/outputs.
3. Build a readable diff showing changed routes, imports, APIs, data and dependencies.
4. Classify impact: `ui_only`, `frontend_orchestration`, `backend_change_required`.
5. Run known tests, secret scans and a conflict check.
6. Request explicit human approval, by scope and changes, before applying.
7. Apply atomically when possible; on conflict, abort without discarding local changes.
8. Record a documented rollback; never declare a rollback safe without a test.

## Exceptions
`in_place` mode only by explicit opt-in and additional confirmation. `backend_change_required` does not create fictitious contracts or endpoints. `apply_patch` is not enabled in a production environment without specific authorization and risk assessment.

## Blocking tests
SW-01 dirty tree: does not overwrite files.
SW-02 existing route/API changed: diff and approval are mandatory.
SW-03 conflict between generation and application: aborts without loss.
SW-04 symlink outside the root: refuses.
SW-05 patch without approval: refuses.
SW-06 build/test fails: blocks application by default.
SW-07 request for an atomic transaction without a backend: `backend_change_required`.
**Status:** design policy, not an implemented guarantee.
