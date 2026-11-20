//! PR-10: REA Deep adapter (opt-in, budgeted, Clean Room blocked).
//! Does not ship or invoke REA binaries here — discovery + consent + fallback contracts.

use craft_anything_evidence::{ErrorCode, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaDiscovery {
    pub available: bool,
    pub node_ok: bool,
    pub rea_version: Option<String>,
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

impl ReaSession {
    pub fn new(consent_granted: bool, clean_room: bool, deep_profile: bool, budget_calls: u32) -> Self {
        Self {
            consent_granted,
            clean_room,
            deep_profile,
            budget_calls,
            calls_used: 0,
            records: vec![],
        }
    }
}

/// Probe whether REA could run. `rea_present` is injected for tests / host probe.
pub fn discover(node_ok: bool, rea_present: bool, rea_version: Option<String>) -> ReaDiscovery {
    if !node_ok {
        return ReaDiscovery {
            available: false,
            node_ok: false,
            rea_version: None,
            notes: vec!["Node.js missing or incompatible; REA Deep unavailable — lean path continues.".into()],
        };
    }
    if !rea_present {
        return ReaDiscovery {
            available: false,
            node_ok: true,
            rea_version: None,
            notes: vec!["REA not installed; fallback to Craft Study + redesign lean tools.".into()],
        };
    }
    ReaDiscovery {
        available: true,
        node_ok: true,
        rea_version,
        notes: vec!["REA discovered; still requires consent + deep profile + budget.".into()],
    }
}

const FIRST_BATCH: &[&str] = &[
    "analyze_web_bundle",
    "trace_javascript_semantics",
    "inspect_web_network_capture",
];

pub fn allowed_operations() -> &'static [&'static str] {
    FIRST_BATCH
}

/// Gate a Deep call. Clean Room always blocks. Consent + deep + budget required.
pub fn request_call(
    session: &mut ReaSession,
    discovery: &ReaDiscovery,
    operation: &str,
    reason: &str,
) -> Result<ReaCallRecord, ToolError> {
    if session.clean_room {
        return Err(ToolError::new(ErrorCode::FeatureMissing, "REA Deep blocked in Clean Room")
            .with_details(json!({"code":"clean_room_blocked"})));
    }
    if !session.consent_granted {
        return Err(ToolError::new(ErrorCode::ConsentRequired, "REA Deep requires explicit consent")
            .with_details(json!({"code":"rea_consent_required"})));
    }
    if !session.deep_profile {
        return Err(ToolError::new(ErrorCode::FeatureMissing, "REA Deep requires deep execution profile")
            .with_details(json!({"code":"deep_required"})));
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
        return Err(ToolError::new(ErrorCode::OverLimit, "REA call budget exhausted")
            .with_details(json!({"code":"rea_budget"})));
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
        return Err(ToolError::new(ErrorCode::NoPermission, "refusing REA finding that contains canary/secret")
            .with_details(json!({"code":"egress_canary"})));
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
