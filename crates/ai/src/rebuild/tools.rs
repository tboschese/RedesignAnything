//! Rebuild control as tools, registered from this layer next to the engine's own
//! ([`craft_anything_engine::Engine::shared`]): start a build (new or a follow-up), see whether
//! it's running, add a note while it works, and stop it. The conversation, questions and result
//! are the project tools' (`project_show`, `project_inbox_answer`, `project_play`…).

use super::*;
use craft_anything_engine::EngineRef;
use craft_anything_engine::questions::{self, AskedQuestion};
use craft_anything_engine::tool::{Ctx, Effects, Requires, ToolOutput, ToolSpec, Toolset, parse_params, schema_of};
use craft_anything_engine::workspace::ProjectUsage;
use schemars::JsonSchema;
use serde::Deserialize;

/// The AI layer's tools, for [`craft_anything_engine::Engine::shared`].
pub fn specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            id: "rebuild_start",
            title: "Start a rebuild",
            summary: "Start building: a new project from a request and a target (a web address, an absolute path, or - for none), or a follow-up on an existing project. A coding agent studies, builds, checks and refines in the background; follow it with rebuild_status and project_show, answer its questions with project_inbox_answer. Up to `build.parallel` projects build at once (2 by default); more are queued and start on their own as others finish. It runs a coding agent that writes and builds code, so it needs consent.",
            beginner: "Starts making something.",
            toolset: Toolset::Projects,
            effects: Effects::LAUNCHES_PROCESS.union(Effects::EXECUTES_FOREIGN_CODE).union(Effects::WRITES_FILESYSTEM).union(Effects::ACCESSES_NETWORK),
            requires: Requires::Nothing,
            input_schema: schema_of::<StartParams>,
            run: rebuild_start,
        },
        ToolSpec {
            id: "rebuild_status",
            title: "Rebuild status",
            summary: "Whether a project's build is running in this session (or queued, and where), its status and note, the latest check, and how many questions are waiting.",
            beginner: "Shows how a build is going.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: rebuild_status,
        },
        ToolSpec {
            id: "project_builder_set",
            title: "Choose what builds a project",
            summary: "Choose the coding agent that builds a project's later changes, by an enabled AI backend's name; with `default`, the app's own choice builds it again. With no name, says which one is chosen and which one builds now (`builds` is null when no AI can build). Takes effect from the next build, also from the CLI and over MCP.",
            beginner: "Chooses which AI makes the changes in this project.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<BuilderParams>,
            run: project_builder_set,
        },
        ToolSpec {
            id: "rebuild_activity_show",
            title: "Show a build's activity",
            summary: "What a project's builds did, line by line, newest last: each stage, every command and file the builder touched and how it ended, what it said, and what programs printed outside the conversation. Secrets are hidden. Works while a build runs and after. `limit` lines (200 by default, at most 2000), `offset` lines back from the newest.",
            beginner: "Shows the details of what a build has been doing.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY.union(Effects::CARRIES_TARGET_CONTENT),
            requires: Requires::Nothing,
            input_schema: schema_of::<ActivityParams>,
            run: rebuild_activity_show,
        },
        ToolSpec {
            id: "rebuild_note",
            title: "Add to a running build",
            summary: "Tell a running build something more; the agent reads it at its next step. With no build running, start a follow-up with rebuild_start instead.",
            beginner: "Adds information while it builds.",
            toolset: Toolset::Projects,
            effects: Effects::CHANGES_UI_STATE.union(Effects::WRITES_FILESYSTEM),
            requires: Requires::Nothing,
            input_schema: schema_of::<NoteParams>,
            run: rebuild_note,
        },
        ToolSpec {
            id: "rebuild_note_withdraw",
            title: "Take back a note",
            summary: "Take back a note given with rebuild_note (by the id it returned) while the agent hasn't read it yet; it returns the note's text. Once the agent has read it, it can't be taken back.",
            beginner: "Takes back something you added before the builder reads it.",
            toolset: Toolset::Projects,
            effects: Effects::CHANGES_UI_STATE,
            requires: Requires::Nothing,
            input_schema: schema_of::<WithdrawParams>,
            run: rebuild_note_withdraw,
        },
        ToolSpec {
            id: "rebuild_stop",
            title: "Stop a rebuild",
            summary: "Pause a project's running build. Its work so far is kept; a follow-up continues it.",
            beginner: "Pauses a build.",
            toolset: Toolset::Projects,
            effects: Effects::CHANGES_UI_STATE,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: rebuild_stop,
        },
        ToolSpec {
            id: QUESTION_TOOL,
            title: "Ask about a build",
            summary: "Ask a question about a project's build (\"where are the latest files?\", \"why this engine?\") without starting a build or a round, and without changing anything. Claude Code answers from a copy of the builder's own session, which isn't kept, reading files only (none in Clean room); an AI with an API key answers with the tools that change nothing. Other coding agents are refused. Safe while a build runs: it doesn't interrupt it. Returns the answer, how it was answered and its cost, and keeps it in the project (`rebuild_questions_list`).",
            beginner: "Asks about what was made, without changing it.",
            toolset: Toolset::Projects,
            effects: Effects::LAUNCHES_PROCESS.union(Effects::ACCESSES_NETWORK),
            requires: Requires::Nothing,
            input_schema: schema_of::<QuestionParams>,
            run: rebuild_question_ask,
        },
        ToolSpec {
            id: "rebuild_questions_list",
            title: "List questions about a build",
            summary: "The questions asked about a project's build and their answers, oldest first (the newest 200).",
            beginner: "Shows what you've asked about this project.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: rebuild_questions_list,
        },
        ToolSpec {
            id: "rebuild_check",
            title: "Check my build",
            summary: "For the builder: test the project's current build against the original now, with the same checks Craft Anything runs after each turn (a website's likeness at each width with the padding, colour, font and layout differences; an app's or game's picture against the original's; whether it builds; the mechanics sheet; credits). The project's running build answers it, so the builder can fix what it lists and check again before replying. Not counted as a round. project defaults to the project this tool server works for.",
            beginner: "Lets the builder test its work against the original while it builds.",
            toolset: Toolset::Core,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<CheckParams>,
            run: rebuild_check,
        },
        ToolSpec {
            id: "redesign_project_init",
            title: "Init a redesign project",
            summary: "Create a Redesign Anything project (kind=redesign) for a URL or -, with React+TypeScript (Vite) by default, without starting the Craft builder. Use this from a host agent (Cursor/Claude/Codex). Does not call rebuild_start.",
            beginner: "Starts a redesign project folder without building yet.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<RedesignInitParams>,
            run: redesign_project_init,
        },
        ToolSpec {
            id: "redesign_project_show",
            title: "Show a redesign project",
            summary: "Show a project's kind, stack, status, note, checks and planned steps for Redesign Anything. Read-only; does not start a builder.",
            beginner: "Shows where a redesign project stands.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_project_show,
        },
        ToolSpec {
            id: "redesign_kinds_list",
            title: "List project kinds",
            summary: "List TargetKind ids this build knows (website, game, redesign, app) and the redesign default stack.",
            beginner: "Lists what kinds of project can be made.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<EmptyParams>,
            run: redesign_kinds_list,
        },
        ToolSpec {
            id: "redesign_store_show",
            title: "Show redesign store",
            summary: "Show the versioned redesign/ store for a project: envelope, evidence records and capabilities. Creates an empty store if missing.",
            beginner: "Shows redesign evidence kept with the project.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_store_show,
        },
        ToolSpec {
            id: "redesign_evidence_put",
            title: "Put redesign evidence",
            summary: "Write one EvidenceRecord into redesign/evidence/. Ids must be ev_…; summaries must not look like secrets.",
            beginner: "Saves a piece of redesign evidence.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<EvidencePutParams>,
            run: redesign_evidence_put,
        },
        ToolSpec {
            id: "redesign_capability_put",
            title: "Put redesign capability",
            summary: "Write one Capability into redesign/capabilities/. Ids must be cap_…; unknowns are preserved.",
            beginner: "Saves a redesign capability.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<CapabilityPutParams>,
            run: redesign_capability_put,
        },
        ToolSpec {
            id: "redesign_capabilities_list",
            title: "List redesign capabilities",
            summary: "List Capability records in redesign/capabilities/ (paginated with limit/offset).",
            beginner: "Lists known redesign capabilities.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ListParams>,
            run: redesign_capabilities_list,
        },
        ToolSpec {
            id: "redesign_coverage_show",
            title: "Show redesign coverage",
            summary: "Show coverage over mapped capabilities. Never claims 100% when unknowns remain.",
            beginner: "Shows what is known vs unknown after Study mapping.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_coverage_show,
        },
        ToolSpec {
            id: "redesign_study_map",
            title: "Map Study into redesign store",
            summary: "Map Craft capture routes into capabilities, and/or import an independent fixture oracle JSON. Does not start a new crawl.",
            beginner: "Turns Study captures or fixture oracles into redesign capabilities.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<StudyMapParams>,
            run: redesign_study_map,
        },
        ToolSpec {
            id: "redesign_identity_record",
            title: "Record visual identity traits",
            summary: "Write look-and-feel traits of the original into redesign/identity.json: category, what was observed, whether it is intentional, and whether the redesign keeps, evolves or replaces it, with optional static checks (fonts, corner radius, shadows, colours). Upserts by trait id unless replace_all. The intentional traits are the target's essence: one left undecided is kept, and replacing one (or no longer calling it intentional) needs a rationale and the operator's approval, recorded with redesign_evidence_put (source \"user\") and passed as approval_evidence_id.",
            beginner: "Saves which parts of the original's look should stay.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<IdentityRecordParams>,
            run: redesign_identity_record,
        },
        ToolSpec {
            id: "redesign_identity_show",
            title: "Show visual identity traits",
            summary: "Show redesign/identity.json with counts by intent and decision and the plain-English constraints given to the builder. With nothing recorded, an empty profile that says so.",
            beginner: "Shows which parts of the original's look the redesign keeps.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_identity_show,
        },
        ToolSpec {
            id: "redesign_modes_list",
            title: "List redesign modes",
            summary: "List the three redesign scopes a plan can use: refresh (organise each screen; same blocks in the same places, same routes, journeys, navigation and look), redesign (reorganise journeys with traceability; the default) and reimagine (new flows, pages and visual design, with every intentional identity trait kept or evolved; backend needs flagged, never invented). mode_scope in redesign_checks_run compares the plan with its mode and warns when the plan goes out of scope.",
            beginner: "Shows how far a redesign can go, from organising the screens to new flows.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<EmptyParams>,
            run: redesign_modes_list,
        },
        ToolSpec {
            id: "redesign_variants_create",
            title: "Make one redesign variant per mode",
            summary: "From a studied redesign project, make one sibling project per mode (all three when modes is empty) so the versions can be compared. A variant carries the capture, evidence, capabilities, identity, design system and oracle, with decisions back as proposals; it does not carry the plan, app, checks or reports. Make variants only when the person asks for more than one version. Refuses a project that is itself a variant, has no capabilities yet, or already has that mode.",
            beginner: "Makes separate versions of a redesign, one for each depth of change, to compare.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<VariantsCreateParams>,
            run: redesign_variants_create,
        },
        ToolSpec {
            id: "redesign_variants_show",
            title: "Show redesign variants",
            summary: "List a project's variants (asked on the source or on any variant) with each one's mode, whether it has a plan and an app, what its last redesign_checks_run said (the Ready gates, identity_essence, identity_drift and mode_scope) and its measured likeness to the original. likeness_order warns when a variant that changes less is less like the original than one that changes more, on the same page. Runs no checks. A listed folder that is not a project pointing back to the source is named under not_found.",
            beginner: "Shows the versions of a redesign side by side.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_variants_show,
        },
        ToolSpec {
            id: "redesign_prototype_capture",
            title: "Capture the prototype and measure it",
            summary: "Serve the project's built prototype (app/dist, build or out), open it in the private browser and save a capture of its start page into redesign/prototype_capture/start/ (and of every original route at `/`), at the widths the original was captured at, then run redesign_likeness_measure. `pages` says which prototype address stands for another original page, e.g. {page: \"details\", path: \"/#/library\"}. Build the prototype first: this tool runs no build. It closes any site this session has open. Runs the prototype's scripts, so it needs your approval.",
            beginner: "Photographs the new design and measures how close it is to the original.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM.union(Effects::EXECUTES_FOREIGN_CODE).union(Effects::ACCESSES_NETWORK).union(Effects::LAUNCHES_PROCESS),
            requires: Requires::Engine("chromium"),
            input_schema: schema_of::<PrototypeCaptureParams>,
            run: redesign_prototype_capture,
        },
        ToolSpec {
            id: "redesign_likeness_measure",
            title: "Measure how alike the prototype is",
            summary: "Compare each original page (the start page and every captured route) with the prototype page captured for it, and save redesign/prototype_capture/likeness.json: page structure, word overlap, and tile layout plus detail of the screenshots, each 0..=100. Capture the prototype first: open each page with web_site_open and save it with web_site_capture into redesign/prototype_capture/start/ or redesign/prototype_capture/<route slug>/ (this tool makes the parent folder and lists the pages it expects). A page that can't be compared is named with its reason, including a screenshot with large blank bands. The scores are reported, not judged against a limit; redesign_variants_show warns when a variant that changes less is less like the original than one that changes more. Opens no browser.",
            beginner: "Measures how close the new design stays to the original, page by page.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_likeness_measure,
        },
        ToolSpec {
            id: "redesign_knowledge_retrieve",
            title: "Retrieve UX knowledge rules",
            summary: "Local tag+text retrieval over licensed UX knowledge packs. Returns 3–8 rules with source ids; never injects a whole pack. No embeddings required.",
            beginner: "Finds a few relevant UX rules for the current task.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<KnowledgeRetrieveParams>,
            run: redesign_knowledge_retrieve,
        },
        ToolSpec {
            id: "redesign_critic_recommend",
            title: "Design Critic recommendations",
            summary: "Generate at least three redesign options for a screen/task, always including keep_as_is, with trade-offs and cited pack rule ids. Heuristics only — not certification.",
            beginner: "Suggests redesign options including leaving the screen as it is.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<CriticParams>,
            run: redesign_critic_recommend,
        },
        ToolSpec {
            id: "redesign_ds_ingest",
            title: "Ingest design system package",
            summary: "Build a Design System Registry from a React/TS package.json + exports and optional DTCG tokens. Missing Storybook does not block. Layout CLI is optional and never overrides local policy.",
            beginner: "Loads a design system package into the redesign project.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<DsIngestParams>,
            run: redesign_ds_ingest,
        },
        ToolSpec {
            id: "redesign_ds_show",
            title: "Show design system registry",
            summary: "Show the saved redesign/design_system.json registry for a project.",
            beginner: "Shows the registered design system.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_ds_show,
        },
        ToolSpec {
            id: "redesign_ds_check",
            title: "Check design system compliance",
            summary: "Check app source files against the registry (imports, props, hardcoded colors). Strict failures are never masked by Layout CLI degradation.",
            beginner: "Checks whether the app uses the design system correctly.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<DsCheckParams>,
            run: redesign_ds_check,
        },
        ToolSpec {
            id: "redesign_diagnose",
            title: "Diagnose redesign feasibility",
            summary: "diagnose → retrieve → alternatives: feasibility class, critic options including keep_as_is, without inventing backend contracts.",
            beginner: "Diagnoses what kind of change a screen needs.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<DiagnoseParams>,
            run: redesign_diagnose,
        },
        ToolSpec {
            id: "redesign_decision_propose",
            title: "Propose a design decision",
            summary: "Persist a proposed DesignDecision (default chosen option keep_as_is) under redesign/decisions/.",
            beginner: "Saves a proposed redesign decision.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<DecisionProposeParams>,
            run: redesign_decision_propose,
        },
        ToolSpec {
            id: "redesign_decision_choose",
            title: "Choose a decision option",
            summary: "Select an option on a proposed decision. Divergent choices (not keep_as_is) require later approval.",
            beginner: "Picks one redesign option.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<DecisionChooseParams>,
            run: redesign_decision_choose,
        },
        ToolSpec {
            id: "redesign_decision_approve",
            title: "Approve a design decision",
            summary: "Mark a decision approved with an evidence id (required for divergent choices). Maps to human confirmation / inbox approval.",
            beginner: "Approves a redesign decision.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<DecisionApproveParams>,
            run: redesign_decision_approve,
        },
        ToolSpec {
            id: "redesign_plan_build",
            title: "Build transformation plan",
            summary: "Write a typed transformation plan with capability mappings. Does not invent routes or endpoints.",
            beginner: "Builds the redesign transformation plan.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<PlanBuildParams>,
            run: redesign_plan_build,
        },
        ToolSpec {
            id: "redesign_plan_show",
            title: "Show transformation plan",
            summary: "Show redesign/transformation_plan.json.",
            beginner: "Shows the current transformation plan.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_plan_show,
        },
        ToolSpec {
            id: "redesign_traceability_check",
            title: "Check plan traceability",
            summary: "Block critical capabilities without destination/exception; require approval for divergent decisions and backend_change_required.",
            beginner: "Checks that every important capability has a destination.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_traceability_check,
        },
        ToolSpec {
            id: "redesign_scaffold_app",
            title: "Scaffold redesign React app",
            summary: "Write a React+TypeScript (Vite) app under app/ with simulated mocks and DS import paths. Does not start Craft's nested builder. Requires redesign/design_system.json.",
            beginner: "Creates the redesign app folder with mock data.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_scaffold_app,
        },
        ToolSpec {
            id: "redesign_checks_run",
            title: "Run native redesign Ready checks",
            summary: "Fail-closed native Rust checks and concept/prototype/engineering Ready gates. Errors/timeouts never pass; Critic alone never promotes Ready; no WCAG certification claim.",
            beginner: "Runs redesign ready checks.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<ProjectParams>,
            run: redesign_checks_run,
        },
        ToolSpec {
            id: "redesign_quality_run",
            title: "Run redesign quality runners",
            summary: "Evaluate oracle scenarios against app sources; ingest optional Playwright/axe JSON under redesign/quality/. Missing runners stay not_evaluated; errors fail closed. Never claims WCAG certification.",
            beginner: "Runs functional/accessibility quality checks.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<QualityRunParams>,
            run: redesign_quality_run,
        },
        ToolSpec {
            id: "redesign_fixture_smoke",
            title: "Smoke a Redesign fixture end-to-end",
            summary: "Create a temp redesign project from a fixture (DS + oracle), scaffold, map study, run Ready checks. Does not start a nested builder. Reports honest readiness.",
            beginner: "Runs a full fixture smoke without nested AI builder.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<FixtureSmokeParams>,
            run: redesign_fixture_smoke,
        },
        ToolSpec {
            id: "redesign_mcp_catalogue",
            title: "Catalogue Application MCP tools",
            summary: "Hash tool schemas, classify effects, apply allowlist, and block hostile descriptions or fake readOnly. Redesign works with an empty catalogue.",
            beginner: "Reviews tools from an application MCP.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<McpCatalogueParams>,
            run: redesign_mcp_catalogue,
        },
        ToolSpec {
            id: "redesign_mcp_gate",
            title: "Gate an Application MCP call",
            summary: "Allow or block a tool call given a catalogue profile; refuses secret-looking bodies and schema changes without reapproval.",
            beginner: "Checks whether an MCP tool call is allowed.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<McpGateParams>,
            run: redesign_mcp_gate,
        },
        ToolSpec {
            id: "redesign_mcp_fixture_smoke",
            title: "Smoke the Application MCP fixture server",
            summary: "Spawn the in-repo fixture stdio server, catalogue tools, gate hostile/mutating tools, call app_context_get. Missing Node → not_evaluated. Lean path works without it.",
            beginner: "Checks the Application MCP fixture against the trust gateway.",
            toolset: Toolset::Projects,
            effects: Effects::LAUNCHES_PROCESS,
            requires: Requires::Nothing,
            input_schema: schema_of::<EmptyParams>,
            run: redesign_mcp_fixture_smoke,
        },
        ToolSpec {
            id: "redesign_rea_discover",
            title: "Discover REA Deep availability",
            summary: "Probe Node + live adapter (fixture or REDESIGN_REA_ADAPTER). Missing adapter falls back to lean redesign. Does not start analysis.",
            beginner: "Checks whether REA Deep can be used.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ReaDiscoverParams>,
            run: redesign_rea_discover,
        },
        ToolSpec {
            id: "redesign_rea_request",
            title: "Request a gated REA Deep call",
            summary: "Consent + deep profile + budget required; Clean Room always blocks. Records reason and evidence-gain intent for first-batch ops only. Does not invoke the adapter — use redesign_rea_run.",
            beginner: "Asks to run one REA Deep operation under budget.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<ReaRequestParams>,
            run: redesign_rea_request,
        },
        ToolSpec {
            id: "redesign_rea_run",
            title: "Run gated REA Deep live analysis",
            summary: "After consent + deep + budget, invoke the Node live adapter on a local input file. Writes redesign/rea evidence. Clean Room blocked. No network from the fixture adapter.",
            beginner: "Runs one opt-in REA Deep analysis on a local file.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM.union(Effects::LAUNCHES_PROCESS),
            requires: Requires::Nothing,
            input_schema: schema_of::<ReaRunParams>,
            run: redesign_rea_run,
        },
        ToolSpec {
            id: "redesign_benchmark_dry_run",
            title: "Benchmark dry-run manifest",
            summary: "Emit arms A/B/C/D with unavailable usage (not zero). Does not invent KPIs; use after controlled runs to fill measured fields only.",
            beginner: "Creates an empty honest benchmark template.",
            toolset: Toolset::Projects,
            effects: Effects::READ_ONLY,
            requires: Requires::Nothing,
            input_schema: schema_of::<BenchmarkDryRunParams>,
            run: redesign_benchmark_dry_run,
        },
        ToolSpec {
            id: "redesign_benchmark_publish",
            title: "Publish benchmark record (honest unavailable fields)",
            summary: "Write a publishable A/B/C/D record under REDESIGN_SPEC/benchmark/published/. Human blind scores stay unavailable; never invents gain vs B.",
            beginner: "Publishes an honest benchmark record for the First public release package.",
            toolset: Toolset::Projects,
            effects: Effects::WRITES_FILESYSTEM,
            requires: Requires::Nothing,
            input_schema: schema_of::<BenchmarkPublishParams>,
            run: redesign_benchmark_publish,
        },
    ]
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct EmptyParams {}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RedesignInitParams {
    /// What to redesign, in your words.
    pub request: String,
    /// A web address, or `-` for none.
    #[serde(default)]
    pub target: Option<String>,
    /// Where the new project's folder goes.
    #[serde(default)]
    pub projects_dir: Option<PathBuf>,
    /// Create the project in this folder itself.
    #[serde(default)]
    pub project_folder: Option<PathBuf>,
    /// Optional stack override; default is React + TypeScript (Vite).
    #[serde(default)]
    pub build_with: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct EvidencePutParams {
    pub project: PathBuf,
    pub record: super::redesign_store::EvidenceRecord,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CapabilityPutParams {
    pub project: PathBuf,
    pub capability: super::redesign_store::Capability,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrototypePage {
    /// A page of the original: `start` or a route's slug.
    pub page: String,
    /// The prototype address that stands for it, e.g. `/` or `/#/library`.
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PrototypeCaptureParams {
    pub project: PathBuf,
    /// Prototype addresses for original pages other than the start page.
    #[serde(default)]
    pub pages: Vec<PrototypePage>,
    /// Window widths; the original's when omitted.
    #[serde(default)]
    pub widths: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct VariantsCreateParams {
    /// The studied redesign project to make variants from.
    pub project: PathBuf,
    /// The modes to make a variant for; all three when empty. See redesign_modes_list.
    #[serde(default)]
    pub modes: Vec<super::redesign_mode::RedesignMode>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct IdentityRecordParams {
    pub project: PathBuf,
    pub traits: Vec<super::redesign_identity::IdentityTrait>,
    /// Replace every recorded trait instead of upserting by id.
    #[serde(default)]
    pub replace_all: bool,
    /// Replace the profile's notes.
    #[serde(default)]
    pub notes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ListParams {
    pub project: PathBuf,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct QualityRunParams {
    pub project: PathBuf,
    #[serde(default)]
    pub oracle_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct FixtureSmokeParams {
    /// Fixture id under REDESIGN_SPEC/fixtures/ (default dense-form).
    #[serde(default)]
    pub fixture_id: Option<String>,
    /// Where to create the smoke project (default: system temp).
    #[serde(default)]
    pub projects_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BenchmarkDryRunParams {
    #[serde(default)]
    pub fixture_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct BenchmarkPublishParams {
    #[serde(default)]
    pub fixture_id: Option<String>,
    /// Optional measured wall time for arm C only (milliseconds).
    #[serde(default)]
    pub arm_c_wall_ms: Option<u64>,
    #[serde(default)]
    pub arm_c_notes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReaDiscoverParams {
    /// When set, skip host probe and use this Node flag (tests).
    #[serde(default)]
    pub node_ok: Option<bool>,
    /// When set with node_ok, skip host probe (tests).
    #[serde(default)]
    pub rea_present: Option<bool>,
    #[serde(default)]
    pub rea_version: Option<String>,
    /// Force host probe even if overrides are present.
    #[serde(default)]
    pub probe_host: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReaRequestParams {
    pub operation: String,
    pub reason: String,
    #[serde(default)]
    pub consent_granted: Option<bool>,
    #[serde(default)]
    pub clean_room: Option<bool>,
    #[serde(default)]
    pub deep_profile: Option<bool>,
    #[serde(default)]
    pub budget_calls: Option<u32>,
    #[serde(default)]
    pub node_ok: Option<bool>,
    #[serde(default)]
    pub rea_present: Option<bool>,
    #[serde(default)]
    pub rea_version: Option<String>,
    #[serde(default)]
    pub calls_already_used: Option<u32>,
    #[serde(default)]
    pub probe_host: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReaRunParams {
    pub project: PathBuf,
    pub operation: String,
    pub reason: String,
    /// Local file to analyse (JS bundle, HAR, etc.). Never a remote URL fetch.
    pub input: PathBuf,
    #[serde(default)]
    pub consent_granted: Option<bool>,
    #[serde(default)]
    pub clean_room: Option<bool>,
    #[serde(default)]
    pub deep_profile: Option<bool>,
    #[serde(default)]
    pub budget_calls: Option<u32>,
    #[serde(default)]
    pub calls_already_used: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct McpToolIn {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub declared_read_only: Option<bool>,
    #[serde(default)]
    pub schema: Option<Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct McpCatalogueParams {
    #[serde(default)]
    pub tools: Vec<McpToolIn>,
    #[serde(default)]
    pub allowlist: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct McpGateParams {
    pub profile: super::redesign_mcp_trust::ApplicationMcpProfile,
    pub tool: String,
    #[serde(default)]
    pub body: Option<Value>,
    #[serde(default)]
    pub previous_schema_hash: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DiagnoseParams {
    pub problem: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub capability_ids: Vec<String>,
    #[serde(default)]
    pub needs_atomic_backend: Option<bool>,
    #[serde(default)]
    pub has_known_backend_contract: Option<bool>,
    #[serde(default)]
    pub orchestration_possible: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DecisionProposeParams {
    pub project: PathBuf,
    pub problem: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub capability_ids: Vec<String>,
    #[serde(default)]
    pub needs_atomic_backend: Option<bool>,
    #[serde(default)]
    pub has_known_backend_contract: Option<bool>,
    #[serde(default)]
    pub orchestration_possible: Option<bool>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DecisionChooseParams {
    pub project: PathBuf,
    pub decision_id: String,
    pub option_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DecisionApproveParams {
    pub project: PathBuf,
    pub decision_id: String,
    pub approval_evidence_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct PlanBuildParams {
    pub project: PathBuf,
    /// How far the redesign may go. When omitted: the variant's mode in a variant, else `redesign`. See redesign_modes_list.
    #[serde(default)]
    pub mode: Option<super::redesign_mode::RedesignMode>,
    pub mappings: Vec<super::redesign_plan::PlanMapping>,
    #[serde(default)]
    pub unknowns: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DsIngestParams {
    pub project: PathBuf,
    /// Path to a React/TS design-system package directory (with package.json).
    pub package_dir: PathBuf,
    /// `strict` or `advisory` (default strict).
    #[serde(default)]
    pub policy: Option<String>,
    /// Whether Layout CLI MCP is reachable (optional; does not override local checks).
    #[serde(default)]
    pub layout_cli_available: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DsCheckParams {
    pub project: PathBuf,
    /// Source files to check: [{ "path": "app/App.tsx", "content": "..." }] or omit to scan app/ for .tsx/.ts/.jsx/.js.
    #[serde(default)]
    pub sources: Option<Vec<DsSourceFile>>,
    /// If set, registry package_version must match.
    #[serde(default)]
    pub required_package_version: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct DsSourceFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct KnowledgeRetrieveParams {
    /// Task or screen description to match against packs.
    pub query: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Desired rule count (clamped to 3–8).
    #[serde(default)]
    pub limit: Option<usize>,
    /// Optional extra pack JSON files (must include version + allowed license).
    #[serde(default)]
    pub pack_paths: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CriticParams {
    /// Problem statement for the screen or task.
    pub problem: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub pack_paths: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct StudyMapParams {
    pub project: PathBuf,
    /// Also map Craft capture/ routes when present.
    #[serde(default = "default_true")]
    pub map_capture: bool,
    /// Optional path to an independent fixture oracle JSON.
    #[serde(default)]
    pub oracle_path: Option<PathBuf>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StartParams {
    /// What to make (a new project), or the follow-up message (with `project`, or a
    /// `project_folder` that already holds one).
    pub request: String,
    /// For a new project: a web address, an absolute path to an app, game or file, or `-` for none.
    /// A `project_folder` that already holds one keeps its saved target.
    #[serde(default)]
    pub target: Option<String>,
    /// Continue this project folder instead of starting a new one.
    #[serde(default)]
    pub project: Option<PathBuf>,
    /// With `project`: carry on where its last run stopped (paused, or the app or the computer
    /// stopped) instead of sending `request` as a follow-up.
    #[serde(default)]
    pub resume: bool,
    /// Where a new project's folder goes (default: the projects folder from settings).
    #[serde(default)]
    pub projects_dir: Option<PathBuf>,
    /// Start the new project in this folder itself instead (it's added to the project list). If
    /// it already has files, `existing` says what they are; if it already holds a project, its
    /// saved target, settings and stack are kept and `request` becomes its follow-up.
    #[serde(default)]
    pub project_folder: Option<PathBuf>,
    /// With `project_folder` that has files and isn't a project yet: `edit` (they're the app,
    /// edited in place) or `start_from` (material to start from; the app is built in app/ beside
    /// them).
    #[serde(default)]
    pub existing: Option<craft_anything_engine::workspace::ExistingFiles>,
    /// What to build with: an engine (unity, unreal, godot, rust) or a stack's full name.
    #[serde(default)]
    pub build_with: Option<String>,
    /// Explicit project kind (`redesign`, `website`, `app`, `game`). Required for redesign;
    /// without it, the kind is detected from the request and target.
    #[serde(default)]
    pub kind: Option<String>,
    /// With `controller=host_agent`, allow this call to hand the builder to Craft's runner.
    /// Default false: nested rebuild_start is refused (`controller_conflict`).
    #[serde(default)]
    pub handoff_to_craft_runner: bool,
    /// For a new game: the builder may generate pictures and 3D models (paid from the person's
    /// asset provider key, within the project's budget). Default false; change it later with
    /// project_assets_allow.
    #[serde(default)]
    pub generate_assets: bool,
    /// The coding agent to use (an enabled backend name), kept with a new project for its later
    /// changes; default: the one set up for building.
    #[serde(default)]
    pub backend: Option<String>,
    /// Refinement rounds after the first build (default 2).
    #[serde(default)]
    pub rounds: Option<u32>,
    /// Pictures of the original (absolute paths): the look the person expects. They're copied
    /// into the project's reference/ folder for the builder.
    #[serde(default)]
    pub pictures: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CheckParams {
    /// The project's folder; default: the project this tool server was started for.
    #[serde(default)]
    pub project: Option<PathBuf>,
    /// Longest wait for the result, in minutes (1..=60, default 35): a website with several pages
    /// or a game's engine build takes a few minutes, and the run stops a check at 30.
    #[serde(default = "d_wait_minutes")]
    pub wait_minutes: u64,
}

/// Longer than the run's own limit on the check ([`self_check::SELF_CHECK_LIMIT`]), so a check
/// that hangs is stopped and reported by the run before the builder stops waiting for it.
fn d_wait_minutes() -> u64 {
    self_check::SELF_CHECK_LIMIT.as_secs() / 60 + 5
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuilderParams {
    /// The project's folder.
    pub project: PathBuf,
    /// An enabled AI backend's name; `default` gives the project back to the app's choice. None
    /// just reads what applies.
    #[serde(default)]
    pub builder: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectParams {
    /// The project's folder.
    pub project: PathBuf,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActivityParams {
    /// The project's folder.
    pub project: PathBuf,
    /// How many lines to return (200 by default, at most 2000).
    #[serde(default)]
    pub limit: Option<usize>,
    /// How many lines back from the newest to end at (0 by default).
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NoteParams {
    /// The project's folder.
    pub project: PathBuf,
    /// What to tell the builder.
    pub text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WithdrawParams {
    /// The project's folder.
    pub project: PathBuf,
    /// The note's id, as rebuild_note returned it.
    pub id: u64,
}

/// Builds started by tools in this session, by project folder.
struct Runs(Arc<Mutex<RunSet>>);

/// How often queued builds are checked for a free place between tool calls.
const QUEUE_TICK: Duration = Duration::from_secs(2);

fn runs(ctx: &mut Ctx<'_>) -> Arc<Runs> {
    match ctx.session.ext.get::<Runs>() {
        Some(r) => r,
        None => {
            let r = Arc::new(Runs(Arc::new(Mutex::new(RunSet::new(ctx.session.settings.build.parallel)))));
            ctx.session.ext.insert(r.clone());
            r
        }
    }
}

/// The engine the rebuild tools were registered on.
fn engine(ctx: &Ctx<'_>) -> Option<Arc<Engine>> {
    ctx.session.ext.get::<EngineRef>().and_then(|r| r.0.upgrade())
}

/// Bring every run up to date and start queued ones in free places. Events are kept in each
/// project's feed by the run itself, so they're dropped here.
fn refresh(ctx: &Ctx<'_>, set: &mut RunSet) {
    set.set_limit(ctx.session.settings.build.parallel);
    if let Some(engine) = engine(ctx) {
        let hub = AiHub::with_stores(engine, ctx.session.config_dir().join("config.toml"), crate::KeyStore::system());
        let _ = set.poll(&hub);
    }
}

fn open(project: &Path) -> Result<Workspace, ToolError> {
    Workspace::open(project).map_err(|e| {
        ToolError::new(ErrorCode::NotFound, format!("{} isn't a project folder: {}", project.display(), e.message))
            .with_remediation("Pass a folder from project_list (it holds craft.json).")
    })
}

/// The question tool's id.
pub const QUESTION_TOOL: &str = "rebuild_question_ask";

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuestionParams {
    /// The project's folder.
    pub project: PathBuf,
    /// The question, in plain words.
    pub question: String,
}

fn rebuild_question_ask(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: QuestionParams = parse_params(params)?;
    let engine = engine(ctx).ok_or_else(|| {
        ToolError::new(ErrorCode::FeatureMissing, "Questions about a build aren't available in this engine.")
            .with_remediation("Use the app or `craft-anything-cli`.")
    })?;
    let ws = open(&p.project)?;
    let after = craft_anything_engine::feed::load(&ws).len();
    ctx.report(None, "Asking…");
    // The session is this call's while it runs: the question gets its settings from here.
    let mut settings = ctx.session.settings.clone();
    settings.deep_mode = !ctx.session.clean_room();
    let answered =
        super::question::ask(&AiHub::new(engine), &settings, Some(crate::failure_log::dir_in(&ctx.session.config_dir())), &ws, &p.question, &ctx.cancel)?;
    let question = p.question.trim();
    let from_session = answered.from == super::question::AnsweredFrom::BuildersSession;
    let asked = AskedQuestion {
        deep: settings.deep_mode,
        ..AskedQuestion::new(question.to_string(), answered.answer.clone(), from_session, answered.backend.clone(), answered.cost_usd, after)
    };
    record_question_usage(&ws, &answered);
    // The replacement notice is for this caller; the saved question keeps the AI that answered.
    // An answer that couldn't be kept is still the answer.
    let not_kept = questions::add(&ws, asked).err().map(|e| e.message);
    Ok(ToolOutput::Value(json!({"project": ws.root, "question": question, "answered": answered, "not_kept": not_kept})))
}

/// Add a question's reported cost to the project's usage, as a build turn's cost is added.
fn record_question_usage(ws: &Workspace, answered: &super::question::Answered) {
    let Some(cost) = answered.cost_usd.filter(|cost| cost.is_finite() && *cost > 0.0) else { return };
    if let Err(e) = ProjectUsage::update(&ws.root, |u| u.add_turn(0, 0, Some(cost), answered.cost_is_estimate)) {
        log::warn!("Couldn't record the question's cost in {}: {}", ws.root.display(), e.message);
    }
}

/// What builds a project's later changes. A name is checked first, so a refused choice leaves
/// the project's own choice as it was.
fn project_builder_set(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: BuilderParams = parse_params(params)?;
    let cfg = crate::AiConfig::from_settings(&ctx.session.settings);
    let mut ws = open(&p.project)?;
    if let Some(name) = p.builder.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        if name.eq_ignore_ascii_case("default") {
            ws.set_person(|m| m.builder = None)?;
        } else {
            let entry = builder_backend(&cfg, Some(name))?;
            ws.set_person(|m| m.builder = Some(entry.config.name.clone()))?;
        }
    }
    // With no AI able to build, the choice is still read (and `default` still clears it): the
    // answer says so with `builds: null` instead of failing after the project changed.
    let builds = builder_backend(&cfg, project_builder(&cfg, &ws)).ok().map(|b| b.config.name);
    Ok(ToolOutput::Value(json!({
        "project": ws.root,
        "builder": ws.manifest.builder,
        "builds": builds,
    })))
}

fn rebuild_questions_list(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let questions = questions::load(&open(&p.project)?);
    Ok(ToolOutput::Value(json!({"count": questions.len(), "questions": questions})))
}

/// Start a build: a new project, or a follow-up on an existing one, through the run queue.
fn rebuild_start(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: StartParams = parse_params(params)?;
    // MCP / host-agent default: refuse nested Craft builder without explicit handoff.
    let policy = ctx.session.ext.get::<super::ExecutionPolicy>().as_deref().cloned().unwrap_or_default();
    policy.guard_rebuild_start(p.handoff_to_craft_runner)?;
    let engine = engine(ctx).ok_or_else(|| {
        ToolError::new(ErrorCode::FeatureMissing, "Building isn't available in this engine.").with_remediation("Use the app or `craft-anything-cli rebuild`.")
    })?;
    let runs = runs(ctx);
    let cfg = crate::AiConfig::from_settings(&ctx.session.settings);
    // Refuse an unknown or switched-off name before opening or making a project or adding pictures:
    // a refused start must not leave anything behind.
    if let Some(name) = p.backend.as_deref() {
        builder_backend(&cfg, Some(name))?;
    }
    let (ws, message) = match &p.project {
        Some(dir) => {
            if p.build_with.is_some() {
                return Err(ToolError::invalid("build_with is for new projects; a project keeps the stack it was built with."));
            }
            if p.generate_assets {
                return Err(ToolError::invalid("generate_assets is for new projects.").with_remediation("Use project_assets_allow for an existing project."));
            }
            let ws = open(dir)?;
            ctx.session.remember_project(&ws.root)?;
            (ws, (!p.resume).then(|| p.request.clone()))
        }
        None if p.resume => return Err(ToolError::invalid("resume is for an existing project.").with_remediation("Pass its folder as `project`.")),
        None => {
            let attached = p.project_folder.as_deref().is_some_and(project::has_project);
            if attached {
                if p.existing.is_some() {
                    return Err(ToolError::invalid("This folder already holds a project; pass it as `project` to continue it."));
                }
                if p.build_with.is_some() {
                    return Err(ToolError::invalid("build_with is for new projects; a project keeps the stack it was built with."));
                }
                if p.generate_assets {
                    return Err(
                        ToolError::invalid("generate_assets is for new projects.").with_remediation("Use project_assets_allow for an existing project.")
                    );
                }
            }
            let target = p.target.as_deref().unwrap_or("-");
            let place = match (&p.project_folder, &p.projects_dir) {
                (Some(_), Some(_)) => return Err(ToolError::invalid("Pass projects_dir or project_folder, not both.")),
                (Some(f), None) => project::Place::At(f, p.existing),
                (None, Some(d)) => project::Place::In(d),
                (None, None) => project::Place::Projects,
            };
            let ws = project::new_project(
                &ctx.session.settings,
                target,
                &p.request,
                place,
                project::NewProjectOpts {
                    build_with: p.build_with.as_deref(),
                    kind: p.kind.as_deref(),
                    generate_assets: p.generate_assets,
                    builder: p.backend.as_deref(),
                },
            )?;
            if p.project_folder.is_some() || matches!(place, project::Place::At(..)) {
                ctx.session.remember_project(&ws.root)?;
            }
            (ws, project::attached_request(attached, &p.request))
        }
    };
    if !p.pictures.is_empty() {
        ws.add_references(&p.pictures)?;
    }
    // A headless start can't show the app's toast: say which AI builds when the saved one is gone.
    let builder_notice =
        p.project.as_ref().and_then(|_| super::builder_replacement(&cfg, &ws, p.backend.as_deref()).map(|r| super::replaced_builder_notice(&r.saved, &r.used)));
    let mut set = runs.0.lock().unwrap_or_else(PoisonError::into_inner);
    refresh(ctx, &mut set);
    if set.is_busy(&ws.root) {
        return Err(ToolError::new(ErrorCode::BadInput, "This project is already building.")
            .with_remediation("Add to it with rebuild_note, or stop it with rebuild_stop."));
    }
    let root = ws.root.clone();
    let opts = RunOptions {
        backend: p.backend,
        shown: message.clone(),
        message,
        continuing: p.resume,
        max_rounds: p.rounds.unwrap_or(2),
        beginner: false,
        force_kind: p.kind.or_else(|| ws.manifest.kind.clone()),
        clean_room: Some(ctx.session.clean_room()),
        ..RunOptions::default()
    };
    // A tool runs with the session locked, so the hub reads the saved file: use this session's
    // config file (the person's own in the app), not the machine's default one.
    let hub = AiHub::with_stores(engine, ctx.session.config_dir().join("config.toml"), crate::KeyStore::system());
    let mut out = match set.start(&hub, ws, opts)? {
        Started::Running => json!({"project": root.display().to_string(), "started": true}),
        Started::Queued { position } => {
            // It starts when a place comes free, whether or not another tool is called.
            drop(set);
            runs::keep_queue_moving(&runs.0, &hub.engine, ctx.session.config_dir().join("config.toml"), QUEUE_TICK);
            json!({"project": root.display().to_string(), "started": false, "queued": position})
        }
    };
    if let Some(notice) = builder_notice {
        out["builder_notice"] = json!(notice);
    }
    Ok(ToolOutput::Value(out))
}

fn rebuild_status(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let runs = runs(ctx);
    let mut set = runs.0.lock().unwrap_or_else(PoisonError::into_inner);
    refresh(ctx, &mut set);
    let m = &ws.manifest;
    Ok(ToolOutput::Value(json!({
        "project": ws.root.display().to_string(),
        "running": set.is_running(&ws.root),
        "queued": set.position(&ws.root),
        "status": m.status,
        "status_label": m.status.label(),
        "note": m.note,
        "last_check": m.checks.last(),
        "waiting": Inbox::new(ws.inbox_dir()).pending().len(),
    })))
}

fn rebuild_activity_show(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ActivityParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let limit = p.limit.unwrap_or(200).min(2000);
    let (lines, total) = super::activity::read_log(&ws.root, p.offset.unwrap_or(0), limit)?;
    Ok(ToolOutput::Value(json!({
        "project": ws.root.display().to_string(),
        "log": ws.root.join(craft_anything_engine::workspace::ACTIVITY_LOG).display().to_string(),
        "lines": lines,
        "total_lines_read": total,
    })))
}

fn rebuild_note(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: NoteParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let runs = runs(ctx);
    let mut set = runs.0.lock().unwrap_or_else(PoisonError::into_inner);
    refresh(ctx, &mut set);
    match set.get(&ws.root).filter(|r| r.is_running()) {
        Some(run) => {
            let id = run.add_note(p.text.clone(), p.text, Vec::new());
            Ok(ToolOutput::Value(json!({"id": id, "queued": run.queued()})))
        }
        None => Err(ToolError::new(ErrorCode::BadInput, "No build is running for this project in this session.")
            .with_remediation("Start a follow-up with rebuild_start (pass project and your message).")),
    }
}

fn rebuild_note_withdraw(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: WithdrawParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let runs = runs(ctx);
    let mut set = runs.0.lock().unwrap_or_else(PoisonError::into_inner);
    refresh(ctx, &mut set);
    match set.get(&ws.root).and_then(|r| r.unsend(p.id)) {
        Some(note) => Ok(ToolOutput::Value(json!({"withdrawn": note.text}))),
        None => Err(ToolError::new(ErrorCode::BadInput, format!("Note {} isn't waiting: the agent has read it, or it isn't from this session.", p.id))),
    }
}

fn rebuild_stop(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let runs = runs(ctx);
    let mut set = runs.0.lock().unwrap_or_else(PoisonError::into_inner);
    refresh(ctx, &mut set);
    match set.stop(&ws.root) {
        Stopped::Stopping => Ok(ToolOutput::Value(json!({"stopping": ws.root.display().to_string()}))),
        Stopped::Unqueued => Ok(ToolOutput::Value(json!({"unqueued": ws.root.display().to_string()}))),
        Stopped::Nothing => Err(ToolError::new(ErrorCode::BadInput, "No build is running for this project in this session.")),
    }
}

/// Ask the project's running build to check the current build (see [`super::self_check`]) and
/// wait for what it found.
fn rebuild_check(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: CheckParams = parse_params(params)?;
    if !(1..=60).contains(&p.wait_minutes) {
        return Err(ToolError::invalid("wait_minutes must be 1..=60"));
    }
    // The tool server is started with the project's inbox (<project>/inbox).
    let project = p
        .project
        .or_else(|| ctx.session.ext.get::<InboxDir>().and_then(|d| d.0.parent().map(Path::to_path_buf)))
        .ok_or_else(|| ToolError::invalid("No project given.").with_remediation("Pass project: the project's folder (it holds craft.json)."))?;
    let ws = open(&project)?;
    if self_check::building_here(&ws.root) {
        return Err(ToolError::new(ErrorCode::FeatureMissing, "This builder can't check its own work while it builds.")
            .with_remediation("Finish the turn: Craft Anything checks the build after it."));
    }
    let asks = Inbox::new(self_check::asks_dir(&ws.root));
    let id = asks.post(craft_anything_engine::inbox::RequestKind::Question, "Check the build", "", Vec::new(), false, None)?;
    ctx.report(None, "Checking the build against the original…");
    let cancel = ctx.cancel.clone();
    let answer = asks.wait(&id, Duration::from_secs(p.wait_minutes.saturating_mul(60)), &|| cancel.is_cancelled());
    // The run marks a request it has taken: without the mark, no run took it.
    let mark = self_check::taken_mark(&ws.root, &id);
    let taken = mark.as_ref().is_some_and(|m| m.is_file());
    if let Some(mark) = &mark {
        let _ = std::fs::remove_file(mark);
    }
    match answer {
        Some(a) => Ok(ToolOutput::Value(json!({"project": ws.root.display().to_string(), "report": a.text.unwrap_or_default()}))),
        None if ctx.cancel.is_cancelled() => Err(ToolError::new(ErrorCode::Cancelled, "Stopped.")),
        None => Err(unanswered(taken, p.wait_minutes)),
    }
}

fn redesign_project_init(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: RedesignInitParams = parse_params(params)?;
    let target = p.target.as_deref().unwrap_or("-");
    let place = match (&p.project_folder, &p.projects_dir) {
        (Some(_), Some(_)) => return Err(ToolError::invalid("Pass projects_dir or project_folder, not both.")),
        (Some(f), None) => project::Place::At(f, None),
        (None, Some(d)) => project::Place::In(d),
        (None, None) => project::Place::Projects,
    };
    let opts = project::NewProjectOpts { build_with: p.build_with.as_deref(), kind: Some(super::RedesignKind::ID), ..Default::default() };
    let ws = project::new_project(&ctx.session.settings, target, &p.request, place, opts)?;
    let _ = super::redesign_store::ensure(&ws)?;
    if p.project_folder.is_some() {
        ctx.session.remember_project(&ws.root)?;
    }
    let kind = ws.manifest.kind.clone().unwrap_or_else(|| super::RedesignKind::ID.into());
    Ok(ToolOutput::Value(json!({
        "project": ws.root.display().to_string(),
        "kind": kind,
        "stack": ws.manifest.stack,
        "status": ws.manifest.status,
        "request": ws.manifest.request,
        "steps": [
            "study (reuse Craft web capture when target is a URL)",
            "diagnose / keep_as_is options",
            "build React+TypeScript in app/",
            "check (build; no pixel-similarity gate)",
            "ready report"
        ],
        "builder_started": false
    })))
}

fn redesign_project_show(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let _ = ctx;
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let kinds = KindRegistry::built_in();
    let kind = project::resolve_kind(&kinds, &ws, None);
    let kind_id = kind.as_ref().map(|k| k.id()).unwrap_or("unknown");
    let check_ids: Vec<&str> = kind.as_ref().map(|k| k.checks(&ws).iter().map(|c| c.id()).collect()).unwrap_or_default();
    Ok(ToolOutput::Value(json!({
        "project": ws.root.display().to_string(),
        "name": ws.manifest.name,
        "kind": kind_id,
        "manifest_kind": ws.manifest.kind,
        "stack": ws.manifest.stack,
        "status": ws.manifest.status,
        "note": ws.manifest.note,
        "request": ws.manifest.request,
        "checks": ws.manifest.checks,
        "kind_checks": check_ids,
        "steps": [
            "study",
            "diagnose",
            "build",
            "check",
            "ready"
        ]
    })))
}

fn redesign_kinds_list(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let _: EmptyParams = parse_params(params)?;
    let kinds = KindRegistry::built_in();
    Ok(ToolOutput::Value(json!({
        "kinds": kinds.ids(),
        "redesign_default_stack": super::RedesignKind::DEFAULT_STACK,
        "explicit_selection": "pass kind=redesign to redesign_project_init, rebuild_start, or craft-anything-cli rebuild --kind redesign"
    })))
}

fn redesign_store_show(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    Ok(ToolOutput::Value(super::redesign_store::show(&ws)?))
}

fn redesign_evidence_put(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: EvidencePutParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let env = super::redesign_store::put_evidence(&ws, p.record)?;
    Ok(ToolOutput::Value(json!({"envelope": env})))
}

fn redesign_capability_put(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: CapabilityPutParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let env = super::redesign_store::put_capability(&ws, p.capability)?;
    Ok(ToolOutput::Value(json!({"envelope": env})))
}

fn redesign_capabilities_list(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ListParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let mut caps = super::redesign_store::list_capabilities(&ws)?;
    let total = caps.len();
    let offset = p.offset.unwrap_or(0).min(total);
    let limit = p.limit.unwrap_or(50).min(200);
    if offset < total {
        caps = caps.into_iter().skip(offset).take(limit).collect();
    } else {
        caps.clear();
    }
    Ok(ToolOutput::Value(json!({"total": total, "offset": offset, "limit": limit, "capabilities": caps})))
}

fn redesign_coverage_show(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    Ok(ToolOutput::Value(serde_json::to_value(super::redesign_study::coverage_of(&ws)?).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_identity_record(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: IdentityRecordParams = parse_params(params)?;
    let ws = open(&p.project)?;
    Ok(ToolOutput::Value(super::redesign_identity::record(&ws, p.traits, p.replace_all, p.notes)?))
}

fn redesign_identity_show(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    Ok(ToolOutput::Value(super::redesign_identity::show(&ws)?))
}

fn redesign_modes_list(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let _: EmptyParams = parse_params(params)?;
    Ok(ToolOutput::Value(super::redesign_mode::list()))
}

fn redesign_variants_create(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: VariantsCreateParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let out = super::redesign_variants::create(&ws, &p.modes)?;
    // Variants sit beside a project that may be outside the project locations: keep them listed too.
    for v in out["variants"].as_array().into_iter().flatten().filter_map(|v| v["project"].as_str()) {
        ctx.session.remember_project(std::path::Path::new(v))?;
    }
    Ok(ToolOutput::Value(out))
}

fn redesign_variants_show(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    Ok(ToolOutput::Value(super::redesign_variants::show(&ws)?))
}

fn redesign_prototype_capture(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: PrototypeCaptureParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let engine = engine(ctx).ok_or_else(|| {
        ToolError::new(ErrorCode::FeatureMissing, "Capturing a prototype isn't available in this engine.")
            .with_remediation("Use the app or `craft-anything-cli`.")
    })?;
    let extra: Vec<(String, String)> = p.pages.into_iter().map(|x| (x.page, x.path)).collect();
    let targets = super::redesign_likeness::capture_targets(&ws, &extra)?;
    let widths = p.widths.unwrap_or_else(|| super::redesign_likeness::original_widths(&ws));
    let server = super::preview(&ws)
        .map_err(|e| e.with_remediation("Build the prototype first (for the scaffold: npm install and npm run build in app/), then call this again."))?;
    let base = server.url();
    // The browser tools are run as this call's own steps: the session is this call's while it runs,
    // so asking the engine for them would wait on it. This tool's effects are theirs, so the same
    // approval was asked for.
    let step = |ctx: &mut Ctx<'_>, id: &str, args: Value| -> Result<ToolOutput, ToolError> {
        let spec = engine.registry().get(id).ok_or_else(|| ToolError::internal(format!("{id} is not registered")))?;
        (spec.run)(ctx, args)
    };
    let (mut captured, mut not_captured) = (Vec::new(), Vec::new());
    for (i, (page, path)) in targets.iter().enumerate() {
        ctx.report(Some(i as f32 / targets.len() as f32), format!("Capturing the prototype's {page} page…"));
        let done = super::redesign_likeness::fresh_page_dir(&ws, page).and_then(|out| {
            step(ctx, "web_site_open", json!({"url": format!("{}{}", base.trim_end_matches('/'), path)}))?;
            step(ctx, "web_site_capture", json!({"output_directory": out.display().to_string(), "widths": widths}))
        });
        // A page opened with only its `#…` changed doesn't load again, so each page gets its own browser.
        let _ = step(ctx, "web_site_close", json!({}));
        match done {
            Ok(_) => captured.push(json!({"page": page, "path": path})),
            Err(e) => not_captured.push(json!({"page": page, "path": path, "why": e.message})),
        }
    }
    drop(server);
    let mut out = super::redesign_likeness::measure(&ws)?;
    out["captured"] = json!(captured);
    out["not_captured"] = json!(not_captured);
    out["widths"] = json!(widths);
    Ok(ToolOutput::Value(out))
}

fn redesign_likeness_measure(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    Ok(ToolOutput::Value(super::redesign_likeness::measure(&ws)?))
}

fn redesign_diagnose(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: DiagnoseParams = parse_params(params)?;
    Ok(ToolOutput::Value(super::redesign_plan::diagnose(
        &p.problem,
        &p.tags,
        &p.capability_ids,
        p.needs_atomic_backend.unwrap_or(false),
        p.has_known_backend_contract.unwrap_or(false),
        p.orchestration_possible.unwrap_or(false),
    )?))
}

fn redesign_decision_propose(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: DecisionProposeParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let (feasibility, deps) = super::redesign_plan::classify_feasibility(
        p.needs_atomic_backend.unwrap_or(false),
        p.has_known_backend_contract.unwrap_or(false),
        p.orchestration_possible.unwrap_or(false),
    );
    let packs = super::redesign_knowledge::bundled_catalog();
    let rules = super::redesign_knowledge::retrieve(&p.problem, &p.tags, &packs, super::redesign_knowledge::RETRIEVE_MAX)?;
    let critic = super::redesign_knowledge::critic(&p.problem, &rules);
    let dec = super::redesign_plan::propose_decision(&p.problem, &p.capability_ids, &critic, feasibility, &deps, &p.evidence_ids);
    super::redesign_plan::save_decision(&ws, &dec)?;
    Ok(ToolOutput::Value(serde_json::to_value(dec).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_decision_choose(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: DecisionChooseParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let dec = super::redesign_plan::choose_option(&ws, &p.decision_id, &p.option_id)?;
    Ok(ToolOutput::Value(serde_json::to_value(dec).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_decision_approve(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: DecisionApproveParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let dec = super::redesign_plan::approve_decision(&ws, &p.decision_id, &p.approval_evidence_id)?;
    Ok(ToolOutput::Value(serde_json::to_value(dec).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_plan_build(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: PlanBuildParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let variant = super::redesign_variants::variant_of(&ws)?.map(|v| v.mode);
    if let (Some(asked), Some(made_for)) = (p.mode, variant)
        && asked != made_for
    {
        return Err(ToolError::invalid(format!(
            "this project is the `{}` variant; its plan can't use mode `{}`. Build that mode's plan in its own variant.",
            made_for.as_str(),
            asked.as_str()
        )));
    }
    let plan = super::redesign_plan::build_plan(&ws, p.mode.or(variant).unwrap_or_default().as_str(), p.mappings, p.unknowns)?;
    Ok(ToolOutput::Value(serde_json::to_value(plan).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_plan_show(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let plan = super::redesign_plan::load_plan(&ws)?;
    Ok(ToolOutput::Value(serde_json::to_value(plan).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_traceability_check(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let caps = super::redesign_store::list_capabilities(&ws)?;
    let plan = super::redesign_plan::load_plan(&ws)?;
    let report = super::redesign_plan::traceability_check(&ws, &caps, &plan)?;
    Ok(ToolOutput::Value(serde_json::to_value(report).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_scaffold_app(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let policy = ctx.session.ext.get::<super::ExecutionPolicy>().as_deref().cloned();
    Ok(ToolOutput::Value(super::redesign_build::scaffold_app(&ws, policy.as_ref())?))
}

fn redesign_checks_run(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let report = super::redesign_checks::run_native_checks(&ws)?;
    Ok(ToolOutput::Value(serde_json::to_value(report).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_quality_run(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: QualityRunParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let checks = super::redesign_quality::run_quality(&ws, p.oracle_path.as_deref())?;
    Ok(ToolOutput::Value(json!({"checks": checks})))
}

fn redesign_fixture_smoke(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: FixtureSmokeParams = parse_params(params)?;
    let fixture_id = p.fixture_id.as_deref().unwrap_or("dense-form");
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture_root = repo.join("REDESIGN_SPEC/fixtures").join(fixture_id);
    let ds = repo.join("REDESIGN_SPEC/fixtures/design-system");
    let oracle = fixture_root.join("oracle/scenarios.json");
    if !oracle.is_file() {
        return Err(ToolError::invalid(format!("fixture oracle missing: {}", oracle.display())));
    }
    let projects = p.projects_dir.unwrap_or_else(|| std::env::temp_dir().join("redesign-smoke"));
    std::fs::create_dir_all(&projects).map_err(|e| ToolError::io(projects.display(), e))?;
    let mut ws = project::new_project(
        &craft_anything_engine::Settings::default(),
        "-",
        &format!("Smoke redesign fixture {fixture_id}"),
        project::Place::In(&projects),
        project::NewProjectOpts { kind: Some(super::RedesignKind::ID), ..Default::default() },
    )?;
    ws.manifest.kind = Some(super::RedesignKind::ID.into());
    ws.save()?;
    super::redesign_ds::ingest_package(&ws, &ds, "strict", false)?;
    super::redesign_build::scaffold_app(&ws, Some(&super::ExecutionPolicy::host_agent()))?;
    // Enrich scaffold with oracle keywords so static scenario eval can exercise pass/fail honestly.
    let app_tsx = ws.app_dir().join("src/App.tsx");
    let mut body = std::fs::read_to_string(&app_tsx).unwrap_or_default();
    if !body.to_ascii_lowercase().contains("submit") {
        body.push_str(
            "\nexport function SmokeFormHints() {\n  return (\n    <section>\n      <label>Amount <input name=\"amount\" /></label>\n      <button type=\"submit\">Submit</button>\n      <button type=\"button\">Approve</button>\n      <p role=\"status\">Role: employee</p>\n      <p>No attachments yet.</p>\n    </section>\n  );\n}\n",
        );
        std::fs::write(&app_tsx, body).map_err(|e| ToolError::io(app_tsx.display(), e))?;
    }
    let oracle_dir = super::redesign_store::redesign_dir(&ws).join("oracle");
    std::fs::create_dir_all(&oracle_dir).map_err(|e| ToolError::io(oracle_dir.display(), e))?;
    std::fs::copy(&oracle, oracle_dir.join("scenarios.json")).map_err(|e| ToolError::io(oracle.display(), e))?;
    let _ = super::redesign_study::import_oracle_file(&ws, &oracle)?;
    let quality = super::redesign_quality::run_quality(&ws, Some(&oracle))?;
    let ready = super::redesign_checks::run_native_checks(&ws)?;
    let report = super::redesign_report::write_reports(&ws, Some(fixture_id), &ready, &quality, &json!({"nested_builder_started": false}))?;
    Ok(ToolOutput::Value(json!({
        "fixture_id": fixture_id,
        "project": ws.root,
        "quality": quality,
        "ready": ready,
        "report": report,
        "preview": {
            "app": "app/",
            "hint": "cd app && npm install && npm run dev",
            "html_report": report.get("html"),
        },
        "nested_builder_started": false,
        "note": "Offline fixture smoke with exportable report; not a published impact benchmark.",
    })))
}

fn redesign_mcp_catalogue(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: McpCatalogueParams = parse_params(params)?;
    let tools: Vec<(String, String, bool, Value)> =
        p.tools.into_iter().map(|t| (t.name, t.description, t.declared_read_only.unwrap_or(false), t.schema.unwrap_or(json!({})))).collect();
    let profile = super::redesign_mcp_trust::catalogue_tools(&tools, &p.allowlist);
    let summary = super::redesign_mcp_trust::profile_summary(&profile);
    Ok(ToolOutput::Value(json!({"profile": profile, "summary": summary})))
}

fn redesign_mcp_gate(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: McpGateParams = parse_params(params)?;
    let decision = super::redesign_mcp_trust::gate_call(&p.profile, &p.tool, &p.body.unwrap_or(json!({})), p.previous_schema_hash.as_deref())?;
    Ok(ToolOutput::Value(serde_json::to_value(decision).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_mcp_fixture_smoke(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let _: EmptyParams = parse_params(params)?;
    Ok(ToolOutput::Value(super::redesign_mcp_trust::smoke_fixture_server()?))
}

fn redesign_benchmark_dry_run(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: BenchmarkDryRunParams = parse_params(params)?;
    let run = super::redesign_benchmark::dry_run_template(p.fixture_id.as_deref().unwrap_or("dense-form"));
    for arm in &run.arms {
        super::redesign_benchmark::validate_arm(arm).map_err(ToolError::invalid)?;
    }
    Ok(ToolOutput::Value(serde_json::to_value(run).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_benchmark_publish(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: BenchmarkPublishParams = parse_params(params)?;
    let fixture_id = p.fixture_id.as_deref().unwrap_or("dense-form");
    let notes = p.arm_c_notes.unwrap_or_else(|| vec!["automated fixture smoke / lean path".into(), "human_blind_eval=unavailable".into()]);
    let run = super::redesign_benchmark::publish_template(fixture_id, p.arm_c_wall_ms, notes);
    for arm in &run.arms {
        super::redesign_benchmark::validate_arm(arm).map_err(ToolError::invalid)?;
    }
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/benchmark/published");
    std::fs::create_dir_all(&out_dir).map_err(|e| ToolError::io(out_dir.display(), e))?;
    let json_path = out_dir.join("RESULTS.json");
    craft_anything_engine::project::write_atomic(&json_path, serde_json::to_vec_pretty(&run).map_err(|e| ToolError::internal(e.to_string()))?.as_slice())?;
    let md = format!(
        "# Benchmark publish — First public release package\n\n\
         - Fixture: `{fixture_id}`\n\
         - Human blind evaluation: **unavailable**\n\
         - Impact verdict: **unevaluated** (do not claim gain vs arm B)\n\
         - Machine record: `REDESIGN_SPEC/benchmark/published/RESULTS.json`\n\n\
         See `REDESIGN_SPEC/BENCHMARK_PROTOCOL.md`. Tokens stay unavailable without host telemetry.\n"
    );
    let md_path = out_dir.join("RESULTS.md");
    craft_anything_engine::project::write_atomic(&md_path, md.as_bytes())?;
    Ok(ToolOutput::Value(json!({
        "run": run,
        "paths": {
            "json": json_path.display().to_string(),
            "markdown": md_path.display().to_string(),
        },
        "impact_verdict": "unevaluated",
    })))
}

fn redesign_rea_discover(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ReaDiscoverParams = parse_params(params)?;
    let d = if p.probe_host.unwrap_or(false) || (p.node_ok.is_none() && p.rea_present.is_none()) {
        super::redesign_rea::discover_host()
    } else {
        super::redesign_rea::discover(p.node_ok.unwrap_or(false), p.rea_present.unwrap_or(false), p.rea_version)
    };
    Ok(ToolOutput::Value(json!({
        "discovery": d,
        "allowed_operations": super::redesign_rea::allowed_operations(),
    })))
}

fn redesign_rea_request(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ReaRequestParams = parse_params(params)?;
    let discovery = if p.probe_host.unwrap_or(false) || (p.node_ok.is_none() && p.rea_present.is_none()) {
        super::redesign_rea::discover_host()
    } else {
        super::redesign_rea::discover(p.node_ok.unwrap_or(false), p.rea_present.unwrap_or(false), p.rea_version.clone())
    };
    let mut session = super::redesign_rea::ReaSession::new(
        p.consent_granted.unwrap_or(false),
        p.clean_room.unwrap_or(false),
        p.deep_profile.unwrap_or(false),
        p.budget_calls.unwrap_or(3),
    );
    session.calls_used = p.calls_already_used.unwrap_or(0);
    let rec = super::redesign_rea::request_call(&mut session, &discovery, &p.operation, &p.reason)?;
    Ok(ToolOutput::Value(json!({"record": rec, "session": session, "discovery": discovery})))
}

fn redesign_rea_run(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ReaRunParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let discovery = super::redesign_rea::discover_host();
    let mut session = super::redesign_rea::ReaSession::new(
        p.consent_granted.unwrap_or(false),
        p.clean_room.unwrap_or(true), // fail closed: require explicit clean_room=false
        p.deep_profile.unwrap_or(false),
        p.budget_calls.unwrap_or(3),
    );
    session.calls_used = p.calls_already_used.unwrap_or(0);
    let result = super::redesign_rea::run_live(&ws, &mut session, &discovery, &p.operation, &p.reason, &p.input)?;
    Ok(ToolOutput::Value(serde_json::to_value(result).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_ds_ingest(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: DsIngestParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let policy = p.policy.as_deref().unwrap_or("strict");
    let layout = p.layout_cli_available.unwrap_or(false);
    let reg = super::redesign_ds::ingest_package(&ws, &p.package_dir, policy, layout)?;
    Ok(ToolOutput::Value(serde_json::to_value(reg).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_ds_show(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: ProjectParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let reg = super::redesign_ds::load_registry(&ws)?;
    Ok(ToolOutput::Value(serde_json::to_value(reg).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn redesign_ds_check(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: DsCheckParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let reg = super::redesign_ds::load_registry(&ws)?;
    let sources: Vec<(String, String)> =
        if let Some(list) = p.sources { list.into_iter().map(|s| (s.path, s.content)).collect() } else { scan_app_sources(&ws)? };
    let report = super::redesign_ds::check_sources(&reg, &sources, p.required_package_version.as_deref());
    Ok(ToolOutput::Value(serde_json::to_value(report).map_err(|e| ToolError::internal(e.to_string()))?))
}

fn scan_app_sources(ws: &Workspace) -> Result<Vec<(String, String)>, ToolError> {
    let app = ws.app_dir();
    let mut out = Vec::new();
    if !app.is_dir() {
        return Ok(out);
    }
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) -> Result<(), ToolError> {
        for ent in std::fs::read_dir(dir).map_err(|e| ToolError::io(dir.display(), e))? {
            let ent = ent.map_err(|e| ToolError::io(dir.display(), e))?;
            let path = ent.path();
            if path.is_dir() {
                walk(&path, root, out)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str())
                && matches!(ext, "ts" | "tsx" | "js" | "jsx" | "css")
            {
                let rel = path.strip_prefix(root).unwrap_or(&path).display().to_string();
                let content = std::fs::read_to_string(&path).map_err(|e| ToolError::io(path.display(), e))?;
                out.push((rel, content));
            }
        }
        Ok(())
    }
    walk(&app, &ws.root, &mut out)?;
    Ok(out)
}

fn redesign_knowledge_retrieve(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: KnowledgeRetrieveParams = parse_params(params)?;
    let limit = p.limit.unwrap_or(super::redesign_knowledge::RETRIEVE_MAX);
    Ok(ToolOutput::Value(super::redesign_knowledge::retrieve_report(&p.query, &p.tags, limit, &p.pack_paths)?))
}

fn redesign_critic_recommend(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: CriticParams = parse_params(params)?;
    let limit = p.limit.unwrap_or(super::redesign_knowledge::RETRIEVE_MAX);
    Ok(ToolOutput::Value(super::redesign_knowledge::critic_report(&p.problem, &p.tags, limit, &p.pack_paths)?))
}

fn redesign_study_map(_ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: StudyMapParams = parse_params(params)?;
    let ws = open(&p.project)?;
    let mut out = json!({});
    if p.map_capture {
        out["capture"] = super::redesign_study::map_capture(&ws)?;
    }
    if let Some(path) = p.oracle_path {
        out["oracle"] = super::redesign_study::import_oracle_file(&ws, &path)?;
    }
    out["coverage"] = serde_json::to_value(super::redesign_study::coverage_of(&ws)?).map_err(|e| ToolError::internal(e.to_string()))?;
    Ok(ToolOutput::Value(out))
}

/// Why `rebuild_check` got no report: a run took the check and it is still going (a step may be
/// stuck), or no run took it at all.
fn unanswered(taken: bool, minutes: u64) -> ToolError {
    if taken {
        ToolError::new(ErrorCode::TimedOut, format!("The running build took the check, but it hadn't finished after {minutes} minutes."))
            .with_remediation("A step of the check may be stuck (the project's feed says which step it's on). Don't ask again: finish the turn, and the build is checked after it, with a time limit.")
    } else {
        ToolError::new(ErrorCode::NotFound, format!("No build is running for this project: nothing took the check in {minutes} minutes."))
            .with_remediation("Use this while Craft Anything is building the project; otherwise build it and finish the turn, and it is checked after.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::{CallOptions, Caller, Consent, Engine};

    #[test]
    fn a_questions_cost_is_counted_in_the_projects_usage() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        let answered = |cost, estimate| super::super::question::Answered {
            answer: "It is a notes app.".into(),
            from: super::super::question::AnsweredFrom::BuildersSession,
            backend: "Claude".into(),
            cost_usd: cost,
            cost_is_estimate: estimate,
            builder_notice: None,
            builder_replacement: None,
        };
        let response = serde_json::to_value(answered(None, false)).unwrap();
        assert!(response.get("builder_notice").is_none() && response.get("builder_replacement").is_none());
        record_question_usage(&ws, &answered(Some(0.02), true));
        let u = ws.usage().known().unwrap();
        assert!((u.cost_usd - 0.02).abs() < 1e-9 && u.api_billed_usd == 0.0 && u.priced, "an agent's estimate: {u:?}");
        record_question_usage(&ws, &answered(Some(0.03), false));
        let u = ws.usage().known().unwrap();
        assert!((u.cost_usd - 0.05).abs() < 1e-9 && (u.api_billed_usd - 0.03).abs() < 1e-9, "a key's charge: {u:?}");
        record_question_usage(&ws, &answered(None, true));
        record_question_usage(&ws, &answered(Some(0.0), true));
        assert!((ws.usage().known().unwrap().cost_usd - 0.05).abs() < 1e-9, "no cost is no change");
    }

    #[test]
    fn a_question_through_the_engine_comes_back_instead_of_waiting_for_the_session() {
        // The tool runs while the call holds the session; reading it again would wait forever.
        let mut s = craft_anything_engine::testing::session();
        s.settings.ai =
            toml::from_str("[[backends]]\nname = \"Agent\"\nkind = \"agent_cli\"\nmodel = \"claude-code\"\ncommand = \"craft-anything-no-such-agent\"\n")
                .unwrap();
        let e = std::sync::Arc::new(Engine::shared(s, specs()));
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Snake", "A snake game", TargetRef::None).unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let e2 = e.clone();
        std::thread::spawn(move || {
            let r = e2.call(QUESTION_TOOL, json!({"project": ws.root, "question": "where are the files?"}), CallOptions::new(Caller::Cli));
            let _ = tx.send(r.map(|o| o.to_json()));
        });
        let r = rx.recv_timeout(std::time::Duration::from_secs(60)).expect("the question came back");
        // No such agent here: an error that says so, not a hang.
        assert!(r.is_err(), "{r:?}");
    }

    /// The capture runs the prototype's scripts in a browser, so it is asked for like the browser
    /// tools it uses, which it runs without going back through the engine.
    #[test]
    fn capturing_a_prototype_is_refused_until_the_person_approves_it() {
        let d = tempfile::tempdir().unwrap();
        // Any program stands in for the browser: the call must stop before one is started.
        let mut session = craft_anything_engine::testing::session();
        session.settings.engines.tools.chromium = Some(std::env::current_exe().unwrap());
        let e = Engine::shared(session, specs());
        let err = e.call("redesign_prototype_capture", json!({"project": d.path()}), CallOptions::new(Caller::Mcp)).unwrap_err();
        assert_eq!(err.code, ErrorCode::ConsentRequired, "{err:?}");
    }

    /// A built site that is both the original and the prototype is captured and found fully alike.
    /// Live (`--ignored`): needs a browser.
    #[test]
    #[ignore]
    fn a_built_prototype_is_captured_and_measured_in_one_call() {
        let d = tempfile::tempdir().unwrap();
        let e = Engine::shared(craft_anything_engine::testing::live_session(), specs());
        let ok = || CallOptions::new(Caller::Cli).with_consent(Consent::Granted);
        let mut ws = Workspace::create(d.path(), "Catalogue", "kind=redesign", craft_anything_engine::workspace::TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let dist = ws.app_dir().join("dist");
        std::fs::create_dir_all(&dist).unwrap();
        // A page drawn all the way down: stripes, so no band of it is one flat colour.
        let page = "<!doctype html><html><body style=\"margin:0\"><header style=\"height:80px;background:repeating-linear-gradient(90deg,#222 0 20px,#555 20px 40px)\"></header>\
                    <main style=\"min-height:700px;background:repeating-linear-gradient(90deg,#fff 0 30px,#ddd 30px 60px)\"><h1>Catalogue</h1><p>Books, films and recordings.</p>\
                    <ul><li>One</li><li>Two</li></ul></main><footer style=\"height:120px;background:repeating-linear-gradient(90deg,#888 0 20px,#aaa 20px 40px)\"></footer></body></html>";
        std::fs::write(dist.join("index.html"), page).unwrap();
        let project = ws.root.display().to_string();

        let first = e.call("redesign_prototype_capture", json!({"project": project, "widths": [800]}), ok()).unwrap().to_json();
        assert_eq!(first["result"]["captured"][0]["page"], "start", "{first}");
        // Its own capture becomes the original, so the next capture is compared with the same page.
        let made = ws.root.join("redesign/prototype_capture/start");
        std::fs::create_dir_all(ws.capture_dir()).unwrap();
        for f in ["capture.json", "screenshot-800.png"] {
            std::fs::copy(made.join(f), ws.capture_dir().join(f)).unwrap();
        }
        let again = e.call("redesign_prototype_capture", json!({"project": project}), ok()).unwrap().to_json();
        assert_eq!(again["result"]["widths"], json!([800]), "the original's widths are used: {again}");
        let start = &again["result"]["pages"][0];
        assert_eq!(
            (start["page"].as_str(), start["structure"].as_f64(), start["not_measured"].as_array().map(Vec::len)),
            (Some("start"), Some(100.0), Some(0)),
            "{again}"
        );
        assert!(start["picture"].as_f64() > Some(99.0), "{again}");
    }

    #[test]
    fn rebuild_tools_are_registered_from_the_ai_layer_and_check_their_input() {
        let d = tempfile::tempdir().unwrap();
        let e = Engine::shared(craft_anything_engine::testing::session(), specs());
        let ok = || CallOptions::new(Caller::Cli).with_consent(Consent::Granted);
        for id in [
            "rebuild_start",
            "rebuild_status",
            "rebuild_note",
            "rebuild_note_withdraw",
            "rebuild_stop",
            "rebuild_check",
            "redesign_project_init",
            "redesign_project_show",
            "redesign_kinds_list",
            "redesign_store_show",
            "redesign_evidence_put",
            "redesign_capability_put",
            "redesign_capabilities_list",
            "redesign_coverage_show",
            "redesign_study_map",
            "redesign_identity_record",
            "redesign_identity_show",
            "redesign_modes_list",
            "redesign_variants_create",
            "redesign_variants_show",
            "redesign_prototype_capture",
            "redesign_likeness_measure",
            "redesign_knowledge_retrieve",
            "redesign_critic_recommend",
            "redesign_ds_ingest",
            "redesign_ds_show",
            "redesign_ds_check",
            "redesign_diagnose",
            "redesign_decision_propose",
            "redesign_decision_choose",
            "redesign_decision_approve",
            "redesign_plan_build",
            "redesign_plan_show",
            "redesign_traceability_check",
            "redesign_scaffold_app",
            "redesign_checks_run",
            "redesign_quality_run",
            "redesign_fixture_smoke",
            "redesign_mcp_catalogue",
            "redesign_mcp_gate",
            "redesign_mcp_fixture_smoke",
            "redesign_rea_discover",
            "redesign_rea_request",
            "redesign_rea_run",
            "redesign_benchmark_dry_run",
            "redesign_benchmark_publish",
        ] {
            assert!(e.registry().get(id).is_some(), "{id}");
        }
        // Starting runs a coding agent: refused without consent, before anything is created.
        let p = json!({"request": "A page", "target": "-", "projects_dir": d.path()});
        assert!(e.call("rebuild_start", p, CallOptions::new(Caller::Mcp)).is_err());
        assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 0, "nothing created without consent");
        // A bad target or stack is refused before a project is made.
        assert!(e.call("rebuild_start", json!({"request": "x", "target": "not a path", "projects_dir": d.path()}), ok()).is_err());
        assert!(e.call("rebuild_start", json!({"request": "x", "build_with": "cobol", "projects_dir": d.path()}), ok()).is_err());
        // Status, notes and stop on a project with nothing running.
        let ws = Workspace::create(d.path(), "Snake", "A snake game", TargetRef::None).unwrap();
        // A project's activity reads through the tool, none yet being no lines, not an error.
        let shown = e.call("rebuild_activity_show", json!({"project": ws.root}), ok()).unwrap().to_json();
        assert_eq!(shown["result"]["lines"], json!([]), "{shown}");
        std::fs::write(ws.root.join(craft_anything_engine::workspace::ACTIVITY_LOG), "one\ntwo\nthree\n").unwrap();
        let shown = e.call("rebuild_activity_show", json!({"project": ws.root, "limit": 1, "offset": 1}), ok()).unwrap().to_json();
        assert_eq!(shown["result"]["lines"], json!(["two"]), "{shown}");
        let st = e.call("rebuild_status", json!({"project": ws.root}), ok()).unwrap().to_json();
        assert_eq!(st["result"]["running"], false);
        assert_eq!(st["result"]["queued"], Value::Null);
        assert!(e.call("rebuild_note", json!({"project": ws.root, "text": "blue"}), ok()).is_err());
        assert!(e.call("rebuild_note_withdraw", json!({"project": ws.root, "id": 1}), ok()).is_err());
        assert!(e.call("rebuild_stop", json!({"project": ws.root}), ok()).is_err());
        // Redesign init creates a project without starting a builder.
        let init = e
            .call(
                "redesign_project_init",
                json!({"request": "Redesign the expense form", "target": "https://example.test/expenses", "projects_dir": d.path()}),
                ok(),
            )
            .unwrap()
            .to_json();
        assert_eq!(init["result"]["kind"], "redesign", "{init}");
        assert_eq!(init["result"]["stack"], super::super::RedesignKind::DEFAULT_STACK, "{init}");
        assert_eq!(init["result"]["builder_started"], false, "{init}");
        let root = init["result"]["project"].as_str().unwrap();
        let shown = e.call("redesign_project_show", json!({"project": root}), ok()).unwrap().to_json();
        assert_eq!(shown["result"]["kind"], "redesign", "{shown}");
        assert!(shown["result"]["kind_checks"].as_array().unwrap().iter().all(|c| c != "web_visual"), "{shown}");
        let kinds = e.call("redesign_kinds_list", json!({}), ok()).unwrap().to_json();
        assert!(kinds["result"]["kinds"].as_array().unwrap().iter().any(|k| k == "redesign"), "{kinds}");
        // host_agent refuses nested rebuild_start without handoff.
        e.session().ext.insert(std::sync::Arc::new(super::super::ExecutionPolicy::host_agent()));
        let conflict = e.call("rebuild_start", json!({"request": "x", "target": "-", "projects_dir": d.path()}), ok()).unwrap_err();
        assert!(conflict.message.contains("controller_conflict"), "{}", conflict.message);
    }

    /// An existing project is attached, not treated as new: its saved settings stay,
    /// new-project-only options are refused, and the request becomes a follow-up.
    #[test]
    fn starting_at_a_folder_that_already_holds_a_project_attaches() {
        let d = tempfile::tempdir().unwrap();
        let e = Engine::shared(craft_anything_engine::testing::session(), specs());
        let ok = || CallOptions::new(Caller::Cli).with_consent(Consent::Granted);
        let mut ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        ws.set_person(|m| {
            m.builder = Some("Claude".into());
            m.asset_budget_usd = 42.0;
            m.generate_assets = true;
        })
        .unwrap();
        for (p, said) in [
            (json!({"project_folder": ws.root, "request": "change it", "build_with": "unity"}), "build_with is for new projects"),
            (json!({"project_folder": ws.root, "request": "change it", "generate_assets": true}), "generate_assets is for new projects"),
            (json!({"project_folder": ws.root, "request": "change it", "existing": "edit"}), "already holds a project"),
        ] {
            let err = e.call("rebuild_start", p, ok()).unwrap_err();
            assert!(err.message.contains(said), "{err:?}");
        }
        let saved = Workspace::open(&ws.root).unwrap().manifest;
        assert_eq!(saved.builder.as_deref(), Some("Claude"));
        assert_eq!(saved.asset_budget_usd, 42.0);
        assert!(saved.generate_assets);
    }

    /// The success path: an attached folder keeps its manifest settings, and its typed request
    /// lands in the run's feed as the follow-up.
    #[test]
    fn starting_at_an_existing_project_sends_its_request_as_a_follow_up() {
        let d = tempfile::tempdir().unwrap();
        let mut s = craft_anything_engine::testing::session();
        let mut ai = crate::AiConfig::default();
        ai.upsert(crate::BackendEntry {
            config: crate::BackendConfig {
                name: "Local".into(),
                kind: crate::BackendKind::OpenaiCompatible,
                model: "qwen3:8b".into(),
                // Nothing listens there: the run only has to start and write its feed.
                base_url: Some("http://127.0.0.1:0/v1".into()),
                ssh_host: None,
                enabled: true,
            },
            options: Default::default(),
        });
        s.settings.ai = ai.to_table().unwrap();
        s.settings.save_to(&s.config_dir().join("config.toml")).unwrap();
        let e = Engine::shared(s, specs());
        let mut ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        ws.set_person(|m| {
            m.builder = Some("Claude".into());
            m.asset_budget_usd = 42.0;
            m.generate_assets = true;
        })
        .unwrap();
        ws.manifest.stack = "Godot".into();
        ws.save().unwrap();
        let ok = || CallOptions::new(Caller::Mcp).with_consent(Consent::Granted);
        let out =
            e.call("rebuild_start", json!({"project_folder": ws.root, "request": "add a jump", "rounds": 0, "backend": "Local"}), ok()).unwrap().to_json();
        assert_eq!(out["result"]["started"].as_bool(), Some(true), "{out}");
        let saved = Workspace::open(&ws.root).unwrap().manifest;
        assert_eq!(saved.builder.as_deref(), Some("Claude"), "the saved builder is kept");
        assert_eq!(saved.asset_budget_usd, 42.0, "the saved budget is kept");
        assert!(saved.generate_assets, "the saved asset setting is kept");
        assert_eq!(saved.stack, "Godot", "the saved stack is kept");
        let feed = craft_anything_engine::feed::load(&ws);
        assert!(
            feed.iter().any(|i| matches!(i, craft_anything_engine::feed::FeedItem::User(u) if u.text() == "add a jump")),
            "the request is the run's follow-up: {feed:?}"
        );
        let _ = e.call("rebuild_stop", json!({"project": ws.root}), ok());
    }

    #[test]
    fn a_disabled_or_unknown_backend_is_refused_before_the_tool_makes_a_project() {
        let d = tempfile::tempdir().unwrap();
        let mut s = craft_anything_engine::testing::session();
        s.settings.ai = toml::from_str("[[backends]]\nname = \"Off\"\nkind = \"anthropic\"\nmodel = \"claude-test\"\nenabled = false\n").unwrap();
        let e = Engine::shared(s, specs());
        let ok = || CallOptions::new(Caller::Cli).with_consent(Consent::Granted);
        for (backend, expected) in [("Off", "turned off"), ("No Such AI", "No AI backend named")] {
            let p = json!({"request": "A page", "target": "-", "projects_dir": d.path(), "backend": backend});
            let err = e.call("rebuild_start", p, ok()).unwrap_err();
            assert!(err.message.contains(expected), "{backend}: {err:?}");
            assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 0, "{backend}: a refused start must not make a project");
        }
    }

    /// A project's builder is chosen by name, a name it can't use is refused before anything is
    /// saved, and `default` gives it back to the app's own choice.
    #[test]
    fn a_project_keeps_the_builder_it_is_given() {
        let d = tempfile::tempdir().unwrap();
        let mut s = craft_anything_engine::testing::session();
        s.settings.ai = toml::from_str(
            "[[backends]]\nname = \"Claude\"\nkind = \"anthropic\"\nmodel = \"claude-test\"\n\
             [[backends]]\nname = \"Off\"\nkind = \"anthropic\"\nmodel = \"claude-test\"\nenabled = false\n",
        )
        .unwrap();
        let e = Engine::shared(s, specs());
        let ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        let call = |p: Value| e.call("project_builder_set", p, CallOptions::new(Caller::Cli)).map(|o| o.to_json()["result"].clone());

        // Nothing chosen yet: the app's own choice builds.
        let read = call(json!({"project": ws.root})).unwrap();
        assert_eq!((read["builder"].as_str(), read["builds"].as_str()), (None, Some("Claude")));

        // A name that can't build is refused, and the project keeps what it had.
        for (name, expected) in [("Off", "turned off"), ("No Such AI", "No AI backend named")] {
            let err = call(json!({"project": ws.root, "builder": name})).unwrap_err();
            assert!(err.message.contains(expected), "{name}: {err:?}");
            assert_eq!(Workspace::open(&ws.root).unwrap().manifest.builder, None, "{name}: nothing saved");
        }

        // A builder it can use is kept in the project.
        let set = call(json!({"project": ws.root, "builder": "Claude"})).unwrap();
        assert_eq!((set["builder"].as_str(), set["builds"].as_str()), (Some("Claude"), Some("Claude")));
        assert_eq!(Workspace::open(&ws.root).unwrap().manifest.builder.as_deref(), Some("Claude"));

        // `default` gives it back to the app's choice.
        let back = call(json!({"project": ws.root, "builder": "default"})).unwrap();
        assert_eq!((back["builder"].as_str(), back["builds"].as_str()), (None, Some("Claude")));
        assert_eq!(Workspace::open(&ws.root).unwrap().manifest.builder, None);
    }

    /// Clean room is the default mode, and this tool reads and writes only the project's own
    /// choice, so it runs there: the chosen builder is part of the project, not the target.
    #[test]
    fn a_builder_is_chosen_in_clean_room() {
        let d = tempfile::tempdir().unwrap();
        let mut s = craft_anything_engine::testing::session();
        s.settings.deep_mode = false;
        assert!(s.clean_room());
        s.settings.ai = toml::from_str("[[backends]]\nname = \"Claude\"\nkind = \"anthropic\"\nmodel = \"claude-test\"\n").unwrap();
        let e = Engine::shared(s, specs());
        let ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        let set = e
            .call("project_builder_set", json!({"project": ws.root, "builder": "Claude"}), CallOptions::new(Caller::Cli))
            .map(|o| o.to_json()["result"].clone())
            .unwrap();
        assert_eq!(set["builder"], "Claude");
        assert_eq!(Workspace::open(&ws.root).unwrap().manifest.builder.as_deref(), Some("Claude"));
    }

    /// With nothing able to build, the tool still reads the choice and still accepts `default`:
    /// the answer says no AI builds instead of failing after the project changed.
    #[test]
    fn a_project_with_no_builder_says_nothing_builds_and_still_resets() {
        let d = tempfile::tempdir().unwrap();
        let e = Engine::shared(craft_anything_engine::testing::session(), specs());
        let mut ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        ws.set_person(|m| m.builder = Some("Gone".into())).unwrap();
        let call = |p: Value| e.call("project_builder_set", p, CallOptions::new(Caller::Cli)).map(|o| o.to_json()["result"].clone());
        let read = call(json!({"project": ws.root})).unwrap();
        assert_eq!((read["builder"].as_str(), read["builds"].is_null()), (Some("Gone"), true), "{read}");
        let reset = call(json!({"project": ws.root, "builder": "default"})).unwrap();
        assert_eq!((reset["builder"].is_null(), reset["builds"].is_null()), (true, true), "{reset}");
        assert_eq!(Workspace::open(&ws.root).unwrap().manifest.builder, None);
    }

    /// #1102: a tool holds the session while it runs, so starting a build on an API backend must not
    /// ask for it again; over MCP the call waited on itself until the deadlock was broken.
    #[test]
    fn rebuild_start_on_an_api_backend_does_not_wait_on_its_own_session() {
        let d = tempfile::tempdir().unwrap();
        let mut s = craft_anything_engine::testing::session();
        let mut ai = crate::AiConfig::default();
        ai.upsert(crate::BackendEntry {
            config: crate::BackendConfig {
                name: "Local".into(),
                kind: crate::BackendKind::OpenaiCompatible,
                model: "qwen3:8b".into(),
                // Nothing listens there: the run only has to start, not to get an answer.
                base_url: Some("http://127.0.0.1:9/v1".into()),
                ssh_host: None,
                enabled: true,
            },
            options: Default::default(),
        });
        s.settings.ai = ai.to_table().unwrap();
        s.settings.save_to(&s.config_dir().join("config.toml")).unwrap();
        let e = Engine::shared(s, specs());
        let ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        let ok = || CallOptions::new(Caller::Mcp).with_consent(Consent::Granted);
        let (tx, rx) = std::sync::mpsc::channel();
        let (call, root) = (e.clone(), ws.root.clone());
        std::thread::spawn(move || {
            let _ = tx.send(call.call("rebuild_start", json!({"project": root, "request": "make it blue", "rounds": 0, "backend": "Local"}), ok()));
        });
        let started = rx.recv_timeout(std::time::Duration::from_secs(60)).expect("rebuild_start returned instead of waiting on its own session");
        assert!(started.is_ok(), "{:?}", started.err());
        let _ = e.call("rebuild_stop", json!({"project": ws.root}), ok());
    }

    /// A saved builder that is off is named in the start result, so MCP callers see the
    /// replacement too.
    #[cfg(unix)]
    #[test]
    fn rebuild_start_names_a_replaced_saved_builder() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let script = d.path().join("claude");
        std::fs::write(
            &script,
            "#!/bin/sh\ncat >/dev/null\nprintf '%s\\n' '{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"Made it blue.\"}]}}'\nprintf '%s\\n' '{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"num_turns\":1}'\n",
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut s = craft_anything_engine::testing::session();
        let mut ai = crate::AiConfig::default();
        ai.upsert(crate::BackendEntry {
            config: crate::BackendConfig {
                name: "Off".into(),
                kind: crate::BackendKind::Anthropic,
                model: "claude-test".into(),
                base_url: None,
                ssh_host: None,
                enabled: false,
            },
            options: Default::default(),
        });
        ai.upsert(crate::BackendEntry {
            config: crate::BackendConfig {
                name: "Claude".into(),
                kind: crate::BackendKind::AgentCli,
                model: "claude-code".into(),
                base_url: None,
                ssh_host: None,
                enabled: true,
            },
            options: crate::BackendOptions { command: Some(script.display().to_string()), ..Default::default() },
        });
        s.settings.ai = ai.to_table().unwrap();
        // The tool runs with the session locked; save the same settings where its hub reads them.
        s.settings.save_to(&s.config_dir().join("config.toml")).unwrap();
        let e = Engine::shared(s, specs());
        let mut ws = Workspace::create(d.path(), "Notes", "A notes app", TargetRef::None).unwrap();
        ws.set_person(|m| m.builder = Some("Off".into())).unwrap();
        let ok = || CallOptions::new(Caller::Cli).with_consent(Consent::Granted);
        let out = e.call("rebuild_start", json!({"project": ws.root, "request": "make it blue", "rounds": 0}), ok()).unwrap().to_json();
        let notice = out["result"]["builder_notice"].as_str().unwrap_or_default();
        assert!(notice.contains("Off") && notice.contains("Claude"), "{out}");
        let _ = e.call("rebuild_stop", json!({"project": ws.root}), ok());
    }

    #[test]
    fn the_builder_asks_the_running_build_for_a_check_and_gets_its_report() {
        let d = tempfile::tempdir().unwrap();
        let e = Engine::shared(craft_anything_engine::testing::session(), specs());
        let ok = || CallOptions::new(Caller::Mcp);
        let ws = Workspace::create(d.path(), "Snake", "A snake game", TargetRef::None).unwrap();
        assert!(e.call("rebuild_check", json!({"project": ws.root, "wait_minutes": 0}), ok()).is_err(), "wait is checked");
        assert!(e.call("rebuild_check", json!({"project": d.path().join("nope")}), ok()).is_err(), "not a project");
        assert!(e.call("rebuild_check", json!({"wait_minutes": 1}), ok()).is_err(), "no project and no inbox to infer it from");
        // A stand-in for the run: answer the first check request with a report.
        let asks = Inbox::new(self_check::asks_dir(&ws.root));
        let answerer = std::thread::spawn(move || {
            for _ in 0..200 {
                if let Some(r) = asks.pending().first() {
                    asks.answer(&r.id, &Answer { text: Some("build: passed — Build succeeded.".into()), ..Default::default() }).unwrap();
                    return true;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            false
        });
        // The project comes from the tool server's inbox (<project>/inbox) when not given.
        e.session().ext.insert(Arc::new(InboxDir(ws.inbox_dir())));
        let v = e.call("rebuild_check", json!({"wait_minutes": 1}), ok()).unwrap().to_json();
        assert!(answerer.join().unwrap(), "the request reached checks/asks/");
        assert_eq!(v["result"]["report"], "build: passed — Build succeeded.");
        // A build running in this process can't answer a check asked from inside it.
        let _here = self_check::BuildingHere::new(&ws.root);
        let err = e.call("rebuild_check", json!({"project": ws.root}), ok()).err().unwrap();
        assert_eq!(err.code, ErrorCode::FeatureMissing);
    }

    /// A check the run took but didn't finish isn't reported as "no build is running", and the
    /// builder waits longer than the run's own limit on a check, so the run reports a stuck one.
    #[test]
    fn a_check_still_going_is_told_apart_from_no_build() {
        assert!(Duration::from_secs(d_wait_minutes() * 60) > self_check::SELF_CHECK_LIMIT);
        let root = Path::new("project");
        assert!(self_check::taken_mark(root, "1762700000000-42-1").is_some_and(|m| m.starts_with(self_check::asks_dir(root))));
        for bad in ["", "../../x", "..\\x", "c:x", "a/b"] {
            assert!(self_check::taken_mark(root, bad).is_none(), "{bad:?} would put the mark outside checks/asks/");
        }
        let going = unanswered(true, 35);
        assert_eq!(going.code, ErrorCode::TimedOut);
        assert!(going.message.contains("took the check") && !going.message.contains("No build is running"), "{}", going.message);
        let none = unanswered(false, 35);
        assert_eq!(none.code, ErrorCode::NotFound);
        assert!(none.message.contains("No build is running") && none.message.contains("35 minutes"), "{}", none.message);
    }
}
