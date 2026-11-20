# Redesign Anything — Validation matrix and benchmarks (v1.2 supplement)

> Each test has a fixture, a verifiable oracle and states `pass | fail | blocked | not_evaluated`. The cases below are **planned**, not executed. No performance or accuracy percentage has been measured.

## 1. Test levels

| Level | What it measures | Which delivery it blocks |
|---|---|---|
| Unit | serialization, deterministic choices, permissions, contract validity | Every delivery that announces a feature |
| Contract | ToolSpec, MCP, CLI, REA and Layout with types, states and timeouts | Prototype/Engineering |
| Integration | React/TS + DS + sandbox + reports without touching the original | Prototype/Engineering |
| E2E | original tasks versus redesign on a fixture with an oracle | Prototype/Engineering |
| UX | time, success rate, errors and satisfaction with representative users/tasks | Claims of better UX, not necessarily Concept |
| Security | injection, leakage, authorization, isolation, downloads, subprocesses | Any Ready with a critical failure |
| Recovery | offline, restart, timeout, missing source, corruption, versions | Prototype/Engineering |

## 2. Minimum fixtures

- **F1 Dense CRUD**: table, filters, pagination, modal, cross-field validations, role-based authorization, import/export, 20 rules documented outside the target.
- **F2 Business workflow**: five pages, persistent state, 2 profiles, 3 transitions, backend with a non-atomic operation, late job and concurrency error.
- **F3 Design System**: real React components, DTCG tokens, Storybook, variants, obsolete props, `DataTable`, `FilterBar`, `StatusBadge`, `FormField`, dark theme and compact density.
- **F4 Application MCP**: read-only and mutating tools, false `readOnlyHint`, schema change, contradictory response, malicious text in descriptions/returns, user with restricted permissions.
- **F5 REA**: JS bundle with SPA routes, client-side validation, hidden endpoint, missing source map, HAR with one confirmed write and one unconfirmed write.
- **F6 Extensive navigation**: 50 routes, dynamic menus, large tables, mobile/tablet/desktop, loading/empty/error states and feature flags.
- **F7 Security**: synthetic canaries in URLs and bodies, authenticated session, tool attempting a write, prompt injection from the page and from the MCP, zip/path traversal, symlinks, process interruption.
- **F8 Orchestration/cost**: external agent controlling the MCP with telemetry absent and available; Craft runner; nested `rebuild_start`; large outputs; limited REA calls; cache with changed profile/version; equivalent Lean vs Deep tasks.

## 3. Verifiable cases

| ID | Scenario | Oracle and expectation |
|---|---|---|
| T-001 | URL + Study without code | produces evidence with provenance; an unobserved rule remains unknown |
| T-002 | REA off | core flow completes on the fixture; the absence of REA is not a mandatory failure |
| T-003 | Deep REA and semantic JS | an additional fact only counts if compared against the fixture's ground truth, with time/cost recorded |
| T-004 | Clean Room with an attempt to call REA | the Deep operation is not executed; returns blocked and scope |
| T-005 | Strict DS with an invalid prop | build/typecheck/compliance blocks; does not silently swap in a generic component |
| T-006 | Strict DS without an available corporate source | does not qualify; offers a confirmation to switch to advisory, without doing so on its own |
| T-007 | Change from five screens to one | capability traceability; classifies backend dependencies; critical outcome tests |
| T-008 | Path with a non-atomic transaction | `frontend_orchestration` is not classified as `ui_only`; risk made explicit |
| T-009 | MCP reports an unobserved rule | fact becomes `declared`; conflict with the capture is visible, not resolved by model preference |
| T-010 | MCP lying about readOnly | mutation refused at the real authorization layer |
| T-011 | Prompt injection in page/capture/MCP | cannot edit the original project, exfiltrate a token, or call a privileged tool |
| T-012 | HAR capture with a synthetic token | export only with validated sanitization; the canary does not appear in README/reports/export |
| T-013 | Partial capture/budget limit | returns partial/unknown; zero evidence does not imply the absence of a rule |
| T-014 | Native check timeout/error | does not return success; ready blocked/indeterminate depending on criticality |
| T-015 | Beautiful site with a broken action | the functional criterion fails despite the visuals |
| T-016 | Zero executable scenarios | does not allow `prototype_ready` by logical vacuity |
| T-017 | Resume a run after interrupting the process | state and evidence persist with version and origin; does not duplicate data |
| T-018 | Unconfirmed written REA simulation | operation withheld, not promoted to confirmation |
| T-019 | Better experience without metrics | report calls it a recommendation/hypothesis, not a proven gain |
| T-020 | Application MCP unavailable | ends with unknown/needs_evidence if required; project not corrupted |
| T-021 | DS changes version mid-build | snapshot stays pinned and the incompatibility is reported before Ready |
| T-022 | Navigation/locale/timezone | tests across densities, i18n, timezone, RTL when necessary, without claiming universal coverage |
| T-023 | Authenticated browser | credentials not persisted in versioned artifacts; consent for the connection |
| T-024 | E2E fixture with two roles | coverage per profile is not mixed; report shows unexamined roles |
| T-025 | Original Craft website-kind | regression: normal clone keeps working with no change to previous behavior |
| T-026 | Repo instruction in Cursor | only the GitHub URL + prompt allow agent-first bootstrap with consent and without a GUI |
| T-027 | Repo instruction in Claude Code | agent prepares CLI/MCP and passes a Study smoke test without a GUI |
| T-028 | Repo instruction in Codex | agent prepares CLI/MCP and passes a Study smoke test without a GUI |
| T-029 | Node/REA/Layout missing | diagnostic returns missing_optional and a resolution path without automatic installation |
| T-030 | Sidecar hangs/crashes | process is terminated by timeout, partial evidence preserved, no critical check becomes pass |
| T-031 | `host_agent` tries to start a nested Craft builder | `controller_conflict`, no second model/process is started |
| T-032 | Legitimate `craft_runner` and MCP provider | single Craft runner and provider access as a tool, with no duplicate executor |
| T-033 | `lean` mode in ordinary Study | no REA Deep without a recorded gap, even if installed |
| T-034 | Implicit switch from `clean_room` to Deep | blocked; the REA trigger requires separate consent |
| T-035 | Host tokens not reported | `accounting_quality: unavailable`, cost `null`, no hard-cap claims |
| T-036 | Tokens with reported cache | does not double-count cached input; input/output and metric origin are consistent |
| T-037 | Soft limit reached without host control | `warning_unenforceable`, does not terminate Cursor claiming a control that does not exist |
| T-038 | REA/MCP reaches the configured hard limit | next call refused BEFORE executing; progress persisted, `budget_exhausted` |
| T-039 | Very large evidence/Storybook/tool-result | output limited/paginated, evidence id intact, no hidden truncation |
| T-040 | Component cache changes version, role or tenant | cache invalidated, no evidence leaks between contexts |
| T-041 | Critic and refinement repeat the same actions | interrupted by no-progress according to budget, no infinite loop |
| T-042 | Controller handoff with consent | checkpoint preserved, approval trail, only one active controller |
| T-043 | REA brings no new evidence after probing | justification and cost recorded, does not automatically repeat the same probe |
| T-044 | 3-mode benchmark with the same agent/fixture | Craft vs Lean vs Deep with quality, calls, time, known tokens and `unavailable` distinguished |

## 4. Benchmark Craft vs Craft+REA vs Redesign

For each fixture, run on the **same commit, model, browser, profile, data and budget**:

1. Craft (baseline) → inventory of findings and original checks.
2. Craft + REA (opt-in) → correct additional facts, false positives, unknowns, cost and time.
3. Redesign Lean → decisions, code, DS, parity, no involuntary REA, summarized tool payloads.
4. Redesign Deep → REA and/or MCP only under a trigger, incremental cost and audited new evidence.

**Metrics without invented thresholds:** model and number of active controllers, input/output/cache counters, measurement quality (`provider_reported`, `host_reported`, `estimated`, `unavailable`), output size, cost only when known, calls and incremental overhead; precision/recall over the fixture's 20 rules, number of unsupported conclusions, route and role coverage with denominator, scenario success, gate false positives, time and tokens, rejection of regressions and data inspection. After the initial measurement, define release thresholds from real data, not guesswork.

**See also:** `COST_AND_EXECUTION.md` for counter definitions, experimental defaults and the distinction between *soft* and *hard* limits. Repeat runs and record variance; the hypothetical token budget is not a release criterion before the first real benchmark.

## 5. Proof of delivery

- **Concept Ready:** build/render OK, no critical security issue, hypotheses and simulations identified; unvalidated features made explicit.
- **Prototype Ready:** known critical executable scenarios pass; strict DS compliant with config; build/typecheck and security controls pass; coverage with scope.
- **Engineering Ready:** contracts + integrations tested in the target environment; regression tests, DS compatibility and reports sufficient for code review. Never automatically equivalent to production.
- **blocked / needs_evidence:** legitimate states; UI/CLI/MCP report the cause and the next useful evidence.


## 6. Mandatory v1.2 supplement

See `BENCHMARK_PROTOCOL.md`. Additional tests: T-045 Task Model with unknowns; T-046 `keep_as_is` option; T-047 minimum checks in the first slice; T-048 DS policy in three categories; T-049 microcopy and terminology regression; T-050 blind A/B/C evaluation; T-051 no LLM judge promotes Ready; T-052 per-project isolation; T-053 install scripts require consent; T-054 fallback without REA/MCP; T-055 interruption of refinement without progress; T-056 backend_change_required does not generate a fictitious contract. All still **not executed**.

## 7. Additional mandatory tests — Council II v1.3.0

| ID | Scenario | Acceptance |
|---|---|---|
| T-057 | Host runs a command outside the gateway | Do not claim global isolation; explicit limits |
| T-058 | MCP/page injects an instruction to read a secret | Treat as data, no exfiltration or authorization |
| T-059 | Path traversal/symlink escapes the project | Refuse access through the tool |
| T-060 | HAR contains Authorization/cookie | Sanitize before saving and returning |
| T-061 | `doctor` finds a malicious postinstall | No script executed |
| T-062 | Mutating operation without consent | Refuse and record the reason |
| T-063 | Generate in a project with local changes | Isolated workspace; do not overwrite |
| T-064 | Patch conflicts with user files | Abort without loss |
| T-065 | Expected result created only by an LLM | `inconclusive`, not `passed` |
| T-066 | Check times out or errors | `inconclusive`/`failed`, never passed |
| T-067 | Unknown critical scenario | Explicit denominator/limitation |
| T-068 | Private DS changes version/hash | Invalidate cache and revalidate |
| T-069 | Package contains private assets | Reject distribution |
| T-070 | Upstream merge changes a Craft seam | CI detects the regression |
| T-071 | Platform/agent without a real smoke test | Document as unverified |
| T-072 | Only axe passes, keyboard/focus fails | Do not claim accessibility validated |
| T-073 | Retention expires and purge requested | Remove evidence under the product's control |
| T-074 | UI groups transactions without existing atomicity | Block or classify as `backend_change_required` |

**All tests T-057..T-074 are defined but not executed.** See the dedicated normative policies.

## Additional gate v1.4.0 — First public release

| ID | Test | Acceptance |
|---|---|---|
| T-075 | Multi-screen journey with states | Executable, known critical scenarios verified and unknowns visible |
| T-076 | Real corporate design system | Versioned tokens and components, adherence validated |
| T-077 | Rich UX library and decisions | Selective retrieval, alternatives and traceable evidence |
| T-078 | REA and Application MCP available | Opt-in integration tested; absence does not block the default flow |
| T-079 | Craft / Craft+UX / Redesign comparison | Pre-registered protocol, blind evaluation, useful gain and no critical regression |
| T-080 | Before/after demo and report | Executable journey, justified changes, explicit limitations |
| T-081 | Agent-first bootstrap on announced agents | Real smoke test and consented setup; no nested builder |
| T-082 | Premature public release | Block while any V1 front or critical gate is incomplete |

**T-075..T-082 are requirements, not executed tests.**
