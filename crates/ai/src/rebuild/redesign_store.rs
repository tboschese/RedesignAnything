//! Versioned Redesign Anything workspace store under `<project>/redesign/`.
//!
//! Layout:
//! ```text
//! redesign/
//!   store.json          envelope (schema_version, project_id, revision, execution)
//!   evidence/<id>.json  EvidenceRecord
//!   capabilities/<id>.json
//!   identity.json       IdentityProfile (see redesign_identity)
//! ```
//! Writes are atomic. Paths must stay inside `redesign/`; symlinks out are refused.

use craft_anything_engine::project::write_atomic;
use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::{ErrorCode, ToolError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

pub const DIR: &str = "redesign";
pub const STORE_FILE: &str = "store.json";
pub const SCHEMA_VERSION: &str = "1.0.0";
pub const MAX_RECORD_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreEnvelope {
    pub schema_version: String,
    pub project_id: String,
    pub revision: u32,
    pub generated_at: String,
    #[serde(default)]
    pub execution: ExecutionState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionState {
    pub controller: String,
    pub profile: String,
    pub allow_nested_builder: bool,
    #[serde(default)]
    pub budgets: Value,
    #[serde(default)]
    pub usage: UsageState,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self { controller: "host_agent".into(), profile: "lean".into(), allow_nested_builder: false, budgets: json!({}), usage: UsageState::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageState {
    pub accounting_quality: String,
    pub tool_calls: u64,
    pub input_tokens_reported: Option<u64>,
    pub output_tokens_reported: Option<u64>,
    pub tokens_estimated: Option<u64>,
}

impl Default for UsageState {
    fn default() -> Self {
        Self { accounting_quality: "unavailable".into(), tool_calls: 0, input_tokens_reported: None, output_tokens_reported: None, tokens_estimated: None }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceRecord {
    pub id: String,
    pub source: String,
    pub source_operation: String,
    pub subject: String,
    pub kind: String,
    pub status: String,
    #[serde(default)]
    pub scope: Value,
    #[serde(default)]
    pub locator: Option<Value>,
    #[serde(default)]
    pub digest: Option<String>,
    pub observed_at: String,
    #[serde(default)]
    pub limitations: Vec<String>,
    pub sensitivity: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Capability {
    pub id: String,
    pub title: String,
    pub task: String,
    #[serde(default)]
    pub actor_roles: Vec<String>,
    #[serde(default)]
    pub original_routes: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub outcomes: Vec<String>,
    #[serde(default)]
    pub criticality: String,
    #[serde(default)]
    pub knowledge_status: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub unknowns: Vec<String>,
}

/// `<project>/redesign`
pub fn redesign_dir(ws: &Workspace) -> PathBuf {
    ws.root.join(DIR)
}

fn evidence_dir(root: &Path) -> PathBuf {
    root.join("evidence")
}

fn capabilities_dir(root: &Path) -> PathBuf {
    root.join("capabilities")
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{secs}Z")
}

/// Ensure the store exists; create an empty envelope on first use.
pub fn ensure(ws: &Workspace) -> Result<StoreEnvelope, ToolError> {
    let root = redesign_dir(ws);
    fs::create_dir_all(evidence_dir(&root)).map_err(|e| ToolError::io(root.display(), e))?;
    fs::create_dir_all(capabilities_dir(&root)).map_err(|e| ToolError::io(root.display(), e))?;
    let path = root.join(STORE_FILE);
    if path.is_file() {
        return load_envelope(&path);
    }
    let env = StoreEnvelope {
        schema_version: SCHEMA_VERSION.into(),
        project_id: ws.manifest.name.clone(),
        revision: 1,
        generated_at: now_iso(),
        execution: ExecutionState::default(),
    };
    save_envelope(&path, &env)?;
    Ok(env)
}

fn load_envelope(path: &Path) -> Result<StoreEnvelope, ToolError> {
    let bytes = fs::read(path).map_err(|e| ToolError::io(path.display(), e))?;
    serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("redesign/store.json: {e}")))
}

fn save_envelope(path: &Path, env: &StoreEnvelope) -> Result<(), ToolError> {
    let bytes = serde_json::to_vec_pretty(env).map_err(|e| ToolError::internal(e.to_string()))?;
    write_atomic(path, &bytes)
}

pub(super) fn bump(ws: &Workspace) -> Result<StoreEnvelope, ToolError> {
    let mut env = ensure(ws)?;
    env.revision = env.revision.saturating_add(1);
    env.generated_at = now_iso();
    save_envelope(&redesign_dir(ws).join(STORE_FILE), &env)?;
    Ok(env)
}

pub(super) fn validate_id(prefix: &str, id: &str) -> Result<(), ToolError> {
    let ok = id.starts_with(prefix) && id.len() > prefix.len() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if ok { Ok(()) } else { Err(ToolError::invalid(format!("id {id:?} must match {prefix}<alnum/_/->"))) }
}

/// Resolve `name` under `dir` without leaving it (no `..`, no absolute, no symlink escape).
pub(super) fn safe_child(dir: &Path, name: &str) -> Result<PathBuf, ToolError> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(ToolError::invalid("path components are not allowed in record ids"));
    }
    let path = dir.join(name);
    let dir_real = fs::canonicalize(dir).map_err(|e| ToolError::io(dir.display(), e))?;
    if path.exists() {
        let real = fs::canonicalize(&path).map_err(|e| ToolError::io(path.display(), e))?;
        if !real.starts_with(&dir_real) {
            return Err(ToolError::new(ErrorCode::NoPermission, "refusing path that escapes redesign/").with_details(json!({"code":"path_escape"})));
        }
    }
    Ok(path)
}

pub(super) fn reject_symlink(path: &Path) -> Result<(), ToolError> {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(ToolError::io(path.display(), e)),
    };
    if meta.file_type().is_symlink() {
        return Err(ToolError::new(ErrorCode::NoPermission, "refusing symlink in redesign store").with_details(json!({"code":"symlink_refused"})));
    }
    Ok(())
}

pub(super) fn write_record(path: &Path, value: &impl Serialize) -> Result<(), ToolError> {
    reject_symlink(path)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| ToolError::internal(e.to_string()))?;
    if bytes.len() as u64 > MAX_RECORD_BYTES {
        return Err(ToolError::new(ErrorCode::OverLimit, format!("record exceeds {MAX_RECORD_BYTES} bytes")));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::io(parent.display(), e))?;
    }
    write_atomic(path, &bytes)
}

/// Never persist obvious secrets in free text.
pub(super) fn refuse_secretish(field: &str, text: &str) -> Result<(), ToolError> {
    let lower = text.to_ascii_lowercase();
    for needle in ["password=", "api_key=", "secret=", "Authorization:", "Bearer "] {
        if lower.contains(&needle.to_ascii_lowercase()) {
            return Err(ToolError::invalid(format!("{field} looks like it contains a secret")));
        }
    }
    Ok(())
}

pub fn put_evidence(ws: &Workspace, record: EvidenceRecord) -> Result<StoreEnvelope, ToolError> {
    validate_id("ev_", &record.id)?;
    if record.summary.chars().count() == 0 {
        return Err(ToolError::invalid("evidence summary must not be empty"));
    }
    refuse_secretish("evidence summary", &record.summary)?;
    let root = redesign_dir(ws);
    ensure(ws)?;
    let path = safe_child(&evidence_dir(&root), &format!("{}.json", record.id))?;
    write_record(&path, &record)?;
    bump(ws)
}

pub fn put_capability(ws: &Workspace, cap: Capability) -> Result<StoreEnvelope, ToolError> {
    validate_id("cap_", &cap.id)?;
    if cap.title.trim().is_empty() {
        return Err(ToolError::invalid("capability title must not be empty"));
    }
    let root = redesign_dir(ws);
    ensure(ws)?;
    let path = safe_child(&capabilities_dir(&root), &format!("{}.json", cap.id))?;
    write_record(&path, &cap)?;
    bump(ws)
}

/// Load one capability by id, if present.
pub fn get_capability(ws: &Workspace, id: &str) -> Result<Option<Capability>, ToolError> {
    validate_id("cap_", id)?;
    let path = safe_child(&capabilities_dir(&redesign_dir(ws)), &format!("{id}.json"))?;
    if !path.is_file() {
        return Ok(None);
    }
    reject_symlink(&path)?;
    let bytes = fs::read(&path).map_err(|e| ToolError::io(path.display(), e))?;
    let cap: Capability = serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("{}: {e}", path.display())))?;
    Ok(Some(cap))
}

pub fn list_evidence(ws: &Workspace) -> Result<Vec<EvidenceRecord>, ToolError> {
    let dir = evidence_dir(&redesign_dir(ws));
    list_json_dir(&dir)
}

pub fn list_capabilities(ws: &Workspace) -> Result<Vec<Capability>, ToolError> {
    let dir = capabilities_dir(&redesign_dir(ws));
    list_json_dir(&dir)
}

fn list_json_dir<T: for<'de> Deserialize<'de>>(dir: &Path) -> Result<Vec<T>, ToolError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    reject_symlink(dir)?;
    let mut out = Vec::new();
    for ent in fs::read_dir(dir).map_err(|e| ToolError::io(dir.display(), e))? {
        let ent = ent.map_err(|e| ToolError::io(dir.display(), e))?;
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        reject_symlink(&path)?;
        let bytes = fs::read(&path).map_err(|e| ToolError::io(path.display(), e))?;
        out.push(serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("{}: {e}", path.display())))?);
    }
    Ok(out)
}

pub fn show(ws: &Workspace) -> Result<Value, ToolError> {
    let env = ensure(ws)?;
    Ok(json!({
        "envelope": env,
        "evidence": list_evidence(ws)?,
        "capabilities": list_capabilities(ws)?,
        "path": redesign_dir(ws).display().to_string()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::TargetRef;

    fn ws() -> (tempfile::TempDir, Workspace) {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Demo", "kind=redesign", TargetRef::None).unwrap();
        (d, ws)
    }

    #[test]
    fn roundtrip_evidence_and_capability() {
        let (_d, ws) = ws();
        let env = put_evidence(
            &ws,
            EvidenceRecord {
                id: "ev_form_submit".into(),
                source: "test".into(),
                source_operation: "fixture".into(),
                subject: "expense-form".into(),
                kind: "behavior".into(),
                status: "observed".into(),
                scope: json!({}),
                locator: None,
                digest: None,
                observed_at: "2026-11-20T00:00:00Z".into(),
                limitations: vec!["static fixture".into()],
                sensitivity: "public".into(),
                summary: "Submit enables when amount, category and date are valid.".into(),
            },
        )
        .unwrap();
        assert!(env.revision >= 2);
        put_capability(
            &ws,
            Capability {
                id: "cap_submit_claim".into(),
                title: "Submit expense claim".into(),
                task: "Employee submits a claim".into(),
                actor_roles: vec!["employee".into()],
                original_routes: vec!["/".into()],
                inputs: vec!["amount".into(), "category".into(), "date".into()],
                actions: vec!["submit".into()],
                outcomes: vec!["claim_submitted".into()],
                criticality: "critical".into(),
                knowledge_status: "observed".into(),
                source_refs: vec!["ev_form_submit".into()],
                unknowns: vec!["server duplicate policy".into()],
            },
        )
        .unwrap();
        let ev = list_evidence(&ws).unwrap();
        let caps = list_capabilities(&ws).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(caps[0].id, "cap_submit_claim");
        assert!(caps[0].unknowns.iter().any(|u| u.contains("duplicate")));
    }

    #[test]
    fn rejects_bad_id_and_secretish_summary() {
        let (_d, ws) = ws();
        assert!(
            put_evidence(
                &ws,
                EvidenceRecord {
                    id: "bad".into(),
                    source: "test".into(),
                    source_operation: "x".into(),
                    subject: "s".into(),
                    kind: "ui".into(),
                    status: "observed".into(),
                    scope: json!({}),
                    locator: None,
                    digest: None,
                    observed_at: "t".into(),
                    limitations: vec![],
                    sensitivity: "public".into(),
                    summary: "ok".into(),
                },
            )
            .is_err()
        );
        assert!(
            put_evidence(
                &ws,
                EvidenceRecord {
                    id: "ev_ok".into(),
                    source: "test".into(),
                    source_operation: "x".into(),
                    subject: "s".into(),
                    kind: "ui".into(),
                    status: "observed".into(),
                    scope: json!({}),
                    locator: None,
                    digest: None,
                    observed_at: "t".into(),
                    limitations: vec![],
                    sensitivity: "public".into(),
                    summary: "Authorization: Bearer secret".into(),
                },
            )
            .is_err()
        );
    }

    #[test]
    fn refuses_symlink_escape() {
        let (_d, ws) = ws();
        ensure(&ws).unwrap();
        let root = redesign_dir(&ws);
        let outside = ws.root.join("outside.txt");
        fs::write(&outside, "x").unwrap();
        let link = evidence_dir(&root).join("ev_link.json");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            let err = put_evidence(
                &ws,
                EvidenceRecord {
                    id: "ev_link".into(),
                    source: "test".into(),
                    source_operation: "x".into(),
                    subject: "s".into(),
                    kind: "ui".into(),
                    status: "observed".into(),
                    scope: json!({}),
                    locator: None,
                    digest: None,
                    observed_at: "t".into(),
                    limitations: vec![],
                    sensitivity: "public".into(),
                    summary: "visible rule".into(),
                },
            )
            .unwrap_err();
            let code = err.details.as_ref().and_then(|d| d.get("code").and_then(|c| c.as_str()));
            assert!(err.message.contains("symlink") || err.message.contains("escapes") || matches!(code, Some("symlink_refused" | "path_escape")), "{err:?}");
        }
    }
}
