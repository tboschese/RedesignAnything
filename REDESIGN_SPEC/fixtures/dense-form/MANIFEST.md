# Fixture: dense-form

## Intent
A single dense “employee expense claim” screen with many fields, client validation, role-gated actions, and empty/error/success states. Used for Slice 0 baseline and later UX/DS checks.

## How to serve
```bash
cd REDESIGN_SPEC/fixtures/dense-form/app
python3 -m http.server 8765
# open http://127.0.0.1:8765/
```

## Visible rules (oracle-backed)
1. Amount must be > 0 and ≤ 10000.
2. Category is required.
3. Submit disabled until Amount, Category, and Date are valid.
4. Approver role sees “Approve” button; Employee does not (`?role=approver` vs default).
5. Empty state copy: “No attachments yet.”

## Intentional unknowns
- Server-side duplicate-claim policy (not implemented).
- Tax calculation rules (label only; no formula).

## Oracle author
Thiago Boschese, 2026-11-20. Type: `human_confirmed` on this synthetic fixture (author also observed running the static app).