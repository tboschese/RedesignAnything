//! How far a redesign may go. The plan's `mode` is one of three scopes:
//! `refresh` organises each screen (hierarchy, grouping, spacing, legibility) with the same blocks
//! in the same places, the same journeys and the same look; `redesign` may reorganise journeys
//! with traceability; `reimagine` may propose new flows and pages and must flag what would need
//! backend work. `mode_scope` compares the plan with its mode. It is advisory: it reports `warn`,
//! never `fail`, because a failed row would fail the kind's Ready check.
//!
//! The mode ids are stored values; what users see is `label()`, in our own words.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::redesign_checks::CheckResult;
use super::redesign_identity::{IdentityProfile, TraitDecision};
use super::redesign_plan::{DesignDecision, TransformationPlan};
use super::redesign_store::Capability;

const CHECK_ID: &str = "mode_scope";
const ADVISORY_NOTE: &str = "Advisory: mode_scope never changes concept/prototype/engineering Ready.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RedesignMode {
    /// Tidy each screen: hierarchy, grouping, spacing and legibility. Journeys, navigation and style stay.
    Refresh,
    /// Reorganise journeys and navigation, each change traced to evidence; style evolves.
    #[default]
    Redesign,
    /// Rethink flows and pages from the capabilities up; backend needs are flagged, never invented.
    Reimagine,
}

impl RedesignMode {
    pub const ALL: [Self; 3] = [Self::Refresh, Self::Redesign, Self::Reimagine];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Refresh => "refresh",
            Self::Redesign => "redesign",
            Self::Reimagine => "reimagine",
        }
    }

    /// User-visible name: a plain description, checked in docs/feature-names.md.
    pub fn label(self) -> &'static str {
        match self {
            Self::Refresh => "Organize screens",
            Self::Redesign => "Redesign journeys",
            Self::Reimagine => "New flows",
        }
    }

    pub fn parse(s: &str) -> Result<Self, ToolError> {
        Self::ALL
            .into_iter()
            .find(|m| m.as_str() == s.trim())
            .ok_or_else(|| ToolError::invalid(format!("unknown redesign mode `{s}`; use one of: {}", Self::ALL.map(Self::as_str).join(", "))))
    }

    /// What may change, for people and the builder.
    pub fn scope(self) -> &'static str {
        match self {
            Self::Refresh => {
                "Organise each screen: clearer hierarchy, grouping, spacing and legibility, with the same blocks in \
                 the same places. Keep every route, journey and the navigation; keep the look (no identity trait is \
                 replaced); no backend changes."
            }
            Self::Redesign => {
                "Reorganise journeys and navigation where evidence supports it, each change traced to a decision. \
                 The look follows redesign/identity.json: kept traits evolve within their style."
            }
            Self::Reimagine => {
                "Build new flows and pages from the capabilities up: new routes, merged or split screens, new \
                 navigation. It changes structure, not character: the look still follows redesign/identity.json. \
                 Anything that would need backend work is flagged as backend_change_required, never invented."
            }
        }
    }

    /// What the builder does with the original's capture; replaces the website kind's "match it exactly".
    pub fn capture_guidance(self) -> &'static str {
        match self {
            Self::Refresh => {
                "Keep the original's layout: the same blocks in the same places at every width, with colours, fonts, text \
                 sizes, padding, gaps, corner radii and content widths taken from capture.json. Change only what makes \
                 each screen easier to read and use (hierarchy, grouping, spacing, legibility, hit areas, focus), and \
                 never let the page scroll sideways on a phone."
            }
            Self::Redesign | Self::Reimagine => {
                "The capture is evidence of what the original does and how it looks, not a layout to copy: do not clone \
                 its pages. Take the look (colours, fonts, shapes) from redesign/identity.json, with exact values from \
                 capture.json for the traits that are kept, and never let the page scroll sideways on a phone."
            }
        }
    }

    /// The mode of a project's plan; the default when there is no plan yet or its mode can't be read.
    pub fn of(ws: &Workspace) -> Self {
        super::redesign_plan::load_plan(ws).ok().and_then(|p| Self::parse(&p.mode).ok()).unwrap_or_default()
    }

    /// One line for the builder brief.
    pub fn builder_guidance(self) -> String {
        format!("Redesign mode `{}` ({}): {}", self.as_str(), self.label(), self.scope())
    }
}

/// The three modes, for `redesign_modes_list`.
pub fn list() -> Value {
    json!({
        "modes": RedesignMode::ALL.iter().map(|m| json!({"id": m.as_str(), "label": m.label(), "scope": m.scope()})).collect::<Vec<_>>(),
        "default": RedesignMode::default().as_str(),
        "check": "mode_scope (in redesign_checks_run) compares the plan with its mode. It is advisory: out of scope is a warning.",
    })
}

fn same_route(a: &str, b: &str) -> bool {
    let norm = |s: &str| {
        let t = s.trim().trim_end_matches('/');
        if t.is_empty() { "/".to_string() } else { t.to_ascii_lowercase() }
    };
    norm(a) == norm(b)
}

/// A mapping that sends a capability to a route it didn't have in the original.
fn moved_routes(caps: &[Capability], plan: &TransformationPlan) -> Vec<String> {
    plan.mappings
        .iter()
        .filter_map(|m| {
            let target = m.target_route.as_deref().filter(|r| !r.trim().is_empty())?;
            let cap = caps.iter().find(|c| c.id == m.capability_id)?;
            (!cap.original_routes.is_empty() && !cap.original_routes.iter().any(|r| same_route(r, target)))
                .then(|| format!("{}: {} → {target}", cap.id, cap.original_routes.join(", ")))
        })
        .collect()
}

pub fn evaluate_scope(caps: &[Capability], plan: Option<&TransformationPlan>, decisions: &[DesignDecision], identity: Option<&IdentityProfile>) -> CheckResult {
    let Some(plan) = plan else {
        return CheckResult {
            id: CHECK_ID.into(),
            status: "not_evaluated".into(),
            summary: "no transformation plan, so there is no mode to check".into(),
            details: json!({"advisory": true, "note": ADVISORY_NOTE}),
        };
    };
    let mode = match RedesignMode::parse(&plan.mode) {
        Ok(m) => m,
        Err(e) => {
            return CheckResult { id: CHECK_ID.into(), status: "warn".into(), summary: e.message, details: json!({"advisory": true, "note": ADVISORY_NOTE}) };
        }
    };
    let moved = moved_routes(caps, plan);
    let backend: Vec<String> = plan.mappings.iter().filter(|m| m.feasibility == "backend_change_required").map(|m| m.capability_id.clone()).collect();
    let replaced: Vec<String> =
        identity.map(|p| p.traits.iter().filter(|t| t.effective_decision() == TraitDecision::Replace).map(|t| t.id.clone()).collect()).unwrap_or_default();
    let divergent: Vec<String> = decisions.iter().filter(|d| d.chosen_option_id != "keep_as_is").map(|d| d.id.clone()).collect();

    let (status, summary) = match mode {
        RedesignMode::Refresh => {
            let mut over = Vec::new();
            if !moved.is_empty() {
                over.push(format!("{} capability(ies) moved to a new route", moved.len()));
            }
            if !backend.is_empty() {
                over.push(format!("{} mapping(s) need backend changes", backend.len()));
            }
            if !replaced.is_empty() {
                over.push(format!("{} identity trait(s) replaced", replaced.len()));
            }
            if over.is_empty() {
                ("pass", "stays within the original's routes, backend and identity".to_string())
            } else {
                ("warn", format!("goes beyond organizing the screens: {}", over.join("; ")))
            }
        }
        RedesignMode::Redesign => ("pass", format!("{} route move(s), {} divergent decision(s); traceability rules apply", moved.len(), divergent.len())),
        RedesignMode::Reimagine if moved.is_empty() && divergent.is_empty() && replaced.is_empty() => {
            ("warn", "proposes nothing new: no route moves, divergent decisions or replaced traits".to_string())
        }
        RedesignMode::Reimagine => {
            ("pass", format!("{} route move(s), {} divergent decision(s), {} backend need(s) flagged", moved.len(), divergent.len(), backend.len()))
        }
    };
    CheckResult {
        id: CHECK_ID.into(),
        status: status.into(),
        summary,
        details: json!({
            "advisory": true,
            "note": ADVISORY_NOTE,
            "mode": mode.as_str(),
            "label": mode.label(),
            "scope": mode.scope(),
            "moved_routes": moved,
            "backend_change_required": backend,
            "replaced_traits": replaced,
            "divergent_decisions": divergent,
        }),
    }
}

/// `mode_scope` for a project. Unreadable inputs are reported, not passed.
pub fn scope_check(ws: &Workspace) -> CheckResult {
    let not_evaluated = |what: &str, e: ToolError| CheckResult {
        id: CHECK_ID.into(),
        status: "not_evaluated".into(),
        summary: format!("{what} could not be read: {}", e.message),
        details: json!({"advisory": true, "note": ADVISORY_NOTE}),
    };
    let caps = match super::redesign_store::list_capabilities(ws) {
        Ok(c) => c,
        Err(e) => return not_evaluated("capabilities", e),
    };
    let decisions = match super::redesign_plan::list_decisions(ws) {
        Ok(d) => d,
        Err(e) => return not_evaluated("decisions", e),
    };
    let identity = match super::redesign_identity::load(ws) {
        Ok(p) => p,
        Err(e) => return not_evaluated("identity profile", e),
    };
    let plan = match super::redesign_plan::load_plan(ws) {
        Ok(p) => Some(p),
        Err(e) if e.code == craft_anything_evidence::ErrorCode::NotFound => None,
        Err(e) => return not_evaluated("transformation plan", e),
    };
    evaluate_scope(&caps, plan.as_ref(), &decisions, identity.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rebuild::redesign_identity::{IdentityTrait, TraitCategory, TraitIntent};
    use crate::rebuild::redesign_plan::PlanMapping;

    fn cap(id: &str, route: &str) -> Capability {
        Capability {
            id: id.into(),
            title: id.into(),
            task: "task".into(),
            actor_roles: vec![],
            original_routes: vec![route.into()],
            inputs: vec![],
            actions: vec![],
            outcomes: vec![],
            criticality: "critical".into(),
            knowledge_status: "observed".into(),
            source_refs: vec![],
            unknowns: vec![],
        }
    }

    fn plan(mode: &str, target: &str, feasibility: &str) -> TransformationPlan {
        TransformationPlan {
            schema_version: "1.0.0".into(),
            project_id: "p".into(),
            revision: 1,
            generated_at: "0Z".into(),
            mode: mode.into(),
            snapshot_id: "s".into(),
            mappings: vec![PlanMapping {
                capability_id: "cap_search".into(),
                target_route: Some(target.into()),
                target_component: None,
                target_action: None,
                decision_id: "dec_search".into(),
                scenario_ids: vec![],
                exception_approved: false,
                feasibility: feasibility.into(),
            }],
            unknowns: vec![],
            decisions: vec![],
            notes: vec![],
        }
    }

    fn replaced_trait() -> IdentityProfile {
        IdentityProfile {
            traits: vec![IdentityTrait {
                id: "trait_chrome".into(),
                label: "Black chrome".into(),
                category: TraitCategory::Chrome,
                observation: "black bar".into(),
                intent: TraitIntent::Intentional,
                decision: TraitDecision::Replace,
                rationale: Some("new brand".into()),
                evolve_within: None,
                evidence_ids: vec![],
                checks: Default::default(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn a_tidy_up_that_moves_a_journey_or_replaces_a_trait_is_flagged() {
        let caps = [cap("cap_search", "/search")];
        let same = evaluate_scope(&caps, Some(&plan("refresh", "/search/", "ui_only")), &[], None);
        assert_eq!(same.status, "pass", "{same:?}");
        let moved = evaluate_scope(&caps, Some(&plan("refresh", "/find", "ui_only")), &[], None);
        assert_eq!(moved.status, "warn", "{moved:?}");
        let replaced = evaluate_scope(&caps, Some(&plan("refresh", "/search", "ui_only")), &[], Some(&replaced_trait()));
        assert_eq!(replaced.status, "warn", "{replaced:?}");
        // The same plan is within scope once the mode allows it.
        let redesign = evaluate_scope(&caps, Some(&plan("redesign", "/find", "ui_only")), &[], Some(&replaced_trait()));
        assert_eq!(redesign.status, "pass", "{redesign:?}");
    }

    #[test]
    fn an_out_of_scope_plan_warns_without_failing_the_ready_check() {
        let caps = [cap("cap_search", "/search")];
        let out_of_scope = evaluate_scope(&caps, Some(&plan("refresh", "/find", "backend_change_required")), &[], Some(&replaced_trait()));
        let report = crate::rebuild::redesign_checks::ReadyReport {
            concept_ready: true,
            prototype_ready: true,
            engineering_ready: true,
            checks: vec![out_of_scope],
            notes: vec![],
        };
        let ready = crate::rebuild::kinds::redesign::ready_result_from_report(1, &report);
        let row = ready.claims.iter().find(|c| c.title.contains(CHECK_ID)).expect("mode_scope is reported");
        assert!(row.passed, "advisory: {row:?}");
        assert!(row.title.contains("warn"), "and still visible as a warning: {row:?}");
    }

    #[test]
    fn a_rethink_that_changes_nothing_is_flagged() {
        let caps = [cap("cap_search", "/search")];
        let same = evaluate_scope(&caps, Some(&plan("reimagine", "/search", "ui_only")), &[], None);
        assert_eq!(same.status, "warn", "{same:?}");
        let moved = evaluate_scope(&caps, Some(&plan("reimagine", "/find", "backend_change_required")), &[], None);
        assert_eq!(moved.status, "pass", "{moved:?}");
    }

    #[test]
    fn an_unknown_mode_is_refused() {
        assert!(RedesignMode::parse("overhaul").is_err());
        let r = evaluate_scope(&[], Some(&plan("overhaul", "/", "ui_only")), &[], None);
        assert_eq!(r.status, "warn", "{r:?}");
    }
}
