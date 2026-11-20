# PR-14 — The target's essence blocks Ready

The intentional identity traits of the original are its essence. No redesign, in any mode, may lose it.

## What landed

- `IdentityTrait.approval_evidence_id`: replacing an intentional trait needs a rationale **and** the operator's approval, an evidence record with `source: "user"`. `redesign_identity_record` refuses it otherwise, and also refuses to stop calling a recorded trait intentional, or to drop one with `replace_all`, without that approval.
- A replacement without approval (an older file, a hand edit) still counts as **kept**: the builder is told to keep it and `identity_drift` checks it.
- `identity_essence` — new row in `redesign_checks_run`. Fails when an intentional trait is marked for replacement without approval, or the profile can't be read. `not_evaluated` when no intentional trait is recorded.
- `identity_drift` — now **fails** when the app sources drift from an intentional kept trait; drift on another kept trait still warns.
- Gates: `identity_essence` failing blocks Concept Ready (and so every gate); `identity_drift` failing blocks Prototype Ready. Both fail the `redesign-ready` check; drift can be refined by the builder.
- `identity_evolution` stays advisory.

## Guarantees tested

- Replacing an intentional trait needs a rationale and an existing `user` evidence record.
- Relabelling or dropping an intentional trait cannot bypass the approval.
- An unapproved replacement counts as kept and fails the essence check.
- Losing the essence blocks every gate; a frozen character does not change them.
- An app that drifts from the essence is a concept but never a ready prototype.

## Limits

- The approval is an audit trail, not proof: the store cannot tell who wrote the `user` record.
- The essence is what the agent classified as intentional. With no intentional trait recorded the check is `not_evaluated`, which does not block.
- `identity_drift` is a static scan of app sources and only checks traits that declare checks.
