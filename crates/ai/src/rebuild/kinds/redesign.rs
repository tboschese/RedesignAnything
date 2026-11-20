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
    "- Write the app under app/; do not modify the original target. Use synthetic/mock data when the backend is unknown."
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
        if !cx.ws.manifest.own_project() {
            out.push(website::HOW_TO_RUN.into());
        }
        out
    }

    fn checks(&self, _ws: &Workspace) -> Vec<Box<dyn Check>> {
        // No WebVisualCheck / pixel-similarity gate. Build + native Ready (fail-closed).
        vec![Box::new(BuildCheck), Box::new(RedesignReadyCheck)]
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
    let blocker = report
        .checks
        .iter()
        .any(|c| matches!(c.id.as_str(), "security_secrets" | "design_system" | "build_scaffold") && matches!(c.status.as_str(), "fail" | "blocked"));
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
        // Only ask the builder to fix code-addressable blockers (secrets, DS, scaffold).
        // Missing oracles / prototype maturity are not refined by inventing evidence.
        result.claims.iter().any(|c| !c.passed && (c.title.contains("security") || c.title.contains("design_system") || c.title.contains("build_scaffold")))
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
    fn redesign_checks_include_ready_but_not_visual_likeness() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::Website { url: "https://example.test".into() }).unwrap();
        let ids: Vec<_> = RedesignKind.checks(&ws).iter().map(|c| c.id()).collect();
        assert_eq!(ids, [CheckResult::BUILD, RedesignReadyCheck::ID]);
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
