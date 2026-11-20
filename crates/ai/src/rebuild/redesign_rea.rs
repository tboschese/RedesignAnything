//! PR-10: REA Deep adapter (opt-in, budgeted, Clean Room blocked).
//!
//! Contracts + host probe + optional **live** Node adapter invocation.
//! Default adapter is the in-repo fixture (`REDESIGN_SPEC/fixtures/rea-live/adapter.mjs`),
//! not a vendored copy of upstream REA. Override with `REDESIGN_REA_ADAPTER`.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::{ErrorCode, ToolError};
use craft_anything_platform::Process;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::redesign_store;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaDiscovery {
    pub available: bool,
    pub node_ok: bool,
    pub rea_version: Option<String>,
    #[serde(default)]
    pub adapter_path: Option<String>,
    #[serde(default)]
    pub adapter_kind: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaCallRecord {
    pub operation: String,
    pub reason: String,
    pub evidence_gain: String,
    pub status: String,
    pub budget_remaining: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaSession {
    pub consent_granted: bool,
    pub clean_room: bool,
    pub deep_profile: bool,
    pub budget_calls: u32,
    pub calls_used: u32,
    pub records: Vec<ReaCallRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaLiveResult {
    pub record: ReaCallRecord,
    pub discovery: ReaDiscovery,
    pub session: ReaSession,
    pub finding: Option<Value>,
    pub evidence_path: Option<String>,
    pub live: bool,
    pub notes: Vec<String>,
}

impl ReaSession {
    pub fn new(consent_granted: bool, clean_room: bool, deep_profile: bool, budget_calls: u32) -> Self {
        Self { consent_granted, clean_room, deep_profile, budget_calls, calls_used: 0, records: vec![] }
    }
}

const FIRST_BATCH: &[&str] = &["analyze_web_bundle", "trace_javascript_semantics", "inspect_web_network_capture"];

pub fn allowed_operations() -> &'static [&'static str] {
    FIRST_BATCH
}

/// Probe whether REA could run. `rea_present` is injected for tests / host probe.
pub fn discover(node_ok: bool, rea_present: bool, rea_version: Option<String>) -> ReaDiscovery {
    if !node_ok {
        return ReaDiscovery {
            available: false,
            node_ok: false,
            rea_version: None,
            adapter_path: None,
            adapter_kind: "none".into(),
            notes: vec!["Node.js missing or incompatible; REA Deep unavailable — lean path continues.".into()],
        };
    }
    if !rea_present {
        return ReaDiscovery {
            available: false,
            node_ok: true,
            rea_version: None,
            adapter_path: None,
            adapter_kind: "none".into(),
            notes: vec!["REA not installed; fallback to Craft Study + redesign lean tools.".into()],
        };
    }
    ReaDiscovery {
        available: true,
        node_ok: true,
        rea_version,
        adapter_path: None,
        adapter_kind: "injected".into(),
        notes: vec!["REA discovered; still requires consent + deep profile + budget.".into()],
    }
}

fn which_exists(program: &str) -> bool {
    let Ok(path_env) = std::env::var("PATH") else {
        return false;
    };
    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(program);
        if candidate.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{program}.exe"));
            if with_exe.is_file() {
                return true;
            }
            let with_cmd = dir.join(format!("{program}.cmd"));
            if with_cmd.is_file() {
                return true;
            }
        }
    }
    false
}

pub fn probe_node() -> bool {
    if !which_exists("node") {
        return false;
    }
    match Process::new("node").arg("--version").timeout(Duration::from_secs(5)).stdout_cap(4 * 1024).stderr_cap(4 * 1024).run() {
        Ok(out) => out.success,
        Err(_) => false,
    }
}

/// Resolve adapter: `REDESIGN_REA_ADAPTER` env, else in-repo fixture `adapter.mjs`.
pub fn resolve_adapter() -> (Option<PathBuf>, &'static str) {
    if let Ok(raw) = std::env::var("REDESIGN_REA_ADAPTER") {
        let p = PathBuf::from(raw.trim());
        if p.is_file() {
            return (Some(p), "env");
        }
    }
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/rea-live/adapter.mjs");
    if let Ok(canon) = fixture.canonicalize() {
        if canon.is_file() {
            return (Some(canon), "fixture");
        }
    }
    if fixture.is_file() {
        return (Some(fixture), "fixture");
    }
    (None, "none")
}

/// Host probe: Node on PATH + adapter file present.
pub fn discover_host() -> ReaDiscovery {
    let node_ok = probe_node();
    let (adapter, kind) = resolve_adapter();
    if !node_ok {
        return discover(false, false, None);
    }
    match adapter {
        Some(path) => {
            let version = if kind == "fixture" { Some("fixture-0.1.0".into()) } else { Some("env-adapter".into()) };
            let mut d = discover(true, true, version);
            d.adapter_path = Some(path.display().to_string());
            d.adapter_kind = kind.into();
            d.notes.push(format!("Live adapter ready ({kind}). Opt-in only; Clean Room still blocks execution."));
            d
        }
        None => {
            let mut d = discover(true, false, None);
            d.adapter_kind = "none".into();
            d.notes.push("Set REDESIGN_REA_ADAPTER to a Node entry script, or keep the fixture under REDESIGN_SPEC/fixtures/rea-live/.".into());
            d
        }
    }
}

/// Gate a Deep call. Clean Room always blocks. Consent + deep + budget required.
pub fn request_call(session: &mut ReaSession, discovery: &ReaDiscovery, operation: &str, reason: &str) -> Result<ReaCallRecord, ToolError> {
    if session.clean_room {
        return Err(ToolError::new(ErrorCode::FeatureMissing, "REA Deep blocked in Clean Room").with_details(json!({"code":"clean_room_blocked"})));
    }
    if !session.consent_granted {
        return Err(ToolError::new(ErrorCode::ConsentRequired, "REA Deep requires explicit consent").with_details(json!({"code":"rea_consent_required"})));
    }
    if !session.deep_profile {
        return Err(ToolError::new(ErrorCode::FeatureMissing, "REA Deep requires deep execution profile").with_details(json!({"code":"deep_required"})));
    }
    if !discovery.available {
        let rec = ReaCallRecord {
            operation: operation.into(),
            reason: reason.into(),
            evidence_gain: "none".into(),
            status: "fallback".into(),
            budget_remaining: session.budget_calls.saturating_sub(session.calls_used),
        };
        session.records.push(rec.clone());
        return Ok(rec);
    }
    if !FIRST_BATCH.contains(&operation) {
        return Err(ToolError::invalid(format!("operation {operation} not in first REA batch")));
    }
    if reason.trim().is_empty() {
        return Err(ToolError::invalid("REA call requires a justified reason"));
    }
    if session.calls_used >= session.budget_calls {
        return Err(ToolError::new(ErrorCode::OverLimit, "REA call budget exhausted").with_details(json!({"code":"rea_budget"})));
    }
    session.calls_used += 1;
    let remaining = session.budget_calls.saturating_sub(session.calls_used);
    let rec = ReaCallRecord {
        operation: operation.into(),
        reason: reason.into(),
        evidence_gain: "incremental_pending_execution".into(),
        status: "allowed".into(),
        budget_remaining: remaining,
    };
    session.records.push(rec.clone());
    Ok(rec)
}

/// Convert a successful REA-ish finding into evidence-shaped JSON (no egress of secrets).
pub fn evidence_from_finding(operation: &str, finding: &str, canary: &str) -> Result<Value, ToolError> {
    if finding.contains(canary) || finding.to_ascii_lowercase().contains("authorization: bearer") {
        return Err(ToolError::new(ErrorCode::NoPermission, "refusing REA finding that contains canary/secret").with_details(json!({"code":"egress_canary"})));
    }
    Ok(json!({
        "source": "rea_deep",
        "source_operation": operation,
        "status": "observed",
        "summary": finding,
        "limitations": ["Experimental Deep adapter; validate before promoting knowledge_status."],
        "sensitivity": "internal",
    }))
}

fn invoke_adapter(adapter: &Path, operation: &str, input: &Path, out_file: &Path) -> Result<Value, ToolError> {
    if !input.is_file() {
        return Err(ToolError::invalid(format!("REA input not found: {}", input.display())));
    }
    if let Some(parent) = out_file.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::io(parent.display(), e))?;
    }
    let out = Process::new("node")
        .arg(adapter)
        .arg("--operation")
        .arg(operation)
        .arg("--input")
        .arg(input)
        .arg("--out")
        .arg(out_file)
        .timeout(Duration::from_secs(30))
        .stdout_cap(512 * 1024)
        .stderr_cap(64 * 1024)
        .label("rea-live-adapter")
        .remediation("Install Node.js and ensure REDESIGN_REA_ADAPTER (or the fixture adapter) is readable.")
        .run()
        .map_err(|e| {
            ToolError::new(ErrorCode::ProgramFailed, format!("REA live adapter failed to start: {}", e.message))
                .with_details(json!({"code":"rea_adapter_spawn"}))
        })?;
    if !out.success {
        return Err(ToolError::new(ErrorCode::ProgramFailed, format!("REA live adapter exited non-zero: {}", out.stderr_end_text().trim()))
            .with_details(json!({"code":"rea_adapter_failed"})));
    }
    let text = if out_file.is_file() { fs::read_to_string(out_file).map_err(|e| ToolError::io(out_file.display(), e))? } else { out.stdout_text() };
    let finding: Value = serde_json::from_str(text.trim()).map_err(|e| ToolError::invalid(format!("REA adapter output is not JSON: {e}")))?;
    // Refuse if adapter claims network egress.
    if finding.get("network_egress").and_then(|v| v.as_bool()) == Some(true) {
        return Err(
            ToolError::new(ErrorCode::NoPermission, "refusing REA finding that claims network_egress=true").with_details(json!({"code":"rea_egress_claimed"}))
        );
    }
    let blob = finding.to_string();
    evidence_from_finding(operation, &blob, "CANARY_TOKEN_XYZ")?;
    Ok(finding)
}

/// Gate then optionally run the live Node adapter against a local input file.
pub fn run_live(
    ws: &Workspace,
    session: &mut ReaSession,
    discovery: &ReaDiscovery,
    operation: &str,
    reason: &str,
    input: &Path,
) -> Result<ReaLiveResult, ToolError> {
    let record = request_call(session, discovery, operation, reason)?;
    if record.status == "fallback" {
        return Ok(ReaLiveResult {
            record,
            discovery: discovery.clone(),
            session: session.clone(),
            finding: None,
            evidence_path: None,
            live: false,
            notes: vec!["Adapter unavailable — lean redesign continues; no fabricated findings.".into()],
        });
    }
    let Some(adapter) = discovery.adapter_path.as_ref().map(PathBuf::from).filter(|p| p.is_file()).or_else(|| resolve_adapter().0) else {
        return Ok(ReaLiveResult {
            record: ReaCallRecord { evidence_gain: "none".into(), status: "fallback".into(), ..record },
            discovery: discovery.clone(),
            session: session.clone(),
            finding: None,
            evidence_path: None,
            live: false,
            notes: vec!["Call allowed but adapter path missing at execution time.".into()],
        });
    };

    redesign_store::ensure(ws)?;
    let evidence_dir = redesign_store::redesign_dir(ws).join("rea").join("findings");
    fs::create_dir_all(&evidence_dir).map_err(|e| ToolError::io(evidence_dir.display(), e))?;
    let stamp = session.calls_used;
    let out_file = evidence_dir.join(format!("{operation}-{stamp}.json"));
    let finding = invoke_adapter(&adapter, operation, input, &out_file)?;
    let evidence = evidence_from_finding(operation, &finding.to_string(), "CANARY_TOKEN_XYZ")?;
    let evidence_path = redesign_store::redesign_dir(ws).join("rea").join("evidence").join(format!("{operation}-{stamp}.json"));
    if let Some(parent) = evidence_path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::io(parent.display(), e))?;
    }
    craft_anything_engine::project::write_atomic(
        &evidence_path,
        serde_json::to_vec_pretty(&evidence).map_err(|e| ToolError::internal(e.to_string()))?.as_slice(),
    )?;

    // Mark record as executed.
    if let Some(last) = session.records.last_mut() {
        last.status = "executed".into();
        last.evidence_gain = "fixture_or_adapter_observed".into();
    }
    let record = session.records.last().cloned().unwrap_or(record);

    Ok(ReaLiveResult {
        record,
        discovery: discovery.clone(),
        session: session.clone(),
        finding: Some(finding),
        evidence_path: Some(evidence_path.strip_prefix(&ws.root).unwrap_or(&evidence_path).display().to_string()),
        live: true,
        notes: vec![format!("Live adapter kind={}", discovery.adapter_kind), "Finding is observed from local input only; not production-validated.".into()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::{TargetRef, Workspace};

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/rea-live").canonicalize().unwrap()
    }

    #[test]
    fn missing_rea_falls_back() {
        let d = discover(true, false, None);
        assert!(!d.available);
        let mut s = ReaSession::new(true, false, true, 3);
        let r = request_call(&mut s, &d, "analyze_web_bundle", "gap in bundle facts").unwrap();
        assert_eq!(r.status, "fallback");
    }

    #[test]
    fn clean_room_blocks() {
        let d = discover(true, true, Some("0.1.0".into()));
        let mut s = ReaSession::new(true, true, true, 3);
        let err = request_call(&mut s, &d, "analyze_web_bundle", "need symbols").unwrap_err();
        assert_eq!(err.code, ErrorCode::FeatureMissing);
    }

    #[test]
    fn consent_and_budget_enforced() {
        let d = discover(true, true, Some("0.1.0".into()));
        let mut s = ReaSession::new(false, false, true, 1);
        assert_eq!(request_call(&mut s, &d, "analyze_web_bundle", "x").unwrap_err().code, ErrorCode::ConsentRequired);
        s.consent_granted = true;
        assert_eq!(request_call(&mut s, &d, "analyze_web_bundle", "justify").unwrap().status, "allowed");
        assert_eq!(request_call(&mut s, &d, "trace_javascript_semantics", "more").unwrap_err().code, ErrorCode::OverLimit);
    }

    #[test]
    fn canary_token_refused_in_evidence() {
        let err = evidence_from_finding("analyze_web_bundle", "leak CANARY_TOKEN_XYZ", "CANARY_TOKEN_XYZ").unwrap_err();
        assert_eq!(err.code, ErrorCode::NoPermission);
    }

    #[test]
    fn first_batch_operations_listed() {
        assert!(allowed_operations().contains(&"analyze_web_bundle"));
        assert!(allowed_operations().contains(&"trace_javascript_semantics"));
        assert!(allowed_operations().contains(&"inspect_web_network_capture"));
    }

    #[test]
    fn fixture_adapter_resolves() {
        let (path, kind) = resolve_adapter();
        assert_eq!(kind, "fixture");
        assert!(path.unwrap().ends_with("adapter.mjs"));
    }

    #[test]
    fn live_run_executes_fixture_adapter_when_gated() {
        if !probe_node() {
            // Host without Node: still prove fallback path with injected discovery.
            let d = discover(true, false, None);
            let dir = tempfile::tempdir().unwrap();
            let mut ws = Workspace::create(dir.path(), "R", "kind=redesign", TargetRef::None).unwrap();
            ws.manifest.kind = Some("redesign".into());
            ws.save().unwrap();
            let mut s = ReaSession::new(true, false, true, 2);
            let r = run_live(&ws, &mut s, &d, "analyze_web_bundle", "need exports", &fixture_dir().join("sample-bundle.js")).unwrap();
            assert!(!r.live);
            assert_eq!(r.record.status, "fallback");
            return;
        }
        let mut d = discover_host();
        assert!(d.available, "fixture adapter should be discoverable: {:?}", d.notes);
        assert_eq!(d.adapter_kind, "fixture");
        let dir = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(dir.path(), "R", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let mut s = ReaSession::new(true, false, true, 2);
        let r = run_live(&ws, &mut s, &d, "analyze_web_bundle", "need export list for redesign", &fixture_dir().join("sample-bundle.js")).unwrap();
        assert!(r.live);
        assert_eq!(r.record.status, "executed");
        let finding = r.finding.unwrap();
        assert_eq!(finding["network_egress"], false);
        assert!(finding["observed"]["export_names"].as_array().unwrap().iter().any(|v| v == "bootstrapApp"));
        assert!(r.evidence_path.as_ref().unwrap().contains("rea/evidence/"));

        let r2 =
            run_live(&ws, &mut s, &d, "inspect_web_network_capture", "need API surface from capture", &fixture_dir().join("sample-capture.har.json")).unwrap();
        assert!(r2.live);
        assert_eq!(r2.finding.unwrap()["observed"]["entry_count"], 2);

        // Clean room still blocks even with live adapter present.
        d = discover_host();
        let mut blocked = ReaSession::new(true, true, true, 1);
        assert!(run_live(&ws, &mut blocked, &d, "analyze_web_bundle", "should block", &fixture_dir().join("sample-bundle.js"),).is_err());
    }
}
