//! V1 quality runners: oracle scenarios, optional Playwright/axe results.
//! Missing tooling → `not_evaluated` (never fake pass). Runner error/timeout → `fail`.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use craft_anything_platform::process::Process;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::redesign_store;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityCheck {
    pub id: String,
    pub status: String, // pass|fail|warn|not_evaluated|blocked
    pub summary: String,
    #[serde(default)]
    pub details: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioEval {
    pub scenario_id: String,
    pub capability_id: String,
    pub status: String,
    pub notes: Vec<String>,
}

/// Probe whether an external program looks available on PATH (no install).
pub fn tool_available(program: &str) -> bool {
    which_exists(program)
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

/// Run a short probe command; timeout or non-zero → unavailable (not a pass).
pub fn probe_command(program: &str, args: &[&str], timeout: Duration) -> Result<bool, String> {
    if !which_exists(program) {
        return Ok(false);
    }
    match Process::new(program).args(args).timeout(timeout).stdout_cap(4 * 1024).stderr_cap(4 * 1024).run() {
        Ok(out) => Ok(out.success),
        Err(e) => {
            let msg = e.message.to_ascii_lowercase();
            if msg.contains("timeout") || msg.contains("timed out") || msg.contains("was stopped") { Err("timeout".into()) } else { Err(e.message) }
        }
    }
}

/// Evaluate fixture/oracle scenarios against app sources (static, honest).
/// Does not invent backend behaviour; missing hints → fail or not_evaluated.
pub fn evaluate_oracle_file(ws: &Workspace, oracle_path: &Path) -> Result<(QualityCheck, Vec<ScenarioEval>), ToolError> {
    let bytes = fs::read(oracle_path).map_err(|e| ToolError::io(oracle_path.display(), e))?;
    let oracle: Value = serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("oracle json: {e}")))?;
    evaluate_oracle(ws, &oracle)
}

pub fn evaluate_oracle(ws: &Workspace, oracle: &Value) -> Result<(QualityCheck, Vec<ScenarioEval>), ToolError> {
    let app_blob = read_app_blob(ws);
    let mut evals = Vec::new();
    let scenarios = oracle.get("scenarios").and_then(|s| s.as_array()).cloned().unwrap_or_default();
    if scenarios.is_empty() {
        return Ok((
            QualityCheck {
                id: "functional_scenarios".into(),
                status: "not_evaluated".into(),
                summary: "oracle has zero scenarios".into(),
                details: json!({"wcag_certified": false}),
            },
            evals,
        ));
    }

    let mut passed = 0u32;
    let mut failed = 0u32;
    for sc in &scenarios {
        let id = sc.get("id").and_then(|x| x.as_str()).unwrap_or("scenario").to_string();
        let cap = sc.get("capability_id").and_then(|x| x.as_str()).unwrap_or("").to_string();
        let executable = sc.get("executable").and_then(|x| x.as_bool()).unwrap_or(true);
        let thens: Vec<String> =
            sc.get("then").and_then(|t| t.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();

        if !executable {
            evals.push(ScenarioEval {
                scenario_id: id,
                capability_id: cap,
                status: "not_evaluated".into(),
                notes: vec!["scenario marked not executable".into()],
            });
            continue;
        }

        let mut notes = Vec::new();
        let mut ok = true;
        for then in &thens {
            let hint = then_to_hint(then);
            if hint.is_empty() {
                notes.push(format!("no static hint for: {then}"));
                ok = false;
                continue;
            }
            if !app_blob.to_ascii_lowercase().contains(&hint.to_ascii_lowercase()) {
                notes.push(format!("app sources missing hint {hint:?} for: {then}"));
                ok = false;
            } else {
                notes.push(format!("found hint {hint:?}"));
            }
        }
        if ok {
            passed += 1;
            evals.push(ScenarioEval { scenario_id: id, capability_id: cap, status: "pass".into(), notes });
        } else {
            failed += 1;
            evals.push(ScenarioEval { scenario_id: id, capability_id: cap, status: "fail".into(), notes });
        }
    }

    let status = if failed > 0 {
        "fail"
    } else if passed == 0 {
        "not_evaluated"
    } else {
        "pass"
    };
    Ok((
        QualityCheck {
            id: "functional_scenarios".into(),
            status: status.into(),
            summary: format!("scenarios pass={passed} fail={failed} total={}", evals.len()),
            details: json!({ "evals": evals }),
        },
        evals.clone(),
    ))
}

fn then_to_hint(then: &str) -> String {
    let t = then.to_ascii_lowercase();
    // Prefer an explicit quoted phrase when authors put one in the "then" clause.
    if let Some(q) = first_quoted_phrase(then) {
        if q.chars().count() >= 3 {
            return q;
        }
    }
    // Conservative keyword extraction — longer / more specific needles first.
    // Not a full test runner; missing hints stay fail/not_evaluated.
    const NEEDLES: &[&str] = &[
        "save page",
        "wayback",
        "calendar",
        "snapshot",
        "library",
        "download",
        "donate",
        "search",
        "attachment",
        "approve",
        "submit",
        "amount",
        "filter",
        "format",
        "player",
        "login",
        "play",
        "read",
        "role",
    ];
    for needle in NEEDLES {
        if t.contains(needle) {
            return (*needle).into();
        }
    }
    String::new()
}

fn first_quoted_phrase(then: &str) -> Option<String> {
    for (open, close) in [('"', '"'), ('\u{201c}', '\u{201d}'), ('\'', '\'')] {
        if let Some(start) = then.find(open) {
            let rest = &then[start + open.len_utf8()..];
            if let Some(end) = rest.find(close) {
                let inner = rest[..end].trim();
                if !inner.is_empty() {
                    return Some(inner.to_ascii_lowercase());
                }
            }
        }
    }
    None
}

fn read_app_blob(ws: &Workspace) -> String {
    let mut out = String::new();
    let app = ws.app_dir();
    for rel in ["src/App.tsx", "src/main.tsx", "src/mocks/data.ts", "index.html"] {
        if let Ok(s) = fs::read_to_string(app.join(rel)) {
            out.push_str(&s);
            out.push('\n');
        }
    }
    // Also scan routes/
    let routes = app.join("src/routes");
    if routes.is_dir() {
        if let Ok(rd) = fs::read_dir(&routes) {
            for ent in rd.flatten() {
                if let Ok(s) = fs::read_to_string(ent.path()) {
                    out.push_str(&s);
                    out.push('\n');
                }
            }
        }
    }
    out
}

/// Ingest Playwright JSON report if present; else `not_evaluated`.
/// Error marker files → fail (never pass). Does not auto-run npx (avoids silent installs / hangs).
pub fn playwright_check(ws: &Workspace) -> QualityCheck {
    let report_path = redesign_store::redesign_dir(ws).join("quality/playwright-report.json");
    if report_path.is_file() {
        return ingest_runner_report("playwright", &report_path);
    }
    let err_path = redesign_store::redesign_dir(ws).join("quality/playwright-error.json");
    if err_path.is_file() {
        return QualityCheck {
            id: "playwright".into(),
            status: "fail".into(),
            summary: "playwright runner recorded an error (fail-closed)".into(),
            details: json!({"path": err_path.display().to_string()}),
        };
    }
    QualityCheck {
        id: "playwright".into(),
        status: "not_evaluated".into(),
        summary: "no redesign/quality/playwright-report.json; drop a report after an external run".into(),
        details: json!({
            "hint": "write {\"status\":\"pass\",\"failed\":0,\"passed\":N} or use playwright-error.json on failure",
            "npx_on_path": tool_available("npx"),
        }),
    }
}

/// axe-core: ingest report or not_evaluated; never claim WCAG certification.
pub fn axe_check(ws: &Workspace) -> QualityCheck {
    let report_path = redesign_store::redesign_dir(ws).join("quality/axe-report.json");
    if report_path.is_file() {
        let mut q = ingest_runner_report("accessibility", &report_path);
        if let Value::Object(ref mut m) = q.details {
            m.insert("wcag_certified".into(), json!(false));
        }
        q.summary = format!("{}; wcag_certified=false", q.summary);
        return q;
    }
    QualityCheck {
        id: "accessibility".into(),
        status: "not_evaluated".into(),
        summary: "axe report absent; do not claim WCAG certification".into(),
        details: json!({"wcag_certified": false, "hint": "write redesign/quality/axe-report.json after an axe run"}),
    }
}

fn ingest_runner_report(id: &str, path: &Path) -> QualityCheck {
    let Ok(bytes) = fs::read(path) else {
        return QualityCheck { id: id.into(), status: "fail".into(), summary: format!("could not read {}", path.display()), details: json!({}) };
    };
    let Ok(v) = serde_json::from_slice::<Value>(&bytes) else {
        return QualityCheck { id: id.into(), status: "fail".into(), summary: "runner report is not valid JSON (fail-closed)".into(), details: json!({}) };
    };
    // Convention: { "status": "pass"|"fail"|"error"|"timeout", "failed": n, ... }
    let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
    let failed = v.get("failed").and_then(|n| n.as_u64()).unwrap_or(0);
    let mapped = match status {
        "pass" if failed == 0 => "pass",
        "pass" if failed > 0 => "fail",
        "fail" | "error" | "timeout" => "fail",
        "" if failed > 0 => "fail",
        "" if failed == 0 && v.get("passed").and_then(|n| n.as_u64()).unwrap_or(0) > 0 => "pass",
        _ => "fail", // unknown shape → fail closed
    };
    QualityCheck { id: id.into(), status: mapped.into(), summary: format!("ingested {} status={status} failed={failed}", path.display()), details: v }
}

/// Ensure quality dir exists for host agents to drop reports.
pub fn ensure_quality_dir(ws: &Workspace) -> Result<PathBuf, ToolError> {
    redesign_store::ensure(ws)?;
    let dir = redesign_store::redesign_dir(ws).join("quality");
    fs::create_dir_all(&dir).map_err(|e| ToolError::io(dir.display(), e))?;
    Ok(dir)
}

/// Aggregate quality checks for Ready.
pub fn run_quality(ws: &Workspace, oracle_path: Option<&Path>) -> Result<Vec<QualityCheck>, ToolError> {
    ensure_quality_dir(ws)?;
    let mut out = Vec::new();
    if let Some(p) = oracle_path {
        let (qc, _) = evaluate_oracle_file(ws, p)?;
        out.push(qc);
    } else {
        // Look for oracle next to project or default fixture hint in redesign/
        let candidate = redesign_store::redesign_dir(ws).join("oracle/scenarios.json");
        if candidate.is_file() {
            let (qc, _) = evaluate_oracle_file(ws, &candidate)?;
            out.push(qc);
        } else {
            out.push(QualityCheck {
                id: "functional_scenarios".into(),
                status: "not_evaluated".into(),
                summary: "no oracle path; pass --oracle-path or redesign/oracle/scenarios.json".into(),
                details: json!({}),
            });
        }
    }
    out.push(playwright_check(ws));
    out.push(axe_check(ws));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::{TargetRef, Workspace};

    #[test]
    fn zero_scenarios_not_evaluated() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        let (qc, _) = evaluate_oracle(&ws, &json!({"scenarios": []})).unwrap();
        assert_eq!(qc.status, "not_evaluated");
    }

    #[test]
    fn oracle_then_without_app_hint_fails() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        std::fs::create_dir_all(ws.app_dir().join("src")).unwrap();
        std::fs::write(ws.app_dir().join("src/App.tsx"), "export function App(){return null}\n").unwrap();
        let oracle = json!({
            "scenarios": [{
                "id": "s1",
                "capability_id": "cap_x",
                "executable": true,
                "then": ["Approve button is visible"]
            }]
        });
        let (qc, evals) = evaluate_oracle(&ws, &oracle).unwrap();
        assert_eq!(qc.status, "fail");
        assert_eq!(evals[0].status, "fail");
    }

    #[test]
    fn oracle_then_with_hint_passes() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        std::fs::create_dir_all(ws.app_dir().join("src")).unwrap();
        std::fs::write(ws.app_dir().join("src/App.tsx"), "export function App(){ return <button>Approve</button> }\n").unwrap();
        let oracle = json!({
            "scenarios": [{
                "id": "s1",
                "capability_id": "cap_x",
                "executable": true,
                "then": ["Approve button is visible"]
            }]
        });
        let (qc, _) = evaluate_oracle(&ws, &oracle).unwrap();
        assert_eq!(qc.status, "pass", "{qc:?}");
    }

    #[test]
    fn oracle_then_matches_library_and_quoted_phrases() {
        assert_eq!(then_to_hint("Donate control remains available"), "donate");
        assert_eq!(then_to_hint("Calendar day selection is available"), "calendar");
        assert_eq!(then_to_hint("Wayback URL field is separate"), "wayback");
        assert_eq!(then_to_hint("Play/Read is primary"), "play");
        assert_eq!(then_to_hint("Get actions list formats"), "format");
        assert_eq!(then_to_hint("Status text equals \"Workspace ready.\""), "workspace ready.");
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        std::fs::create_dir_all(ws.app_dir().join("src")).unwrap();
        std::fs::write(ws.app_dir().join("src/App.tsx"), "export function App(){ return <button>Donate</button> }\n").unwrap();
        let oracle = json!({
            "scenarios": [{
                "id": "donate",
                "capability_id": "cap_donate",
                "executable": true,
                "then": ["Donate control remains available"]
            }]
        });
        let (qc, _) = evaluate_oracle(&ws, &oracle).unwrap();
        assert_eq!(qc.status, "pass", "{qc:?}");
    }

    #[test]
    fn playwright_error_file_fails_closed() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        let dir = ensure_quality_dir(&ws).unwrap();
        fs::write(dir.join("playwright-error.json"), r#"{"error":"boom"}"#).unwrap();
        let q = playwright_check(&ws);
        assert_eq!(q.status, "fail");
    }

    #[test]
    fn playwright_report_pass_and_fail() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        let dir = ensure_quality_dir(&ws).unwrap();
        fs::write(dir.join("playwright-report.json"), r#"{"status":"pass","failed":0,"passed":2}"#).unwrap();
        assert_eq!(playwright_check(&ws).status, "pass");
        fs::write(dir.join("playwright-report.json"), r#"{"status":"timeout","failed":1}"#).unwrap();
        assert_eq!(playwright_check(&ws).status, "fail");
    }

    #[test]
    fn axe_absent_not_evaluated_never_wcag() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        let q = axe_check(&ws);
        assert_eq!(q.status, "not_evaluated");
        assert_eq!(q.details["wcag_certified"], false);
    }

    #[test]
    fn unknown_report_shape_fails_closed() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Q", "q", TargetRef::None).unwrap();
        let dir = ensure_quality_dir(&ws).unwrap();
        fs::write(dir.join("playwright-report.json"), r#"{"hello":"world"}"#).unwrap();
        assert_eq!(playwright_check(&ws).status, "fail");
    }

    #[test]
    fn probe_timeout_is_error_not_pass() {
        // A command that sleeps longer than the budget must surface as timeout Err, never Ok(true).
        #[cfg(unix)]
        {
            let r = probe_command("sleep", &["2"], Duration::from_millis(200));
            assert!(r.is_err(), "{r:?}");
            assert_eq!(r.unwrap_err(), "timeout");
        }
    }

    #[test]
    fn dense_form_oracle_against_enriched_scaffold() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "Smoke", "smoke", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let ds = repo.join("REDESIGN_SPEC/fixtures/design-system");
        let oracle = repo.join("REDESIGN_SPEC/fixtures/dense-form/oracle/scenarios.json");
        crate::rebuild::redesign_ds::ingest_package(&ws, &ds, "strict", false).unwrap();
        crate::rebuild::redesign_build::scaffold_app(&ws, None).unwrap();
        let app = ws.app_dir().join("src/App.tsx");
        let mut body = fs::read_to_string(&app).unwrap();
        body.push_str("\n/* amount submit Approve role attachments */\n");
        fs::write(&app, body).unwrap();
        let (qc, evals) = evaluate_oracle_file(&ws, &oracle).unwrap();
        assert!(!evals.is_empty());
        assert!(matches!(qc.status.as_str(), "pass" | "fail"), "{qc:?}");
        let ready = crate::rebuild::redesign_checks::run_native_checks(&ws).unwrap();
        assert!(ready.checks.iter().any(|c| c.id == "playwright"));
        assert!(ready.checks.iter().any(|c| c.id == "accessibility" && c.details["wcag_certified"] == false));
    }
}
