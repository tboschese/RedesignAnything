# Council — consolidated decisions from the v1.2.0 review

**Status:** simulation of professional perspectives; not a meeting with real PhDs and not a consultation of multiple independent models. **Scope:** critical analysis of the specification and architecture; no benchmark was executed. **Decision:** proceed with P0-minimal and measurable gates.

## Simulated members and conflicts of interest

- **Software engineering / architecture:** fork maintenance, interfaces, tests and code quality. Vetoes excessive coupling without evidence.
- **AI engineering:** reasoning quality, baselines, tokens and evaluation. Vetoes undemonstrated "intelligence".
- **Design / HCI:** tasks, accessibility, content and UX metrics. Vetoes purely aesthetic evaluation.
- **Product Management:** target user, differentiation, scope and adoption. Vetoes an inflated P0.
- **Security and privacy (new):** isolation, consent, data and MCP. Vetoes flows that expose credentials or allow unauthorized operations.
- **QA / reliability (new):** oracles, scenarios, non-regression and false positives. Vetoes `passed` when nothing was executed.
- **Developer Experience and OSS (new):** installation from the repository URL, CLI/MCP, documentation, license, contribution and upstream upgrades. Vetoes opaque bootstrap and mandatory installation of the whole stack.
- **Domain / enterprise operations specialist (new):** real rules, roles, frequent tasks, integrations and transactional changes. Vetoes simplifications without proven feasibility.

## Binding decisions

| ID | Decision | Rationale | Acceptance proof |
|---|---|---|---|
| C-01 | P0-minimal as a vertical slice | Avoid subsystems with no validated value | One Study→Build→Check journey executed in the agent |
| C-02 | Pre-registered A/B/C benchmark | Isolate the REA differential | Same fixture/agent/model, blind rubric, report |
| C-03 | Explicit Task Model | UX depends on the work, not the screen | Task Model with unknowns and expected outcome |
| C-04 | Minimal checks brought forward | Detect functional loss early | Build + smoke + traceability from the first slice |
| C-05 | DS in three categories | Avoid an impractical `strict` mode | Tests for official components / composition / extension |
| C-06 | Advisory Critic | An LLM judge is not proof | Its result alone does not promote to Ready |
| C-07 | REA and MCP opt-in | Avoid cost and risk without benefit | Off in P0-minimal; degrade gracefully |
| C-08 | Single agent and effective limits | Avoid expensive loops and double execution | Anti-nesting, timeout and usage-unavailable tests |
| C-09 | Per-project protection | Avoid leaks and dangerous actions | Exfiltration, permission and isolation tests |
| C-10 | Simplification criteria | Avoid sunk cost | ADR documenting keep/remove decisions for subsystems |
| C-11 | Content as part of the redesign | Labels and errors affect usability | Microcopy and terminology review |
| C-12 | Transaction feasibility | UX does not create services that do not exist | UI-only/orchestration/backend-change classification |

## Scope restrictions

P0-minimal does not require REA, the application MCP, canvas, Figma, full Storybook ingestion, a second critic agent, paid inference, multi-tenancy or a new backend. These capabilities remain optional/future. The Craft desktop experience must not regress.

## Register of unproven hypotheses

H1: Knowledge Packs improve quality versus Craft + a UX prompt. H2: a DS adapter produces higher conformance without harming tasks. H3: REA broadens coverage enough to offset its cost. H4: semantic MCP reduces unknowns without introducing incorrect data. H5: the host_agent flow keeps consumption viable. **No hypothesis is considered validated by this documentation.**
