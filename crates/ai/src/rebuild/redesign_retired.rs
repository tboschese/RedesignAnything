//! Retiring a scenario that an approved decision replaces.
//!
//! The oracle says what the original did. A decision the operator approved can replace exactly
//! that (two search fields become one), and then no honest journey can show the scenario. Such a
//! scenario can be retired in this project, with the operator's own approval of the retirement.
//!
//! A retirement is narrow on purpose: it names the decision that replaces the scenario, that
//! decision must decide the scenario's capability, change the original and be approved, and the
//! retirement stops counting the moment the decision is no longer that. It is kept in this
//! project only, so another variant of the same study still has the scenario. A retired scenario
//! is listed as retired, never as passed.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use super::redesign_plan;
use super::redesign_store;

/// The retirements a project keeps, under `redesign/`.
pub const FILE: &str = "retired_scenarios.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Retired {
    /// The id of a scenario in redesign/oracle/scenarios.json.
    pub scenario_id: String,
    /// The approved decision that replaces what the scenario asks for.
    pub decision_id: String,
    /// `ev_…` id of the operator's approval of this retirement (`source: "user"`).
    pub approval_evidence_id: String,
    /// What the decision does instead, in a sentence.
    pub reason: String,
}

impl Retired {
    /// One line for a check's notes and the report.
    pub fn note(&self) -> String {
        format!("replaced by the approved decision {} ({})", self.decision_id, self.reason.trim())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RetiredFile {
    retired: Vec<Retired>,
}

fn path(ws: &Workspace) -> PathBuf {
    redesign_store::redesign_dir(ws).join(FILE)
}

/// The capability each oracle scenario belongs to.
fn scenario_capabilities(ws: &Workspace) -> BTreeMap<String, String> {
    let file = redesign_store::redesign_dir(ws).join("oracle/scenarios.json");
    let oracle: Value = fs::read(file).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    let scenarios = oracle["scenarios"].as_array().cloned().unwrap_or_default();
    scenarios.iter().filter_map(|s| Some((s["id"].as_str()?.to_string(), s["capability_id"].as_str().unwrap_or_default().to_string()))).collect()
}

/// Why `decision_id` can't retire a scenario of `capability` now, if it can't.
fn decision_stands(ws: &Workspace, decision_id: &str, capability: &str) -> Result<(), String> {
    let decision = redesign_plan::load_decision(ws, decision_id).map_err(|e| e.message)?;
    if !decision.capability_ids.iter().any(|c| c == capability) {
        return Err(format!("decision {decision_id} does not decide the scenario's capability ({capability})"));
    }
    if decision.chosen_option_id == "keep_as_is" {
        return Err(format!("decision {decision_id} keeps the original as it is, so it replaces nothing"));
    }
    if decision.status != "approved" {
        return Err(format!("decision {decision_id} is not approved (status {})", decision.status));
    }
    Ok(())
}

/// Every retirement the project keeps, in force or not.
pub fn load(ws: &Workspace) -> Result<Vec<Retired>, ToolError> {
    let file = path(ws);
    if !file.is_file() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(&file).map_err(|e| ToolError::io(file.display(), e))?;
    let kept: RetiredFile = serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("redesign/{FILE}: {e}")))?;
    Ok(kept.retired)
}

fn save(ws: &Workspace, retired: &[Retired]) -> Result<(), ToolError> {
    redesign_store::ensure(ws)?;
    let bytes = serde_json::to_vec_pretty(&RetiredFile { retired: retired.to_vec() }).map_err(|e| ToolError::internal(e.to_string()))?;
    craft_anything_engine::project::write_atomic(&path(ws), &bytes)
}

/// Retire a scenario, replacing an earlier retirement of the same one.
pub fn retire(ws: &Workspace, new: Retired) -> Result<Vec<Retired>, ToolError> {
    let scenarios = scenario_capabilities(ws);
    let Some(capability) = scenarios.get(&new.scenario_id) else {
        return Err(ToolError::invalid(format!("no scenario {:?} in redesign/oracle/scenarios.json", new.scenario_id)));
    };
    if new.reason.trim().is_empty() {
        return Err(ToolError::invalid("a retirement says what the decision does instead of the scenario"));
    }
    decision_stands(ws, &new.decision_id, capability).map_err(|why| {
        ToolError::invalid(why)
            .with_remediation("A scenario is retired by the approved decision that replaces what it asks for. Choose and approve that decision first.")
    })?;
    if !redesign_store::get_evidence(ws, &new.approval_evidence_id)?.is_some_and(|e| e.source == "user") {
        return Err(ToolError::invalid(format!("{} is not the operator's approval", new.approval_evidence_id)).with_remediation(
            "Ask the operator whether this scenario may be retired, record the answer with redesign_evidence_put (source \"user\") and pass its id as approval_evidence_id.",
        ));
    }
    let mut all = load(ws)?;
    all.retain(|r| r.scenario_id != new.scenario_id);
    all.push(new);
    save(ws, &all)?;
    Ok(all)
}

/// Bring a retired scenario back. `false` when it was not retired.
pub fn restore(ws: &Workspace, scenario_id: &str) -> Result<bool, ToolError> {
    let mut all = load(ws)?;
    let before = all.len();
    all.retain(|r| r.scenario_id != scenario_id);
    if all.len() == before {
        return Ok(false);
    }
    save(ws, &all)?;
    Ok(true)
}

/// The retirements that count now, by scenario, and why each of the others does not: its
/// scenario is gone from the oracle, or its decision is no longer the approved one that replaces it.
pub fn in_force(ws: &Workspace) -> (BTreeMap<String, Retired>, Vec<String>) {
    let scenarios = scenario_capabilities(ws);
    let (mut counted, mut lapsed) = (BTreeMap::new(), Vec::new());
    for r in load(ws).unwrap_or_default() {
        let stands = match scenarios.get(&r.scenario_id) {
            None => Err("it is not in the oracle any more".to_string()),
            Some(capability) => decision_stands(ws, &r.decision_id, capability),
        };
        match stands {
            Ok(()) => {
                counted.insert(r.scenario_id.clone(), r);
            }
            Err(why) => lapsed.push(format!("the retirement of {} no longer counts: {why}", r.scenario_id)),
        }
    }
    (counted, lapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rebuild::redesign_plan::DesignDecision;
    use crate::rebuild::redesign_store::EvidenceRecord;
    use craft_anything_engine::workspace::TargetRef;
    use serde_json::json;

    fn decision(id: &str, capability: &str, option: &str, status: &str) -> DesignDecision {
        DesignDecision {
            id: id.into(),
            capability_ids: vec![capability.into()],
            problem: "p".into(),
            options: vec![],
            chosen_option_id: option.into(),
            rationale: String::new(),
            ux_rule_ids: vec![],
            evidence_ids: vec![],
            feasibility: "ui_only".into(),
            dependencies: vec![],
            status: status.into(),
            approval_evidence_id: None,
        }
    }

    fn evidence(id: &str, source: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            source: source.into(),
            source_operation: "answer".into(),
            subject: "s".into(),
            kind: "approval".into(),
            status: "declared".into(),
            scope: Value::Null,
            locator: None,
            digest: None,
            observed_at: String::new(),
            limitations: vec![],
            sensitivity: "internal".into(),
            summary: "yes".into(),
        }
    }

    fn project() -> (tempfile::TempDir, Workspace) {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::None).unwrap();
        let oracle = redesign_store::redesign_dir(&ws).join("oracle");
        fs::create_dir_all(&oracle).unwrap();
        let scenarios = json!({"scenarios": [
            {"id": "two_fields", "capability_id": "cap_search", "then": ["Two separate search fields"]},
            {"id": "donate", "capability_id": "cap_donate", "then": ["Donate stays"]},
        ]});
        fs::write(oracle.join("scenarios.json"), scenarios.to_string()).unwrap();
        redesign_store::put_evidence(&ws, evidence("ev_operator_yes", "user")).unwrap();
        redesign_store::put_evidence(&ws, evidence("ev_agent_note", "craft")).unwrap();
        (d, ws)
    }

    fn asked(scenario: &str, decision: &str, approval: &str) -> Retired {
        Retired { scenario_id: scenario.into(), decision_id: decision.into(), approval_evidence_id: approval.into(), reason: "one box takes both".into() }
    }

    #[test]
    fn a_scenario_is_retired_only_by_the_approved_decision_that_replaces_it_and_with_the_operators_yes() {
        let (_d, ws) = project();
        let save = |d: DesignDecision| redesign_plan::save_decision(&ws, &d).unwrap();
        save(decision("dec_search", "cap_search", "targeted_improvement", "proposed"));
        save(decision("dec_donate", "cap_donate", "keep_as_is", "approved"));
        let refused = |r: Retired| retire(&ws, r).unwrap_err().message;

        assert!(refused(asked("no_such", "dec_search", "ev_operator_yes")).contains("no scenario"));
        assert!(refused(asked("two_fields", "dec_search", "ev_operator_yes")).contains("not approved"));
        save(decision("dec_search", "cap_search", "targeted_improvement", "approved"));
        assert!(refused(asked("donate", "dec_search", "ev_operator_yes")).contains("does not decide"), "another capability's decision");
        assert!(refused(asked("donate", "dec_donate", "ev_operator_yes")).contains("replaces nothing"), "a decision that keeps the original");
        assert!(refused(asked("two_fields", "dec_search", "ev_agent_note")).contains("not the operator's approval"));
        assert!(refused(Retired { reason: " ".into(), ..asked("two_fields", "dec_search", "ev_operator_yes") }).contains("instead"));

        retire(&ws, asked("two_fields", "dec_search", "ev_operator_yes")).unwrap();
        let (counted, lapsed) = in_force(&ws);
        assert!(counted.contains_key("two_fields") && lapsed.is_empty(), "{counted:?} {lapsed:?}");
    }

    #[test]
    fn a_retirement_stops_counting_when_its_decision_is_no_longer_the_approved_one() {
        let (_d, ws) = project();
        redesign_plan::save_decision(&ws, &decision("dec_search", "cap_search", "targeted_improvement", "approved")).unwrap();
        retire(&ws, asked("two_fields", "dec_search", "ev_operator_yes")).unwrap();
        assert_eq!(in_force(&ws).0.len(), 1);

        // The decision goes back to keeping the original: the scenario is asked for again.
        redesign_plan::save_decision(&ws, &decision("dec_search", "cap_search", "keep_as_is", "proposed")).unwrap();
        let (counted, lapsed) = in_force(&ws);
        assert!(counted.is_empty() && lapsed[0].contains("two_fields"), "{counted:?} {lapsed:?}");

        redesign_plan::save_decision(&ws, &decision("dec_search", "cap_search", "targeted_improvement", "approved")).unwrap();
        assert!(restore(&ws, "two_fields").unwrap() && in_force(&ws).0.is_empty(), "restored by hand");
        assert!(!restore(&ws, "two_fields").unwrap(), "nothing left to restore");
    }
}
