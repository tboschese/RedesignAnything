# Fixture: expense-portal

## Intent
A harder benchmark fixture (2026-11-21): three screens (My claims, a claim's page with an actions
menu, a four-step New claim wizard) with deliberate UX weaknesses a redesign can fix, so the
benchmark's finer measures have room to tell arms apart. Static HTML/CSS/JS, no build.

## How to serve
```bash
cd REDESIGN_SPEC/fixtures/expense-portal/app
python3 -m http.server 8768 --bind 127.0.0.1
# open http://127.0.0.1:8768/   (viewer: /?role=viewer)
```

## Visible rules (oracle-backed, see `oracle/`)
1. Step 1 needs "What was it for" and an amount greater than 0.
2. Over 300 EUR, the Approval step needs an approver code; up to 300 none is asked for.
3. Going through every step and confirming submits the claim ("Claim submitted for approval.").
4. Back keeps what was entered.
5. A pending claim can be withdrawn from its page ("More" → "Withdraw claim"); others cannot.
6. A viewer cannot submit a claim.

## Known UX problems (deliberate)
- Next sits below long policy text, out of the first window on a phone (D7).
- Errors are listed at the top, far from their fields, and no field is marked invalid (D8).
- A fourth "Are you sure?" step adds an action to every submission (D6).
- Grey status text and links with too little contrast; `outline: none` hides the focus (D5).
- The claims table is 900 px wide and makes a phone scroll sideways (D3).
- Withdrawing hides in a "More" menu.

## Intentional unknowns
- What happens after a claim is submitted; claims live in the page only.

## Oracle
`observed_verified` by replay on 2026-11-21 (`oracle/OBSERVATIONS.md`).

## Provenance
Original work written for this repository (Thiago Boschese and coding agents, 2026-11-21). No
third-party app, design or wording was copied. No dependencies.
