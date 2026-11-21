//! Redesign Anything kind: evidence-based redesign of an existing web app.
//! Selected only by an explicit project `kind` / CLI/MCP `kind=redesign` — never by ambiguous
//! wording that would steal ordinary website or app projects.

use super::*;
use craft_anything_engine::workspace::Claim;

pub struct RedesignKind;

impl RedesignKind {
    pub const ID: &'static str = "redesign";
    /// Default stack for redesign projects (React + TypeScript via Vite).
    pub const DEFAULT_STACK: &'static str = "React + TypeScript (Vite)";
}

/// Brief contract shared with builders and host agents.
pub const REDESIGN_BRIEF: &str = concat!(
    "This is a Redesign Anything project (kind=redesign), not a visual clone.\n",
    "- Preserve capabilities that Study observed and recorded; do not invent backend contracts.\n",
    "- Prefer meaningful UX structure over pixel likeness to the original; visual similarity is not a pass/fail gate.\n",
    "- Offer keep_as_is as a valid option when change isn't justified.\n",
    "- Mark unknowns explicitly; never promote inferred rules to validated without an independent oracle.\n",
    "- Default stack is React + TypeScript (Vite) unless the person named another supported stack.\n",
    "- Write the app under app/; do not modify the original target. Use synthetic/mock data when the backend is unknown.\n",
    "- A scenario counts as working when its journey runs in the built prototype (redesign_journeys_run), not when its words appear in the code. Build the app before the checks."
);

impl TargetKind for RedesignKind {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn words(&self) -> KindWords {
        KindWords {
            button: "Open the redesign",
            icon: "layout-template",
            ready: "Your redesign is ready",
            change_hint: "Ask for a change, e.g. \"simplify the form\" or \"keep the submit flow as is\"",
        }
    }

    /// Explicit markers only. Ordinary phrases like "redesign this page" must not win over website.
    fn detect(&self, cx: &DetectCx) -> Option<u32> {
        if cx.says(&[" kind=redesign ", " [kind=redesign] ", " mode=redesign "]) { Some(200) } else { None }
    }

    fn default_stack(&self, _cx: &DetectCx) -> &'static str {
        Self::DEFAULT_STACK
    }

    fn goal(&self) -> &'static str {
        "a redesign that preserves known capabilities with clearer UX, not a pixel clone"
    }

    fn self_screenshots(&self) -> bool {
        false
    }

    /// A redesign never rebuilds the original pixel for pixel; how close it stays depends on the mode.
    fn capture_guidance(&self, cx: &BriefCx) -> Option<String> {
        Some(crate::rebuild::redesign_mode::RedesignMode::of(cx.ws).capture_guidance().into())
    }

    fn brief_sections(&self, cx: &BriefCx) -> Vec<String> {
        let mut out = vec![REDESIGN_BRIEF.into()];
        // Compact redesign refs — pass by path, do not dump full stores into the prompt.
        out.push(
            "Redesign workspace refs (read these files; do not invent contracts): redesign/store.json, \
             redesign/capabilities/, redesign/decisions/, redesign/design_system.json, \
             redesign/transformation_plan.json, redesign/scaffold_refs.json. \
             Prefer redesign_scaffold_app under host_agent instead of nested rebuild_start."
                .into(),
        );
        match crate::rebuild::redesign_identity::load(cx.ws) {
            Ok(Some(profile)) if !profile.traits.is_empty() => out.push(format!(
                "Visual identity (redesign/identity.json; keep traits are checked by identity_drift):\n- {}",
                crate::rebuild::redesign_identity::identity_constraints(&profile).join("\n- ")
            )),
            Ok(_) => {}
            Err(e) => out.push(format!("redesign/identity.json could not be read ({}); ask before changing the original's look.", e.message)),
        }
        if let Ok(plan) = crate::rebuild::redesign_plan::load_plan(cx.ws)
            && let Ok(mode) = crate::rebuild::redesign_mode::RedesignMode::parse(&plan.mode)
        {
            out.push(format!("{} mode_scope checks the plan against this.", mode.builder_guidance()));
        }
        // Reuse website capture notes when Study photographed the target.
        let routes = website::routes(cx.ws);
        if !routes.is_empty() {
            let list = routes.iter().map(|r| format!("{} (capture/routes/{}/)", r.path, r.slug)).collect::<Vec<_>>().join(", ");
            out.push(format!(
                "Study captured related pages: {list}. Preserve navigation and task flows that were observed; list pages not captured as unknowns."
            ));
        }
        let states = website::pictured_states(cx.ws);
        if !states.is_empty() {
            out.push(format!(
                "Observed UI states (menus, dialogs, tabs): {}. Preserve those behaviours when redesigning; do not invent states that were not observed.",
                states.join(", ")
            ));
        }
        out.extend(plan_changes(cx.ws));
        let unmapped = crate::rebuild::redesign_likeness::pages_without_address(cx.ws);
        if !unmapped.is_empty() {
            let routes = website::routes(cx.ws);
            let list = unmapped
                .iter()
                .map(|slug| format!("- {slug}: {}", routes.iter().find(|r| &r.slug == slug).map_or("", |r| r.path.as_str())))
                .collect::<Vec<_>>()
                .join("\n");
            out.push(format!("{CAPTURE_BRIEF}\nThe original pages that need an address in the prototype:\n{list}"));
        }
        let owed = crate::rebuild::redesign_journeys::owed(cx.ws);
        if !owed.is_empty() {
            let list = owed.iter().map(|(id, then)| format!("- {id}: {then}")).collect::<Vec<_>>().join("\n");
            out.push(format!("{JOURNEYS_BRIEF}\nThe scenarios that need one:\n{list}"));
        }
        if !cx.ws.manifest.own_project() {
            out.push(website::HOW_TO_RUN.into());
        }
        out
    }

    /// The redesign tools made the study (capture, capabilities, oracle, plan); studying the site
    /// again would replace the pages the plan is about.
    fn keeps_its_study(&self, ws: &Workspace) -> bool {
        ws.capture_dir().join("capture.json").is_file() && crate::rebuild::redesign_store::list_capabilities(ws).is_ok_and(|caps| !caps.is_empty())
    }

    fn checks(&self, _ws: &Workspace) -> Vec<Box<dyn Check>> {
        // No WebVisualCheck / pixel-similarity gate. Build, the scenarios' journeys run in the
        // built site, then native Ready (fail-closed), which reads what the journeys found.
        vec![Box::new(BuildCheck), Box::new(JourneysCheck), Box::new(PrototypeCaptureCheck), Box::new(RedesignReadyCheck)]
    }
}

/// How the builder shows each scenario working, for [`JourneysCheck`].
pub const JOURNEYS_BRIEF: &str = concat!(
    "Show each scenario working in the built prototype: write redesign/quality/journeys.json as ",
    "{\"journeys\": [{\"scenario_id\": \"<id>\", \"steps\": [...]}]}, one journey per scenario below. ",
    "Steps run in order in a real browser from the start page: {\"click\": \"<visible or accessible name>\"}, ",
    "{\"type\": \"<text>\", \"into\": \"<field label or placeholder>\"}, {\"press\": \"Enter\"}, ",
    "{\"open\": \"/#/<route>\"} and {\"expect\": \"<text that must show>\"}; end each journey with an expect. ",
    "Each check round runs them and tells you the step that failed. A scenario passes only when its journey does, ",
    "not when its words appear in the code, so write the UI that makes the steps work rather than steps that avoid it."
);

/// What the plan changes, said plainly to the builder: each approved decision that changes the
/// original (its problem, the chosen option and what it changes), and each scenario an approved
/// decision retired, with what replaces it. The files alone hold generic option text, so without
/// this a builder rebuilds what the original did.
fn plan_changes(ws: &Workspace) -> Vec<String> {
    let mut out = Vec::new();
    let decisions = crate::rebuild::redesign_plan::list_decisions(ws).unwrap_or_default();
    let changes: Vec<String> = decisions
        .iter()
        .filter(|d| d.chosen_option_id != "keep_as_is" && d.status == "approved")
        .map(|d| {
            let option = d.options.iter().find(|o| o.id == d.chosen_option_id).map_or(d.chosen_option_id.as_str(), |o| o.description.as_str());
            format!("- {} ({}): {} Chosen: {} Rationale: {}", d.id, d.capability_ids.join(", "), d.problem.trim(), option.trim(), d.rationale.trim())
        })
        .collect();
    if !changes.is_empty() {
        out.push(format!("Approved changes to the original; build these, not what the original did:\n{}", changes.join("\n")));
    }
    let (retired, _) = crate::rebuild::redesign_retired::in_force(ws);
    if !retired.is_empty() {
        let lines: Vec<String> = retired.values().map(|r| format!("- {}: {}", r.scenario_id, r.note())).collect();
        out.push(format!(
            "Scenarios retired by an approved decision: the prototype must not do what they ask for; build what replaces them, and write no journey for them:\n{}",
            lines.join("\n")
        ));
    }
    out
}

/// How the builder tells the check round which prototype page stands for each original page.
pub const CAPTURE_BRIEF: &str = concat!(
    "Each check round also photographs the built prototype and compares each page with the original's page. ",
    "Say which prototype address stands for each original page other than the start page in ",
    "redesign/prototype_capture/pages.json as {\"pages\": [{\"page\": \"<page>\", \"path\": \"/#/<route>\"}]}."
);

/// Photographs the built prototype the way `redesign_prototype_capture` does (the start page and
/// each original page at its kept address, at the original's widths), measures how alike each page
/// is, and turns what the captures show into claims the builder can act on: every page fits its
/// window, and the basics (names, alt text, language, the original's landmarks) are there. Likeness
/// is reported, never a claim. An original page without an address is an unknown, said in the note.
pub struct PrototypeCaptureCheck;

impl PrototypeCaptureCheck {
    pub const ID: &'static str = "redesign-capture";
}

impl Check for PrototypeCaptureCheck {
    fn id(&self) -> &'static str {
        Self::ID
    }

    /// Nothing to compare with before the original was captured.
    fn applies(&self, cx: &CheckCx) -> bool {
        cx.ws.capture_dir().join("capture.json").is_file()
    }

    fn run(&self, cx: &mut CheckCx) -> Result<CheckResult, ToolError> {
        use crate::rebuild::redesign_likeness as likeness;
        let (ws, round) = (cx.ws, cx.round);
        let Some(site) = crate::rebuild::site_dir(&ws.app_dir()) else {
            return Ok(CheckResult::new(self.id(), round, false, "No built site to photograph: build the app first (npm run build in app/)."));
        };
        let targets = match likeness::capture_targets(ws, &likeness::kept_pages(ws)) {
            Ok(t) => t,
            Err(e) => {
                return Ok(CheckResult::new(
                    self.id(),
                    round,
                    false,
                    format!("redesign/prototype_capture/{} can't be used: {}. {CAPTURE_BRIEF}", likeness::PAGES_FILE, e.message),
                ));
            }
        };
        let base = cx.serve_preview(&site)?;
        let widths = likeness::original_widths(ws);
        let mut not_captured = Vec::new();
        for (page, path) in &targets {
            let out = likeness::fresh_page_dir(ws, page)?;
            // A page opened with only its `#…` changed doesn't load again, so each page gets its own browser.
            let _ = cx.call("web_site_close", json!({}));
            let done = cx
                .call("web_site_open", json!({"url": format!("{}{}", base.trim_end_matches('/'), path), "settle_ms": 800, "signed_in": false}))
                .and_then(|_| cx.call("web_site_capture", json!({"output_directory": out.display().to_string(), "include_html": false, "widths": widths})));
            if let Err(e) = done {
                not_captured.push(format!("{page} ({path}): {}", e.message));
            }
        }
        let _ = cx.call("web_site_close", json!({}));
        let report = likeness::measure(ws)?;
        let fits = likeness::fits_window_check(ws);
        let basics = likeness::basics_check(ws);
        let copied = likeness::copied_wording_check(ws);
        let mut claims = vec![
            Claim::new("every captured page fits its window", fits.status != "warn", fits.summary.clone()),
            Claim::new("every captured page has the basics (names, alt text, language, landmarks)", basics.status != "warn", basics.summary.clone()),
            Claim::new("no captured page repeats the original's sentences", copied.status != "warn", copied.summary.clone()),
        ];
        claims.extend(not_captured.iter().map(|why| Claim::new(format!("the prototype page {why} could be photographed"), false, why.clone())));
        let scores: Vec<String> =
            report["pages"].as_array().into_iter().flatten().filter_map(|p| Some(format!("{} {:.0}", p["page"].as_str()?, p["overall"].as_f64()?))).collect();
        let mut note = format!("Photographed {} page(s) of the prototype at {:?} px.", targets.len() - not_captured.len(), widths);
        if !scores.is_empty() {
            note.push_str(&format!(" Likeness to the original (0..=100, reported, not judged): {}.", scores.join(", ")));
        }
        for c in claims.iter().filter(|c| !c.passed) {
            note.push_str(&format!(" {}: {}.", c.title, c.detail));
        }
        let mut out = CheckResult::new(self.id(), round, not_captured.len() < targets.len(), note);
        out.claims = claims;
        out.unknowns = likeness::pages_without_address(ws)
            .into_iter()
            .map(|page| format!("{page}: no prototype address in redesign/prototype_capture/{}", likeness::PAGES_FILE))
            .collect();
        out.artifacts = vec![std::path::PathBuf::from("redesign/prototype_capture/likeness.json")];
        Ok(out)
    }
}

/// Runs the kept journeys in the built site, one per owed scenario, and writes
/// redesign/quality/journeys-report.json for [`RedesignReadyCheck`] to read. A scenario without a
/// journey, or whose journey fails, is something the builder can fix, so it is refined.
pub struct JourneysCheck;

impl JourneysCheck {
    pub const ID: &'static str = "redesign-journeys";
}

impl Check for JourneysCheck {
    fn id(&self) -> &'static str {
        Self::ID
    }

    /// Nothing to run without a scenario that owes a journey.
    fn applies(&self, cx: &CheckCx) -> bool {
        !crate::rebuild::redesign_journeys::owed(cx.ws).is_empty()
    }

    fn run(&self, cx: &mut CheckCx) -> Result<CheckResult, ToolError> {
        use crate::rebuild::redesign_journeys as journeys;
        let (ws, round) = (cx.ws, cx.round);
        let owed = journeys::owed(ws);
        let kept = match journeys::load(ws).and_then(|kept| journeys::check(ws, &kept).map(|()| kept)) {
            Ok(kept) => kept,
            Err(e) => {
                return Ok(CheckResult::new(
                    self.id(),
                    round,
                    false,
                    format!("redesign/{} can't be used: {}. {JOURNEYS_BRIEF}", journeys::JOURNEYS_FILE, e.message),
                ));
            }
        };
        let missing: Vec<&str> = owed.iter().map(|(id, _)| id.as_str()).filter(|id| !kept.iter().any(|j| j.scenario_id == *id)).collect();
        let Some(site) = crate::rebuild::site_dir(&ws.app_dir()) else {
            return Ok(CheckResult::new(self.id(), round, false, "No built site to run the journeys in: build the app first (npm run build in app/)."));
        };
        let base = cx.serve_preview(&site)?;
        let built = journeys::built_fingerprint(ws).ok_or_else(|| ToolError::internal("the built prototype could not be read"))?;
        let _ = cx.call("web_site_close", json!({}));
        cx.call("web_site_open", json!({"url": base, "settle_ms": 800, "signed_in": false}))?;
        let mut results = Vec::new();
        for j in &kept {
            let ran = journeys::fresh_picture_dir(ws, &j.scenario_id)
                .and_then(|out| cx.call("web_site_journey_run", json!({"steps": j.steps, "output_directory": out.display().to_string()})));
            results.push(journeys::result_of(&j.scenario_id, ran.map(|o| o.to_json()["result"].clone())));
        }
        let _ = cx.call("web_site_close", json!({}));
        let report_path = journeys::write_report(ws, &journeys::Report { built, width: 1440, journeys: results.clone() })?;

        let mut claims: Vec<Claim> = results.iter().map(|r| Claim::new(format!("the journey for {} passes", r.scenario_id), r.passed, r.note())).collect();
        claims.extend(missing.iter().map(|id| Claim::new(format!("the journey for {id} passes"), false, "no journey was written for it")));
        let failed: Vec<String> = results.iter().filter(|r| !r.passed).map(|r| format!("{}: {}", r.scenario_id, r.note())).collect();
        let passed = failed.is_empty() && missing.is_empty();
        let note = if passed {
            format!("All {} journey(s) passed in the built prototype.", results.len())
        } else {
            let mut parts = Vec::new();
            if !failed.is_empty() {
                parts.push(format!("Journeys that failed — {}.", failed.join("; ")));
            }
            if !missing.is_empty() {
                parts.push(format!("Scenarios without a journey: {}. {JOURNEYS_BRIEF}", missing.join(", ")));
            }
            parts.join(" ")
        };
        let mut out = CheckResult::new(self.id(), round, passed, note);
        out.claims = claims;
        out.artifacts = report_path.strip_prefix(&ws.root).ok().map(Path::to_path_buf).into_iter().collect();
        Ok(out)
    }
}

/// Native redesign Ready gate registered on the kind (not WASM PluginJudge).
/// Fail-closed for secrets/DS/build blockers; prototype/engineering levels are claims, not a
/// pixel score. Missing oracles stay unknowns — never coerced to pass.
pub struct RedesignReadyCheck;

impl RedesignReadyCheck {
    pub const ID: &'static str = "redesign-ready";
}

/// Map a native Ready report into a kind [`CheckResult`] (no pixel similarity).
pub fn ready_result_from_report(round: u32, report: &crate::rebuild::redesign_checks::ReadyReport) -> CheckResult {
    let mut claims = Vec::new();
    let mut unknowns = Vec::new();

    for c in &report.checks {
        match c.status.as_str() {
            "pass" => claims.push(Claim::new(&c.id, true, "")),
            "warn" => claims.push(Claim::new(format!("{} (warn)", c.id), true, &c.summary)),
            "not_evaluated" => unknowns.push(format!("{}: {}", c.id, c.summary)),
            "blocked" | "fail" => claims.push(Claim::new(&c.id, false, &c.summary)),
            other => claims.push(Claim::new(&c.id, false, format!("unexpected status {other}: {}", c.summary))),
        }
    }
    claims.push(Claim::new("concept_ready", report.concept_ready, "security/preview/build gate"));
    claims.push(Claim::new("prototype_ready", report.prototype_ready, "needs oracles + traceability; not a visual likeness score"));
    claims.push(Claim::new("engineering_ready", report.engineering_ready, "prototype + approved backend deps"));

    // Fail closed on security / strict DS / missing scaffold. Prototype maturity is reported
    // as claims so early scaffolds are not stuck in endless refine for missing oracles.
    let blocker = report.checks.iter().any(|c| {
        matches!(c.id.as_str(), "security_secrets" | "design_system" | "build_scaffold" | "identity_essence" | "identity_drift")
            && matches!(c.status.as_str(), "fail" | "blocked")
    });
    let note = format!(
        "concept={} prototype={} engineering={}; native Ready (no PluginJudge, no pixel gate)",
        report.concept_ready, report.prototype_ready, report.engineering_ready
    );
    let mut out = CheckResult::new(RedesignReadyCheck::ID, round, !blocker, note);
    out.claims = claims;
    out.unknowns = unknowns;
    out.similarity = None;
    out.artifacts = vec![std::path::PathBuf::from("redesign/check_results.json")];
    out
}

impl Check for RedesignReadyCheck {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn can_refine(&self, result: &CheckResult) -> bool {
        // Only ask the builder to fix code-addressable blockers (secrets, DS, scaffold, identity drift).
        // Missing oracles / prototype maturity are not refined by inventing evidence.
        // Identity drift is code too: the app's styles left a kept trait.
        result.claims.iter().any(|c| {
            !c.passed
                && (c.title.contains("security")
                    || c.title.contains("design_system")
                    || c.title.contains("build_scaffold")
                    || c.title.contains("identity_drift"))
        })
    }

    fn run(&self, cx: &mut CheckCx) -> Result<CheckResult, ToolError> {
        let report = crate::rebuild::redesign_checks::run_native_checks(cx.ws)?;
        Ok(ready_result_from_report(cx.round, &report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::TargetRef;

    #[test]
    fn redesign_is_only_chosen_with_an_explicit_marker() {
        let kinds = KindRegistry::built_in();
        let web = TargetRef::Website { url: "https://example.test".into() };
        assert_eq!(kinds.detect("Redesign this website please", &web, "").unwrap().id(), "website");
        assert_eq!(kinds.detect("Clone this website kind=redesign", &web, "").unwrap().id(), "redesign");
        assert_eq!(kinds.get("redesign").unwrap().default_stack(&DetectCx::new("", &web, "")), RedesignKind::DEFAULT_STACK);
    }

    #[test]
    fn a_redesign_keeps_its_study_once_it_has_a_capture_and_capabilities() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::Website { url: "https://example.test".into() }).unwrap();
        assert!(!RedesignKind.keeps_its_study(&ws), "nothing studied yet: the builder studies");
        std::fs::create_dir_all(ws.capture_dir()).unwrap();
        std::fs::write(ws.capture_dir().join("capture.json"), "{}").unwrap();
        assert!(!RedesignKind.keeps_its_study(&ws), "a capture alone is not a redesign study");
        let cap = crate::rebuild::redesign_store::Capability {
            id: "cap_search".into(),
            title: "Search".into(),
            task: "find".into(),
            actor_roles: vec![],
            original_routes: vec!["/".into()],
            inputs: vec![],
            actions: vec![],
            outcomes: vec![],
            criticality: "critical".into(),
            knowledge_status: "observed".into(),
            source_refs: vec![],
            unknowns: vec![],
        };
        crate::rebuild::redesign_store::put_capability(&ws, cap).unwrap();
        assert!(RedesignKind.keeps_its_study(&ws));
        assert!(!WebsiteKind.keeps_its_study(&ws), "other kinds study as before");
    }

    #[test]
    fn redesign_checks_include_ready_but_not_visual_likeness() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::Website { url: "https://example.test".into() }).unwrap();
        let ids: Vec<_> = RedesignKind.checks(&ws).iter().map(|c| c.id()).collect();
        assert_eq!(ids, [CheckResult::BUILD, JourneysCheck::ID, PrototypeCaptureCheck::ID, RedesignReadyCheck::ID]);
        assert!(!ids.contains(&CheckResult::WEB_VISUAL));
    }

    #[test]
    fn redesign_ready_check_fails_closed_on_secrets_without_pixel_score() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/design-system").canonicalize().unwrap();
        crate::rebuild::redesign_ds::ingest_package(&ws, &fixture, "strict", false).unwrap();
        crate::rebuild::redesign_build::scaffold_app(&ws, None).unwrap();
        std::fs::write(ws.app_dir().join("src/App.tsx"), "const aws_secret_access_key = \"x\";\n").unwrap();

        let report = crate::rebuild::redesign_checks::run_native_checks(&ws).unwrap();
        let result = ready_result_from_report(1, &report);
        assert!(!result.passed, "{result:?}");
        assert!(result.similarity.is_none());
        assert!(result.claims.iter().any(|c| c.title == "security_secrets" && !c.passed));
        assert!(RedesignReadyCheck.can_refine(&result));
        assert!(!result.good(90.0), "failed claims keep it from being good enough");
    }

    #[test]
    fn early_scaffold_without_oracles_is_not_prototype_but_may_pass_ready_gate() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/design-system").canonicalize().unwrap();
        crate::rebuild::redesign_ds::ingest_package(&ws, &fixture, "strict", false).unwrap();
        crate::rebuild::redesign_build::scaffold_app(&ws, None).unwrap();

        let report = crate::rebuild::redesign_checks::run_native_checks(&ws).unwrap();
        let result = ready_result_from_report(1, &report);
        assert!(result.passed, "no security/DS/build blocker: {result:?}");
        assert!(result.similarity.is_none());
        assert!(result.claims.iter().any(|c| c.title == "prototype_ready" && !c.passed));
        assert!(!RedesignReadyCheck.can_refine(&result), "missing oracles are not builder-refined");
    }
}
