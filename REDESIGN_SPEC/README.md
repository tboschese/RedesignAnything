# Redesign Anything — Specification package (v1.4.0)

**Status:** package embedded in this fork (`tboschese/RedesignAnything`); PR-01…PR-11 implemented in the engine (see root README and `docs/STATUS.md`); First public release not shipped yet. Base: Craft Anything fork, free/open source, local-first, no paid API required. Initial focus: any web application.

Repository documentation is written in **English**. Chat with maintainers may be in another language; that does not change the canonical docs.

## Start here

1. `V1_FIRST_RELEASE.md`: **binding strategy for the first complete public delivery**.
2. `EXECUTION_V12.md`: sequence of **internal** checkpoints; no partial public release.
3. `COUNCIL_DECISIONS.md`: council decisions, risks, hypotheses, and conditions.
4. `BENCHMARK_PROTOCOL.md`: A/B/C comparison and evaluation criteria.
5. `COUNCIL_ROUND_2.md`: second adversarial review (security, QA, DX/OSS, enterprise domain).
6. `SPEC.md`: product, architecture, requirements, limits, security, outputs, gates, proposed interfaces.
7. `IMPLEMENTATION_PLAN.md`: PR-00 through PR-12, dependencies, tasks, and acceptance criteria.
8. `CONTRACTS.md`: semantic models and integrations; complemented by `schemas/`.
9. `COST_AND_EXECUTION.md`: single-agent control, Lean/Deep profiles, token budget, measurement, enforceable limits, comparative benchmark.
10. `AGENT_FIRST.md`: how to point Cursor/Claude/Codex at the repo and use CLI/MCP without the desktop; some commands there are still **proposed**.
11. `AGENTS_REDESIGN.md`: coding instructions for this fork.
12. `VALIDATION_MATRIX.md`: unit, integration, security, E2E, benchmark, agent-first smoke tests.
13. `SOURCES.md`: consulted repositories and documents, licences, redistribution risks.
14. `CHANGELOG.md`: documentation version changes and compatibility.
15. `examples/` and `schemas/`: reproducible Lean config and nine versioned JSON Schemas (docs v1.2.0, schema envelopes v1.0.0).

## First code delivery

Create the independent fork, keep this package under `REDESIGN_SPEC/`, read upstream `AGENTS.md` / `docs/extending.md` / `docs/pitfalls.md`, and run slices as internal checkpoints. **The First public release requires every capability and gate in `V1_FIRST_RELEASE.md`; no partial delivery may be presented as launch.** `AGENT_FIRST.md` is mandatory from P0.

## Intended use after implementation

> “Use Redesign Anything on GitHub to redesign `https://app.example.com`, using my design system and preserving known flows. Read the repo instructions, diagnose dependencies, and ask permission before installing or accessing private environments.”

Passing the repository URL to an agent is a bootstrap path, **not a guaranteed automatic install**.

## Normative addendum v1.3.0

Also read, **before implementing**:

- `TRUST_BOUNDARIES.md` — real scope of host controls and security.
- `ORACLE_POLICY.md` — independent expected results, gates, inconclusive states.
- `SAFE_WORKSPACE_POLICY.md` — isolated output, diff, and approval for patches.
- `OSS_DX_AND_UPSTREAM.md` — assisted install, private design systems, upstream, accessibility.

The v1.3.0 addendum prevails over older order and policies on conflict. `EXECUTION_V12.md` remains the slice sequence, with additional v1.3.0 conditions. Requirements were not validated against real software at the time of writing the original kit.

## Release strategy v1.4.0

The first public version is the **First public release**, not an MVP. Read `V1_FIRST_RELEASE.md` first. Advanced architecture (REA, Application MCP, corporate DS, journeys) is a V1 **availability** requirement, even though each integration is used selectively per task.
