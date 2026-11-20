//! Design Decision + Feasibility + Traceability (PR-06).
//! diagnose → retrieve → alternatives → choose → plan.
//! Never invents endpoints/atomicity; critical caps without destination block readiness.

use craft_anything_engine::project::write_atomic;
use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::{ErrorCode, ToolError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

use super::redesign_knowledge::{self, CriticReport};
use super::redesign_store::{self, Capability, redesign_dir};

pub const PLAN_FILE: &str = "transformation_plan.json";
pub const DECISIONS_DIR: &str = "decisions";
pub const SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionOption {
    pub id: String,
    pub description: String,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignDecision {
    pub id: String,
    pub capability_ids: Vec<String>,
    pub problem: String,
    pub options: Vec<DecisionOption>,
    pub chosen_option_id: String,
    pub rationale: String,
    pub ux_rule_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub feasibility: String,
    pub dependencies: Vec<String>,
    pub status: String,
    #[serde(default)]
    pub approval_evidence_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PlanMapping {
    pub capability_id: String,
    #[serde(default)]
    pub target_route: Option<String>,
    #[serde(default)]
    pub target_component: Option<String>,
    #[serde(default)]
    pub target_action: Option<String>,
    pub decision_id: String,
    #[serde(default)]
    pub scenario_ids: Vec<String>,
    pub exception_approved: bool,
    pub feasibility: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformationPlan {
    pub schema_version: String,
    pub project_id: String,
    pub revision: u32,
    pub generated_at: String,
    pub mode: String,
    pub snapshot_id: String,
    pub mappings: Vec<PlanMapping>,
    pub unknowns: Vec<String>,
    pub decisions: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityReport {
    pub status: String,
    pub blocked: Vec<String>,
    pub needs_approval: Vec<String>,
    pub ok: Vec<String>,
    pub notes: Vec<String>,
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{secs}Z")
}

fn decisions_dir(ws: &Workspace) -> PathBuf {
    redesign_dir(ws).join(DECISIONS_DIR)
}

pub fn plan_path(ws: &Workspace) -> PathBuf {
    redesign_dir(ws).join(PLAN_FILE)
}

/// Classify feasibility from declared backend contracts / atomicity hints.
/// Does **not** invent endpoints — missing atomic ops ⇒ `backend_change_required`.
pub fn classify_feasibility(needs_atomic_backend: bool, has_known_backend_contract: bool, orchestration_possible: bool) -> (&'static str, Vec<String>) {
    if needs_atomic_backend && !has_known_backend_contract {
        (
            "backend_change_required",
            vec![
                "No known atomic backend contract for the combined screens/actions.".into(),
                "Do not invent an endpoint; mark dependency and block Engineering Ready until approved.".into(),
            ],
        )
    } else if has_known_backend_contract && orchestration_possible {
        (
            "frontend_orchestration",
            vec![
                "Existing backend can be composed client-side with race/consistency risks.".into(),
                "Call out partial failure and retry behavior explicitly.".into(),
            ],
        )
    } else if !needs_atomic_backend {
        ("ui_only", vec!["Change stays in presentation/interaction without new contracts.".into()])
    } else {
        ("not_determined", vec!["Insufficient evidence to classify; gather contracts or approve exception.".into()])
    }
}

/// Build a proposed decision from Critic output + feasibility.
pub fn propose_decision(
    problem: &str,
    capability_ids: &[String],
    critic: &CriticReport,
    feasibility: &str,
    dependencies: &[String],
    evidence_ids: &[String],
) -> DesignDecision {
    let options: Vec<DecisionOption> =
        critic.options.iter().map(|o| DecisionOption { id: o.id.clone(), description: o.description.clone(), risks: o.trade_offs.clone() }).collect();
    // Default choice stays keep_as_is until a human/host agent chooses otherwise.
    let chosen = options
        .iter()
        .find(|o| o.id == "keep_as_is")
        .map(|o| o.id.clone())
        .or_else(|| options.first().map(|o| o.id.clone()))
        .unwrap_or_else(|| "keep_as_is".into());
    DesignDecision {
        id: format!("dec_{}", capability_ids.first().map(|s| s.trim_start_matches("cap_")).unwrap_or("screen")),
        capability_ids: capability_ids.to_vec(),
        problem: problem.into(),
        options,
        chosen_option_id: chosen,
        rationale: "Default proposed choice is keep_as_is until explicitly changed and approved when divergent.".into(),
        ux_rule_ids: critic.retrieved_rule_ids.clone(),
        evidence_ids: evidence_ids.to_vec(),
        feasibility: feasibility.into(),
        dependencies: dependencies.to_vec(),
        status: "proposed".into(),
        approval_evidence_id: None,
    }
}

pub fn save_decision(ws: &Workspace, dec: &DesignDecision) -> Result<(), ToolError> {
    redesign_store::ensure(ws)?;
    let dir = decisions_dir(ws);
    fs::create_dir_all(&dir).map_err(|e| ToolError::io(dir.display(), e))?;
    if !dec.id.starts_with("dec_") {
        return Err(ToolError::invalid("decision id must start with dec_"));
    }
    let path = dir.join(format!("{}.json", dec.id));
    let bytes = serde_json::to_vec_pretty(dec).map_err(|e| ToolError::internal(e.to_string()))?;
    write_atomic(&path, &bytes)
}

pub fn load_decision(ws: &Workspace, id: &str) -> Result<DesignDecision, ToolError> {
    let path = decisions_dir(ws).join(format!("{id}.json"));
    let bytes = fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ToolError::new(ErrorCode::NotFound, format!("decision {id} not found"))
        } else {
            ToolError::io(path.display(), e)
        }
    })?;
    serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("decision json: {e}")))
}

pub fn list_decisions(ws: &Workspace) -> Result<Vec<DesignDecision>, ToolError> {
    let dir = decisions_dir(ws);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for ent in fs::read_dir(&dir).map_err(|e| ToolError::io(dir.display(), e))? {
        let ent = ent.map_err(|e| ToolError::io(dir.display(), e))?;
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path).map_err(|e| ToolError::io(path.display(), e))?;
        let d: DesignDecision = serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(e.to_string()))?;
        out.push(d);
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Choosing anything other than keep_as_is (or changing a prior approved choice) requires approval.
pub fn choose_option(ws: &Workspace, decision_id: &str, option_id: &str) -> Result<DesignDecision, ToolError> {
    let mut dec = load_decision(ws, decision_id)?;
    if !dec.options.iter().any(|o| o.id == option_id) {
        return Err(ToolError::invalid(format!("option {option_id} not in decision")));
    }
    let divergent = option_id != "keep_as_is";
    dec.chosen_option_id = option_id.into();
    if divergent {
        dec.status = "proposed".into();
        dec.rationale = format!("Divergent choice `{option_id}` requires explicit approval before build.");
        dec.approval_evidence_id = None;
    } else {
        dec.status = "proposed".into();
        dec.rationale = "keep_as_is selected; still recorded as proposed until acknowledged.".into();
    }
    save_decision(ws, &dec)?;
    Ok(dec)
}

pub fn approve_decision(ws: &Workspace, decision_id: &str, approval_evidence_id: &str) -> Result<DesignDecision, ToolError> {
    let mut dec = load_decision(ws, decision_id)?;
    if dec.chosen_option_id != "keep_as_is" && approval_evidence_id.trim().is_empty() {
        return Err(ToolError::invalid("divergent decisions require approval_evidence_id"));
    }
    dec.status = "approved".into();
    dec.approval_evidence_id = Some(approval_evidence_id.into());
    save_decision(ws, &dec)?;
    Ok(dec)
}

/// Build a transformation plan from capabilities + decisions. Does not invent routes.
pub fn build_plan(ws: &Workspace, mode: &str, mappings: Vec<PlanMapping>, unknowns: Vec<String>) -> Result<TransformationPlan, ToolError> {
    let mode = super::redesign_mode::RedesignMode::parse(mode)?;
    redesign_store::ensure(ws)?;
    let decisions = list_decisions(ws)?;
    let plan = TransformationPlan {
        schema_version: SCHEMA_VERSION.into(),
        project_id: ws.manifest.name.clone(),
        revision: 1,
        generated_at: now_iso(),
        mode: mode.as_str().into(),
        snapshot_id: format!("snap_{}", now_iso()),
        mappings,
        unknowns,
        decisions: decisions.iter().map(|d| d.id.clone()).collect(),
        notes: vec![
            "Plan only maps known destinations; missing targets stay blocked unless exception_approved.".into(),
            "backend_change_required never fabricates contracts.".into(),
        ],
    };
    let bytes = serde_json::to_vec_pretty(&plan).map_err(|e| ToolError::internal(e.to_string()))?;
    write_atomic(&plan_path(ws), &bytes)?;
    Ok(plan)
}

pub fn load_plan(ws: &Workspace) -> Result<TransformationPlan, ToolError> {
    let path = plan_path(ws);
    let bytes = fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ToolError::new(ErrorCode::NotFound, "transformation plan missing")
        } else {
            ToolError::io(path.display(), e)
        }
    })?;
    serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("plan json: {e}")))
}

/// Engineering Ready must not ship while any mapping still needs a backend contract.
/// Approving a `backend_change_required` decision clears traceability, but does **not**
/// invent the contract — those mappings keep Engineering Ready false.
/// `not_determined` is allowed only with an explicit `exception_approved` (unknown trackers).
pub fn backend_contracts_clear(plan: &TransformationPlan) -> (bool, Vec<String>) {
    let mut blocking = Vec::new();
    for m in &plan.mappings {
        match m.feasibility.as_str() {
            "backend_change_required" => {
                blocking.push(format!("{}: backend_change_required (approved gap ≠ known contract)", m.capability_id));
            }
            "not_determined" if !m.exception_approved => {
                blocking.push(format!("{}: not_determined without exception_approved", m.capability_id));
            }
            _ => {}
        }
    }
    (blocking.is_empty(), blocking)
}

/// Critical capabilities need a destination or approved exception; divergent decisions need approval.
pub fn traceability_check(ws: &Workspace, caps: &[Capability], plan: &TransformationPlan) -> Result<TraceabilityReport, ToolError> {
    let decisions = list_decisions(ws)?;
    let dec_by_id: std::collections::HashMap<_, _> = decisions.into_iter().map(|d| (d.id.clone(), d)).collect();
    let mut blocked = Vec::new();
    let mut needs_approval = Vec::new();
    let mut ok = Vec::new();

    for cap in caps {
        let mapping = plan.mappings.iter().find(|m| m.capability_id == cap.id);
        let critical = cap.criticality == "critical" || cap.criticality == "high";
        match mapping {
            None if critical => blocked.push(format!("{}: critical capability has no plan mapping", cap.id)),
            None => {}
            Some(m) => {
                let has_dest = m.target_route.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
                    || m.target_component.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
                    || m.target_action.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
                if critical && !has_dest && !m.exception_approved {
                    blocked.push(format!("{}: critical capability has no destination and no approved exception", cap.id));
                } else if let Some(dec) = dec_by_id.get(&m.decision_id) {
                    if dec.chosen_option_id != "keep_as_is" && dec.status != "approved" {
                        needs_approval.push(format!("{}: divergent decision {} status={}", cap.id, dec.id, dec.status));
                    } else if m.feasibility == "backend_change_required" && dec.status != "approved" {
                        blocked.push(format!("{}: backend_change_required without approved decision", cap.id));
                    } else {
                        ok.push(cap.id.clone());
                    }
                } else if critical {
                    blocked.push(format!("{}: mapping references missing decision {}", cap.id, m.decision_id));
                } else {
                    ok.push(cap.id.clone());
                }
            }
        }
    }

    let status = if !blocked.is_empty() {
        "blocked"
    } else if !needs_approval.is_empty() {
        "needs_approval"
    } else {
        "pass"
    };

    Ok(TraceabilityReport {
        status: status.into(),
        blocked,
        needs_approval,
        ok,
        notes: vec![
            "Traceability is closed-world over known capabilities only.".into(),
            "Unknowns in the plan are preserved and do not count as covered.".into(),
        ],
    })
}

/// Convenience: diagnose a problem with bundled knowledge + feasibility hints.
pub fn diagnose(
    problem: &str,
    tags: &[String],
    capability_ids: &[String],
    needs_atomic_backend: bool,
    has_known_backend_contract: bool,
    orchestration_possible: bool,
) -> Result<Value, ToolError> {
    let (feasibility, deps) = classify_feasibility(needs_atomic_backend, has_known_backend_contract, orchestration_possible);
    let packs = redesign_knowledge::bundled_catalog();
    let rules = redesign_knowledge::retrieve(problem, tags, &packs, redesign_knowledge::RETRIEVE_MAX)?;
    let critic = redesign_knowledge::critic(problem, &rules);
    let decision = propose_decision(problem, capability_ids, &critic, feasibility, &deps, &[]);
    Ok(json!({
        "feasibility": feasibility,
        "dependencies": deps,
        "critic": critic,
        "proposed_decision": decision,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::{TargetRef, Workspace};

    fn cap(id: &str, criticality: &str) -> Capability {
        Capability {
            id: id.into(),
            title: id.into(),
            task: "task".into(),
            actor_roles: vec!["user".into()],
            original_routes: vec!["/a".into()],
            inputs: vec![],
            actions: vec![],
            outcomes: vec![],
            criticality: criticality.into(),
            knowledge_status: "observed".into(),
            source_refs: vec![],
            unknowns: vec![],
        }
    }

    #[test]
    fn three_screens_without_atomic_endpoint_need_backend() {
        let (f, deps) = classify_feasibility(true, false, false);
        assert_eq!(f, "backend_change_required");
        assert!(deps.iter().any(|d| d.contains("atomic")));
    }

    #[test]
    fn known_backend_allows_orchestration_with_risks() {
        let (f, deps) = classify_feasibility(true, true, true);
        assert_eq!(f, "frontend_orchestration");
        assert!(deps.iter().any(|d| d.to_ascii_lowercase().contains("risk")));
    }

    #[test]
    fn divergent_choice_requires_approval_before_pass() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Plan", "p", TargetRef::None).unwrap();
        let critic = redesign_knowledge::critic(
            "dense table",
            &redesign_knowledge::retrieve("dense table", &["table".into()], &redesign_knowledge::bundled_catalog(), 5).unwrap(),
        );
        let mut dec = propose_decision("dense table", &["cap_list".into()], &critic, "ui_only", &[], &[]);
        save_decision(&ws, &dec).unwrap();
        dec = choose_option(&ws, &dec.id, "targeted_improvement").unwrap();
        assert_eq!(dec.status, "proposed");
        assert!(dec.rationale.contains("approval"));

        let plan = build_plan(
            &ws,
            "redesign",
            vec![PlanMapping {
                capability_id: "cap_list".into(),
                target_route: Some("/requests".into()),
                target_component: Some("RequestList".into()),
                target_action: None,
                decision_id: dec.id.clone(),
                scenario_ids: vec![],
                exception_approved: false,
                feasibility: "ui_only".into(),
            }],
            vec![],
        )
        .unwrap();
        let report = traceability_check(&ws, &[cap("cap_list", "critical")], &plan).unwrap();
        assert_eq!(report.status, "needs_approval", "{report:?}");

        approve_decision(&ws, &dec.id, "ev_human_ok").unwrap();
        let report2 = traceability_check(&ws, &[cap("cap_list", "critical")], &plan).unwrap();
        assert_eq!(report2.status, "pass", "{report2:?}");
    }

    #[test]
    fn critical_without_destination_blocks() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Plan", "p", TargetRef::None).unwrap();
        let critic = redesign_knowledge::critic("x", &[]);
        let dec = propose_decision("x", &["cap_finalize".into()], &critic, "backend_change_required", &["atomic_batch".into()], &[]);
        save_decision(&ws, &dec).unwrap();
        approve_decision(&ws, &dec.id, "ev_ok").unwrap();
        let plan = build_plan(
            &ws,
            "redesign",
            vec![PlanMapping {
                capability_id: "cap_finalize".into(),
                target_route: None,
                target_component: None,
                target_action: None,
                decision_id: dec.id.clone(),
                scenario_ids: vec![],
                exception_approved: false,
                feasibility: "backend_change_required".into(),
            }],
            vec!["atomicity of batch finalize unknown".into()],
        )
        .unwrap();
        let report = traceability_check(&ws, &[cap("cap_finalize", "critical")], &plan).unwrap();
        assert_eq!(report.status, "blocked", "{report:?}");
        assert!(report.blocked.iter().any(|b| b.contains("no destination")));
    }

    #[test]
    fn approved_backend_gap_clears_traceability_but_not_contracts() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Plan", "p", TargetRef::None).unwrap();
        let critic = redesign_knowledge::critic("x", &[]);
        let dec = propose_decision("x", &["cap_wayback".into()], &critic, "backend_change_required", &["snapshot api".into()], &[]);
        save_decision(&ws, &dec).unwrap();
        approve_decision(&ws, &dec.id, "ev_ok").unwrap();
        let plan = build_plan(
            &ws,
            "redesign",
            vec![PlanMapping {
                capability_id: "cap_wayback".into(),
                target_route: Some("/wayback".into()),
                target_component: Some("Calendar".into()),
                target_action: Some("browse".into()),
                decision_id: dec.id.clone(),
                scenario_ids: vec![],
                exception_approved: false,
                feasibility: "backend_change_required".into(),
            }],
            vec![],
        )
        .unwrap();
        let report = traceability_check(&ws, &[cap("cap_wayback", "critical")], &plan).unwrap();
        assert_eq!(report.status, "pass", "{report:?}");
        let (clear, blocking) = backend_contracts_clear(&plan);
        assert!(!clear, "{blocking:?}");
        assert!(blocking.iter().any(|b| b.contains("backend_change_required")));
    }

    #[test]
    fn diagnose_keeps_keep_as_is_and_no_fake_endpoint() {
        let v =
            diagnose("combine three approval screens into one submit", &["form".into()], &["cap_a".into(), "cap_b".into(), "cap_c".into()], true, false, false)
                .unwrap();
        assert_eq!(v["feasibility"], "backend_change_required");
        let blob = v.to_string();
        assert!(!blob.to_ascii_lowercase().contains("https://api.invented"));
        assert!(v["proposed_decision"]["options"].as_array().unwrap().iter().any(|o| o["id"] == "keep_as_is"));
    }
}
