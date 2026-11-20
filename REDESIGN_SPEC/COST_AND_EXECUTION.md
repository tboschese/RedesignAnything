# Redesign Anything — Orchestration, context, tokens and execution limits

> **Status:** contract proposed for implementation. The ranges below are **planning hypotheses**, not measurements of Redesign Anything builds. No provider rate, subscription or monetary spend is assumed. Read together with `SPEC.md`, `CONTRACTS.md`, `AGENT_FIRST.md`, `VALIDATION_MATRIX.md` and the real Craft roadmap.

## 1. Principle: one controller per execution

Redesign Anything supports two mutually exclusive modes:

1. **`host_agent` (preferred in agent-first mode):** the user is in Cursor/Claude Code/Codex and provides the repository or MCP tools. **That agent** makes decisions, writes the code and controls execution. Redesign provides Study operations, knowledge retrieval, decisions, build artifacts, checks and reports. **Do not start `rebuild_start`, another Claude/Codex/Cursor or an internal model loop automatically.** The host agent may choose to call the full Craft workflow only after explicit consent, ending/transferring the builder responsibility.
2. **`craft_runner`:** the user starts Build/Run from the Craft desktop or CLI. Craft's `RunSet`/runner owns the builder and its refinements. The intelligence and investigation providers **do not start other agents**; they are tools of the same flow.

**Invariant:** at most one model execution controller for each `run_id`. If the same process receives conflicting requests, refuse with the typed error `controller_conflict`. Resumption reuses the original controller or requires an explicit *handoff*, with identity logging, a checkpoint and consent. A connected MCP does not gain the right to trigger the runner implicitly.

**Model fallback:** prefer an agent authenticated by the user; API or local backends only on explicit selection. Using an open source package does **not** make inference free or compatible with any subscription. Do not copy, share or store the host's credentials.

## 2. Execution profiles

| Profile | Default / trigger | Knowledge consulted | Agents | REA and application MCP |
|---|---|---|---|---|
| **`lean`** | Project default | On-demand retrieval: per-task patterns, few components and summarized evidence | 1 controller, critic in the same agent | REA disabled by default; app MCP only for precise, authorized questions |
| **`deep`** | Explicit selection or approved request in the face of a concrete gap | Incremental investigations, more states/journeys, evidence reconciliation | 1 controller; optional, budgeted second review | REA only with `capture_mode=deep`; app MCP with trust policy and consent |

`deep` does **not** by itself change the `clean_room` inspection mode to `deep`; they are distinct axes. The user separately authorizes investigation depth, use of tools with side effects and data leaving for remote models. Application MCP and the design system may be consulted in `lean` if policy allows and if they add necessary context.

### Triggers for REA (all require authorized Deep)

- A critical capability remains uncertain after normal Study and can be clarified by accessible JavaScript/API flow.
- There is a specific conflict between observed evidence and an MCP claim.
- There is a need for stateful simulation and sufficient captures to identify confirmed effects.

**Do not trigger** by default for a screenshot, a style-only change, reading a guideline, generating a component or a simple visual divergence. Record `probe_reason`, hypotheses, tools called, new facts and marginal cost. An investigation that brings no new evidence does not justify an identical call again.

## 3. Context model (minimal load)

1. **Index, don't inject:** local knowledge packs as a versioned catalog with tags, applicability and contraindications. Retrieve 3–8 relevant rules/candidates first, with *source ids*; expand only after justified need.
2. **On-demand registry:** list components concisely, fetch the contract/props/examples of only the chosen components; never put Storybook or the whole package in the prompt.
3. **Evidence by reference:** keep full captures in the Workspace, transmit only summaries, stable ids and small pages. Pagination and maximum response size are applied before results are returned to the agent.
4. **No monolithic prompts:** `RedesignKind::brief_sections` conveys goals, invariants, evidence index, DS policy, criteria and file paths; detailed decisions are retrieved through tools when needed.
5. **Critic once per important proposal:** additional reviews only on a gate failure, a relevant new decision or authorization within budget. Deterministic checks take precedence over verbal judgment.
6. **Safe cache:** keyed by `target_revision`, `role`, `environment`, `capture_mode`, `design_system_digest`, `knowledge_pack_version`, `tool_version` and `policy_hash`. TTL/expiry for mutable data; do not reuse evidence from one profile/tenant in another. Do not store secrets in the cache key or payload.

## 4. Token and cost accounting

**Distinct** definitions per execution and per step:

- `input_tokens_reported`: total inputs reported by the backend, **including what it classifies as cached when applicable**. Do not add cached tokens to the total again.
- `output_tokens_reported`: total model output.
- `cached_input_tokens_reported`: a subset of the input tokens (or explicitly report the provider's semantics if different).
- `reasoning_tokens_reported`: only when the backend reports it; record separately and document whether it is a subset of output, to avoid double counting.
- `total_tokens_processed_reported`: sum of mutually exclusive categories, documenting the backend's criterion; **it is not a context window nor necessarily the billed volume**.
- `billable_tokens` and `monetary_cost`: only when the provider/host supplies a reliable method and its calculation basis; with no configured/supplied rate → `null`, never an assumed `0`.
- `tool_payload_bytes` / `model_context_bytes`: bytes, **not** "exact tokens"; if estimated with an identified tokenizer, keep `tokens_estimated` separate from `tokens_reported`.
- `accounting_quality`: `provider_reported`, `host_reported`, `estimated`, `unavailable`. Treat partial data per field and preserve the origin and version of the measurement.

In `host_agent` mode, the MCP server may **not receive** metadata of the private messages of Cursor/Claude/Codex. Therefore, **do not claim measurement or enforcement of a total token budget when the host does not expose telemetry**. Offer counters that can actually be controlled: calls to Redesign tools, calls to REA, output volume, elapsed service time, checks and refinements that pass through Redesign. The UI/report shows "host tokens unavailable" and directs the user to check the agent's own consumption.

In `craft_runner` mode, use the runner's `usage.json`/events when available and test cached-token accounting per backend. Report as a partial value if the agent/backend omits data; do not simulate precision.

## 5. Limits: advisory vs enforced

**Required per implementation:**

- `token_soft_limit`: **warning**, not a universal block. It can become a verifiable stop only when the controller reports tokens and accepts a reliable interruption. Otherwise, state `warning_unenforceable`.
- `max_tool_result_chars`: truncation/pagination at the gateway, preserving full evidence and a reference to the details.
- `max_mcp_calls` and `max_rea_invocations`: limits on calls made **through the adapter**, with durable counters and a `budget_exhausted` return before the next execution.
- `max_critic_passes`, `max_refine_rounds`: limited in the flow the product controls; if the host performs refinements outside, the product cannot restrict them.
- `max_run_minutes`: deadline for operations/processes supervised by Redesign; it does not block an external Cursor/Claude session.
- `no_progress_limit`: detect repetition of the same tools/arguments without evidence or useful change and end the *loop* controlled by the product; present the reason and next steps.

When the budget is exhausted: persist checkpoints/evidence, cancel own sidecars, produce `status: budget_exhausted` without converting unexecuted checks into success, and request a decision to raise the limits. **Never automatically repeat an expensive operation** when it failed due to budget or denied access.

## 6. Token ranges for planning benchmarks (not commitments)

| Scope | Accumulated processed tokens — initial hypothesis |
|---|---:|
| One simple screen | 150k–400k |
| Complex corporate screen | 400k–1.2 million |
| Journey of 3–5 screens | 1–3 million |
| Module of 10–20 screens | 3–10 million |
| Large application | Above 10 million; do not estimate a maximum before samples |

These include repeated context, input and output across calls, depending on the model's capabilities. **Do not** use these ranges as a price, consumption included in a plan, a guaranteed minimum or a context capacity. The project must record **number of interactions, mean/p95 per tool type, reported/estimated tokens, cached share, cost when observable, time, success rate and final quality**.

Example of an **illustrative scenario only**, complex screen with 600k processed tokens: Study 150k; UX decision 90k; Build 240k; Check/refinement 120k. The value was not measured in REA or in the user's Craft.

## 7. Proposed engineering defaults, adjustable and not validated

| Parameter | Initial default | Enforcement |
|---|---:|---|
| `execution.controller` | `host_agent` when entering via an agent's MCP; `craft_runner` when starting from the UI/CLI Build | Exclusive per run |
| `execution.profile` | `lean` | Apply toolset and provider gating |
| `execution.allow_nested_builder` | `false` | **Hard block** unless an explicitly approved handoff |
| `token_soft_limit` | 750,000 per corporate-screen execution | Warning when a counter exists; otherwise unavailable |
| `max_mcp_calls` | 30 application bridge calls | Hard at own gateway |
| `max_rea_invocations` | 2 per investigation step | Hard at the adapter |
| `max_critic_passes` | 1 per approved alternative | Hard in controlled steps |
| `max_tool_result_chars` | 12,000 characters per summarized result | Hard before sending to the agent; pagination for detail |
| `max_run_minutes` | 90 minutes of operations under Redesign's control | Hard on own processes, not on the host agent |
| `no_progress_limit` | 2 controlled execution steps without useful evidence/change | Hard where there is sufficient observability |

The values are **starting heuristics** and must not prevent a correct result for lack of testing. Measure in a benchmark and recalibrate by task size; do not address `max_refine_rounds` twice (it already exists in `checks`). If the controller does not support telemetry, the result must indicate partially monitored budget.

## 8. Comparative benchmark and release criteria

For the same fixture, same agent/model/mode, profile, OS, browser, inputs and consents, run:

A. Craft alone as the baseline.
B. Redesign Lean (same agent + local knowledge/DS + native checks).
C. Redesign Deep (only when a justified gap exists: REA and/or application MCP).

Record results **per execution, not just per tool**: evidence quality, precision/recall of known rules, functional scenarios, DS adherence, undue changes, *reported* tokens, cache, duration, tools invoked and budget terminations. Run repetitions to assess variance; do not declare an improvement based on a single execution. Also compare the number of active models/controllers (require one per run).

**Initial acceptance:** Lean completes a fixture with at least one known functional scenario and DS applied; does not use REA without a trigger, does not launch a second agent, does not access an unauthorized external model; reports usage only when measurable. If it produces a useful result with high overhead, **do not** block the project solely on a hypothetical value: record and optimize, defining a numeric threshold after real measurement.

## 9. Limits we cannot promise

- A Cursor/Claude/Codex subscription covers any volume or any use of MCP/CLI.
- Counting the private tokens of an external agent that does not provide telemetry.
- Protecting the host's quota with a hard cap applied by external tools.
- Redesign quality on any small local model.
- The same UX quality, execution time or cost across different projects and agents.
- Zero financial cost when the host or external service has its own billing policy.

**Technical references:** Craft `docs/ROADMAP.md` (R5.3, R5.4), `docs/STATUS.md`, `docs/extending.md`; REA `README.md` and `docs/reference/mcp-contracts.md`; Layout CLI `README.md`. See `SOURCES.md` for URLs and implementation status.

## Addendum v1.3.0 — agent security and validation cost

MCP/CLI gateway limits do not restrict the host's native tools. Record `host_enforcement=unknown|verified` separately from `gateway_enforcement`. Benchmarks must account for the cost of independent verifications and must not reduce critical tests to fit a budget. In case of conflict, stop and report `needs_evidence`, never promote `Ready`. Data policy and consent in `TRUST_BOUNDARIES.md`.

## Alignment v1.4.0 — First public release

The first public delivery is not an MVP. All capabilities and launch criteria are in `V1_FIRST_RELEASE.md`. Internal technical checkpoints do not reduce the release scope; security, cost and oracle policies remain mandatory.
