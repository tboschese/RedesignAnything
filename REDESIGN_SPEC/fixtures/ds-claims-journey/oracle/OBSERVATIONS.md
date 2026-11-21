# Replay observations — ds-claims-journey (2026-11-21)

Each scenario in `scenarios.json` was replayed with `web_site_journey_run` in Craft Anything's
private browser against the built fixture (`npm run build`, `npm run preview`, port 8767).

| Scenario | Steps | Result |
|---|---|---|
| `scenario_claims_details_gate` | Continue; expect both errors and "Claim details" | passed |
| `scenario_claims_approver_required` | type description and 620; Continue; Continue; expect the approver error | passed |
| `scenario_claims_no_approver_under_limit` | type description and 40; Continue; expect the no-approver note; Continue; expect "Review" | passed |
| `scenario_claims_back_preserves` | type; Continue; Back; expect "Claim details"; Continue; expect "Amount: 40.00 EUR" | passed |
| `scenario_claims_submit_success` | type; Continue; approver "MGR-7"; Continue; expect "MGR-7"; Submit claim; expect "Claim submitted." | passed |
| `scenario_claims_viewer_cannot_submit` | `?role=viewer`; type; Continue; Continue; expect the viewer message; click Submit claim | the message shows; the click is refused: "the button \"Submit claim\" is disabled" |

The expected results are `observed_verified` on this synthetic fixture. They are not
`human_confirmed` until Thiago Boschese confirms them; a confirmatory benchmark must not use them
before that (`ORACLE_POLICY.md`, OP-07).
