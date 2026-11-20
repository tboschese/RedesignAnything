# Fixture: multi-step-journey

## Intent
A three-step workspace setup journey: Profile → Workspace → Review. Includes validation errors, a permission-denied branch, and a success confirmation.

## How to serve
```bash
cd REDESIGN_SPEC/fixtures/multi-step-journey/app
python3 -m http.server 8766
# open http://127.0.0.1:8766/
```

## Steps
1. **Profile** — full name and work email required; email must contain `@`.
2. **Workspace** — name required; plan select required; `?denied=1` shows permission error and blocks Continue.
3. **Review** — summarizes choices; Confirm finishes with “Workspace ready.”

## Visible rules (oracle-backed)
1. Cannot advance from Profile without name + valid email.
2. Workspace Continue blocked when `denied=1`.
3. Back navigation preserves previously entered Profile fields.
4. Confirm on Review shows success message “Workspace ready.”

## Intentional unknowns
- Billing webhook behavior after Confirm.
- SSO identity provider mapping.

## Oracle author
Thiago Boschese, 2026-11-20. Type: `human_confirmed` on this synthetic fixture.