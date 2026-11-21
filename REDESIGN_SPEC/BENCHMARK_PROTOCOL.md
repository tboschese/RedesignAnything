# Benchmark protocol — Redesign Anything v1.2.0

## Hypothesis

H1: REA delivers a better or more reliable UX redesign than Craft with UX instructions, **using the same agent/model/fixture** and without losing known critical functions.

## Arms

- **A — Plain Craft:** a simple redesign request without specialized packs.
- **B — Craft + UX prompt:** detailed instructions equivalent to the knowledge base, without REA's own tools.
- **C — Redesign Anything Lean:** Task Model, packs and checks, without external REA/MCP.
- **D — REA Deep (optional):** C + REA, only if there are specific gaps.

## Experimental control

Fix the application/commit, fixtures, synthetic credentials, DS when applicable, agent, model, temperature when controllable, call limit, hardware, time and instructions. Run multiple repetitions when possible and record variance. Judges are blind to the origin of each proposal. Never interpret a missing token count as zero.

## Metrics and gates

- **Blocking:** known critical functional outcomes; no secret exported; no failed check marked as passed; no nested builder.
- **Quality:** per-task success, time/errors if measured, content review, appropriate density, DS conformance, automated accessibility + manual accessibility on a sample.
- **Coverage:** discovered/observed/inferred/unknown with defined denominators; falsely inferring a rule is a defect.
- **Cost:** duration, calls, retries, tokens reported/cached/estimated/unavailable.
- **UX:** evaluators judge with a rubric and without knowing which arm produced each version; the Design Critic does not replace evaluation.

## Decision before execution

Record numeric thresholds, number of tasks, sample size, tie-break criteria and budget in a dedicated ADR **before** generating results. Without this, the result is exploratory, not causal validation. If B is equivalent to C, prefer the simpler architecture. If D does not add coverage, leave it out of the default flow.

## Status

**Not executed.** This is a proposed methodology; no score or gain has been measured.

## v1.3.0 addendum — mandatory pre-registration

Before the first run, create a versioned `BENCHMARK_RUN_PLAN` with: the set and number of fixtures; at least one dense task and one journey with states; inclusion criteria; number of repetitions per arm; agent/model configuration; run order and randomization when possible; equal budget; blind rubric; tie-break criteria; primary metrics; minimum effect size and **numeric** functional-regression limits **defined before observing results**. Do not invent thresholds in this documentation.

**Slice 0 pre-registration:** [`benchmark/BENCHMARK_RUN_PLAN.md`](benchmark/BENCHMARK_RUN_PLAN.md) (frozen on 2026-11-20; execution not yet started). Fixtures: [`fixtures/`](fixtures/).

Small samples can guide engineering decisions but do not authorize statistical generalization. Separate exploratory results from confirmatory ones. Record all failures, retries, outliers, versions, prompts, environment changes, unavailable tokens and evaluator conflicts of interest. No LLM may approve its own output alone.

Independent oracles and synthetic data must be frozen before running A/B/C; see `ORACLE_POLICY.md`. Baseline B must receive strong UX instructions, without artificial weakening. If C does not usefully outperform B, prefer the simpler solution.

## Addendum 2026-11-21 — judged by the tool

Thiago Boschese decided (2026-11-21) that the benchmark needs no human raters and no human oracle
confirmation: the product must run it and settle it. The rubric is therefore computed from
mechanical measures in the product's browser (fixed oracle checks reached by journeys, captures,
design-system check), and no LLM judges any result; an agent may only drive a build to the fixed
checks. Arms A and B run in plain Craft as the person's own project, so no instruction to match
the original biases them. Pre-registration: [`benchmark/BENCHMARK_RUN_PLAN_2026-11-21-v2.md`](benchmark/BENCHMARK_RUN_PLAN_2026-11-21-v2.md) (finer measures; it superseded the first 2026-11-21 plan after an exploratory run showed its rubric saturates).
Where this addendum conflicts with "blind rubric" or "human evaluation" above, it takes precedence.

## First public release launch gate

The benchmark is mandatory before the first public release; compare plain Craft, Craft with strong instructions and Redesign Anything on distinct fixtures, including a multi-step journey and a real design system. Pre-register metrics and thresholds. If it does not show a useful gain over the baseline with strong instructions, improve the implementation before launch. See `V1_FIRST_RELEASE.md`.
