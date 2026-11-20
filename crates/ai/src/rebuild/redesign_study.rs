//! Map Craft Study artifacts (and optional fixture oracles) into redesign capabilities/coverage.
//! Does not crawl the web itself — reuses capture/ written by Craft Study.

use super::kinds::website;
use super::redesign_store::{self, Capability, EvidenceRecord};
use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageReport {
    pub covered: u32,
    pub partial: u32,
    pub uncovered: u32,
    pub unknown: u32,
    /// Never claim 100% when unknowns remain or the denominator is incomplete.
    pub percent_claimed: Option<f64>,
    pub unknowns: Vec<String>,
    pub notes: Vec<String>,
}

/// Import a fixture oracle JSON (`scenarios` + `unknowns`) into the redesign store.
pub fn import_oracle_file(ws: &Workspace, path: &Path) -> Result<Value, ToolError> {
    let bytes = fs::read(path).map_err(|e| ToolError::io(path.display(), e))?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("oracle json: {e}")))?;
    import_oracle(ws, &v)
}

pub fn import_oracle(ws: &Workspace, oracle: &Value) -> Result<Value, ToolError> {
    redesign_store::ensure(ws)?;
    let fixture_id = oracle.get("fixture_id").and_then(|x| x.as_str()).unwrap_or("oracle");
    let mut imported_caps = 0u32;
    let mut imported_ev = 0u32;
    if let Some(scenarios) = oracle.get("scenarios").and_then(|s| s.as_array()) {
        for (i, sc) in scenarios.iter().enumerate() {
            let cap_id = sc
                .get("capability_id")
                .and_then(|x| x.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| format!("cap_{fixture_id}_{i}"));
            let scen_id = sc.get("id").and_then(|x| x.as_str()).unwrap_or("scenario");
            let then = sc
                .get("then")
                .and_then(|t| t.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("; "))
                .unwrap_or_default();
            let ev_id = format!("ev_{scen_id}");
            let source = match sc.get("assertion_source").and_then(|x| x.as_str()) {
                Some("observed") => "observed",
                Some("declared") | Some("user_approved") => "declared",
                _ => "inferred",
            };
            redesign_store::put_evidence(
                ws,
                EvidenceRecord {
                    id: ev_id.clone(),
                    source: "test".into(),
                    source_operation: "oracle_import".into(),
                    subject: fixture_id.into(),
                    kind: "behavior".into(),
                    status: source.into(),
                    scope: json!({"route": "/", "role": sc.get("profile").cloned().unwrap_or(json!("")), "environment": "fixture", "app_revision": "", "target_origin": ""}),
                    locator: Some(json!({"artifact_path": format!("oracle/{scen_id}")})),
                    digest: None,
                    observed_at: oracle.get("captured_at").and_then(|x| x.as_str()).unwrap_or("").into(),
                    limitations: vec!["Imported from independent fixture oracle; not live Study.".into()],
                    sensitivity: "public".into(),
                    summary: then.clone(),
                },
            )?;
            imported_ev += 1;
            redesign_store::put_capability(
                ws,
                Capability {
                    id: cap_id,
                    title: scen_id.into(),
                    task: then,
                    actor_roles: vec![sc.get("profile").and_then(|x| x.as_str()).unwrap_or("user").into()],
                    original_routes: vec!["/".into()],
                    inputs: sc
                        .get("given")
                        .and_then(|g| g.as_array())
                        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                        .unwrap_or_default(),
                    actions: sc
                        .get("when")
                        .and_then(|g| g.as_array())
                        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                        .unwrap_or_default(),
                    outcomes: vec![],
                    criticality: "normal".into(),
                    knowledge_status: source.into(),
                    source_refs: vec![ev_id],
                    unknowns: vec![],
                },
            )?;
            imported_caps += 1;
        }
    }
    let unknowns: Vec<String> = oracle
        .get("unknowns")
        .and_then(|u| u.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    // Preserve intentional unknowns as a capability gap record.
    if !unknowns.is_empty() {
        redesign_store::put_capability(
            ws,
            Capability {
                id: format!("cap_{fixture_id}_unknowns"),
                title: "Declared unknowns".into(),
                task: "Items not observed or not determined".into(),
                actor_roles: vec![],
                original_routes: vec![],
                inputs: vec![],
                actions: vec![],
                outcomes: vec![],
                criticality: "high".into(),
                knowledge_status: "unknown".into(),
                source_refs: vec![],
                unknowns: unknowns.clone(),
            },
        )?;
        imported_caps += 1;
    }
    Ok(json!({
        "imported_evidence": imported_ev,
        "imported_capabilities": imported_caps,
        "unknowns": unknowns,
        "coverage": coverage_of(ws)?
    }))
}

/// Map Craft website capture routes into observed route capabilities (partial Study adapter).
pub fn map_capture(ws: &Workspace) -> Result<Value, ToolError> {
    redesign_store::ensure(ws)?;
    let routes = website::routes(ws);
    let mut mapped = 0u32;
    for r in &routes {
        let id = format!("cap_route_{}", r.slug);
        redesign_store::put_evidence(
            ws,
            EvidenceRecord {
                id: format!("ev_route_{}", r.slug),
                source: "craft".into(),
                source_operation: "web_site_capture".into(),
                subject: r.path.clone(),
                kind: "ui".into(),
                status: "observed".into(),
                scope: json!({"route": r.path, "target_origin": r.url, "role": "", "environment": "capture", "app_revision": ""}),
                locator: Some(json!({"artifact_path": format!("capture/routes/{}/", r.slug)})),
                digest: None,
                observed_at: "".into(),
                limitations: vec!["Route listed from capture links; behavior inside the page may be incomplete.".into()],
                sensitivity: "public".into(),
                summary: format!("Observed route {} ({})", r.path, r.url),
            },
        )?;
        redesign_store::put_capability(
            ws,
            Capability {
                id,
                title: format!("Route {}", r.path),
                task: format!("Navigate to {}", r.path),
                actor_roles: vec!["user".into()],
                original_routes: vec![r.path.clone()],
                inputs: vec![],
                actions: vec!["navigate".into()],
                outcomes: vec![],
                criticality: "normal".into(),
                knowledge_status: "observed".into(),
                source_refs: vec![format!("ev_route_{}", r.slug)],
                unknowns: vec!["In-page rules not fully extracted by this mapper.".into()],
            },
        )?;
        mapped += 1;
    }
    // If capture.json text exists, record a single observed evidence without claiming completeness.
    let capture_json = ws.capture_dir().join("capture.json");
    if capture_json.is_file() {
        redesign_store::put_evidence(
            ws,
            EvidenceRecord {
                id: "ev_capture_start".into(),
                source: "craft".into(),
                source_operation: "web_site_capture".into(),
                subject: "start_page".into(),
                kind: "ui".into(),
                status: "observed".into(),
                scope: json!({}),
                locator: Some(json!({"artifact_path": "capture/capture.json"})),
                digest: None,
                observed_at: "".into(),
                limitations: vec!["Start-page capture present; hidden rules remain unknown.".into()],
                sensitivity: "public".into(),
                summary: "Start page capture exists from Craft Study.".into(),
            },
        )?;
    }
    Ok(json!({
        "routes_mapped": mapped,
        "coverage": coverage_of(ws)?
    }))
}

pub fn coverage_of(ws: &Workspace) -> Result<CoverageReport, ToolError> {
    let caps = redesign_store::list_capabilities(ws)?;
    let mut covered = 0u32;
    let mut partial = 0u32;
    let mut uncovered = 0u32;
    let mut unknown = 0u32;
    let mut unknowns = Vec::new();
    for c in &caps {
        unknowns.extend(c.unknowns.clone());
        match c.knowledge_status.as_str() {
            "observed" | "validated" if c.unknowns.is_empty() => covered += 1,
            "observed" | "validated" | "declared" | "inferred" => partial += 1,
            "unknown" => unknown += 1,
            _ => uncovered += 1,
        }
    }
    let notes = vec![
        "Coverage is over mapped capabilities only; absence of a capability is not proof it does not exist.".into(),
        "percent_claimed stays null when unknowns remain or the denominator is incomplete.".into(),
    ];
    let percent_claimed = if unknowns.is_empty() && unknown == 0 && (covered + partial + uncovered) > 0 {
        // Still refuse 100% — Study is never complete by this mapper.
        None
    } else {
        None
    };
    Ok(CoverageReport { covered, partial, uncovered, unknown, percent_claimed, unknowns, notes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::TargetRef;
    use super::website::{Route, ROUTES_FILE};

    #[test]
    fn oracle_import_keeps_unknowns_and_never_claims_100() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Fix", "kind=redesign", TargetRef::None).unwrap();
        let oracle = json!({
            "fixture_id": "dense-form",
            "captured_at": "2026-11-20T00:00:00Z",
            "scenarios": [{
                "id": "scenario_dense_submit_gate",
                "capability_id": "cap_dense_submit_enablement",
                "profile": "employee",
                "given": ["Form loaded"],
                "when": ["Fill required fields"],
                "then": ["Submit enables"],
                "assertion_source": "user_approved"
            }],
            "unknowns": ["Server duplicate-claim policy", "Tax calculation formula"]
        });
        let out = import_oracle(&ws, &oracle).unwrap();
        assert_eq!(out["imported_capabilities"], 2);
        let cov = coverage_of(&ws).unwrap();
        assert!(cov.unknowns.iter().any(|u| u.contains("duplicate")));
        assert!(cov.percent_claimed.is_none(), "must not claim 100%: {cov:?}");
        let caps = redesign_store::list_capabilities(&ws).unwrap();
        assert!(caps.iter().any(|c| c.id == "cap_dense_submit_enablement"));
        assert!(caps.iter().any(|c| c.knowledge_status == "unknown"));
    }

    #[test]
    fn capture_routes_become_observed_capabilities_with_gaps() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Site", "kind=redesign", TargetRef::Website { url: "https://example.test".into() }).unwrap();
        let routes = vec![Route { path: "/about".into(), slug: "about".into(), url: "https://example.test/about".into() }];
        fs::create_dir_all(ws.capture_dir()).unwrap();
        fs::write(ws.capture_dir().join(ROUTES_FILE), serde_json::to_vec(&routes).unwrap()).unwrap();
        fs::write(ws.capture_dir().join("capture.json"), b"{\"text\":\"hi\"}").unwrap();
        let out = map_capture(&ws).unwrap();
        assert_eq!(out["routes_mapped"], 1);
        let caps = redesign_store::list_capabilities(&ws).unwrap();
        assert!(caps.iter().any(|c| c.id == "cap_route_about"));
        assert!(caps.iter().any(|c| !c.unknowns.is_empty()));
    }
}
