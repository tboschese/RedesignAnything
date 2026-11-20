# Oracle Policy — independent expected results v1.3.0

## Problem
An LLM that infers a rule, generates code and writes the test for its own hypothesis does not provide independent evidence of parity.

## Oracle types and precedence
1. `observed_verified`: behavior observed in the original application, with replay and version/role/tenant/flags context.
2. `contract_verified`: API contract, versioned official documentation or MCP rule confirmed by an owner and/or by observation.
3. `human_confirmed`: explicit acceptance by a domain expert, with identity and date.
4. `inferred_unverified`: hypothesis from an agent or from REA; it **must not** produce `passed=true` in a parity test.
5. `unknown`: absence of information; it is **not** equivalent to success.

The test code may be AI-generated, but `expected` and its provenance must come from sources 1–3. When the observed source itself has partial coverage, record the denominator and the unobserved states. Conflicting evidence blocks the gate until it is reconciled or the scope is explicitly reduced.

## Minimum contract
`scenario_id`, `task_id`, `inputs`, `preconditions`, `expected`, `oracle_type`, `evidence_ids`, `environment_fingerprint`, `role`, `tenant`, `feature_flags`, `captured_at`, `oracle_author`, `coverage_scope`, `unknowns`.

## Gate
- `passed`: real execution, verifiable assert, permitted oracle, result and log persisted.
- `failed`: reproduced divergence.
- `inconclusive`: timeout, unavailability, environment divergence or insufficient oracle.
- `not_run`: not executed.
- `inferred_unverified` and `unknown` never count as passed.
- **Concept Ready** accepts identified hypotheses but does not claim parity.
- **Prototype Ready** requires all *known and declared* critical scenarios to pass; unknown scenarios appear explicitly in the report.
- **Engineering Ready** requires adequate integration oracles and human review for critical flows; it does not mean production ready.

## Blocking tests
OP-01 expected produced only by an LLM: `inconclusive`.
OP-02 test fails but the agent claims success: gate fails.
OP-03 timeout/error in the checker: `inconclusive`, not `passed`.
OP-04 evidence from different versions/roles: block or restricted scope.
OP-05 critical scenario omitted from the denominator: invalid report.
OP-06 unobserved stateful simulation treated as a real contract: block.
OP-07 human oracle without traceable confirmation: not promoted.
**Status:** specified, not implemented.
