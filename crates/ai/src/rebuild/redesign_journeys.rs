//! Journeys: the steps that show a scenario in this prototype, run in the built site by the
//! product's own browser (`web_site_journey_run`).
//!
//! The oracle says what must be true ("then"); a journey says how this prototype is driven to
//! show it, so it belongs to the project, not to the oracle: two variants of one study label
//! their controls differently. A report is only read while the build it ran in is still the one
//! on disk.
//!
//! Limit: the builder writes the steps, so a journey proves that these steps passed, not that
//! they are the right ones for the scenario. The report keeps every step so a person can read it.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::{ToolError, sha256_hex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::redesign_store;

/// The journeys a project keeps, under `redesign/`.
pub const JOURNEYS_FILE: &str = "quality/journeys.json";
/// What the last run found, under `redesign/`.
pub const REPORT_FILE: &str = "quality/journeys-report.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Journey {
    /// The id of a scenario in redesign/oracle/scenarios.json.
    pub scenario_id: String,
    /// The steps, as web_site_journey_run takes them: {"click": name}, {"type": text, "into":
    /// field}, {"press": key}, {"open": path}, {"expect": text}. The last one is an "expect".
    pub steps: Vec<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct JourneyFile {
    journeys: Vec<Journey>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyResult {
    pub scenario_id: String,
    pub passed: bool,
    /// The step that failed, counted from 1.
    #[serde(default)]
    pub failed_at: Option<u64>,
    /// Each step that ran, with whether it passed and what happened.
    #[serde(default)]
    pub steps: Vec<Value>,
    /// Why the journey could not run at all.
    #[serde(default)]
    pub error: Option<String>,
}

impl JourneyResult {
    /// One line for a check's notes.
    pub fn note(&self) -> String {
        if self.passed {
            return format!("journey run in the built prototype: {} step(s) passed", self.steps.len());
        }
        if let Some(why) = &self.error {
            return format!("journey could not run: {why}");
        }
        let at = self.failed_at.unwrap_or(0);
        let why = at.checked_sub(1).and_then(|i| self.steps.get(i as usize)).and_then(|s| s["note"].as_str()).unwrap_or("it failed");
        format!("journey failed at step {at}: {why}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// The built site the journeys ran in ([`built_fingerprint`]).
    pub built: String,
    pub width: u32,
    pub journeys: Vec<JourneyResult>,
}

fn file(ws: &Workspace, name: &str) -> PathBuf {
    redesign_store::redesign_dir(ws).join(name)
}

fn scenario_ids(ws: &Workspace) -> Vec<String> {
    let oracle: Value = fs::read(file(ws, "oracle/scenarios.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    oracle["scenarios"].as_array().into_iter().flatten().filter_map(|s| s["id"].as_str().map(str::to_string)).collect()
}

/// The scenarios that owe a journey, with what each must show: every executable scenario of the
/// oracle that is not retired by an approved decision (`redesign_scenario_retire`).
pub fn owed(ws: &Workspace) -> Vec<(String, String)> {
    let oracle: Value = fs::read(file(ws, "oracle/scenarios.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
    let (retired, _) = super::redesign_retired::in_force(ws);
    oracle["scenarios"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|s| s["executable"].as_bool().unwrap_or(true))
        .filter_map(|s| {
            let id = s["id"].as_str()?.to_string();
            let then: Vec<&str> = s["then"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
            (!retired.contains_key(&id)).then(|| (id, then.join("; ")))
        })
        .collect()
}

/// Refuse journeys that name no scenario of the oracle, repeat one, or check nothing at the end.
pub fn check(ws: &Workspace, journeys: &[Journey]) -> Result<(), ToolError> {
    let known = scenario_ids(ws);
    let mut seen = Vec::new();
    for j in journeys {
        if !known.contains(&j.scenario_id) {
            return Err(ToolError::invalid(format!("no scenario {:?} in redesign/oracle/scenarios.json", j.scenario_id)).with_remediation(format!(
                "A journey shows one of the oracle's scenarios. Their ids: {}.",
                if known.is_empty() { "none yet".into() } else { known.join(", ") }
            )));
        }
        if seen.contains(&&j.scenario_id) {
            return Err(ToolError::invalid(format!("two journeys for scenario {:?}: keep one", j.scenario_id)));
        }
        seen.push(&j.scenario_id);
        if j.steps.last().is_none_or(|last| !last["expect"].is_string()) {
            return Err(ToolError::invalid(format!(
                "the journey for {:?} must end with an {{\"expect\": text}} step, so it checks what the steps before it led to",
                j.scenario_id
            )));
        }
    }
    Ok(())
}

pub fn save(ws: &Workspace, journeys: &[Journey]) -> Result<(), ToolError> {
    check(ws, journeys)?;
    super::redesign_quality::ensure_quality_dir(ws)?;
    let bytes = serde_json::to_vec_pretty(&JourneyFile { journeys: journeys.to_vec() }).map_err(|e| ToolError::internal(e.to_string()))?;
    craft_anything_engine::project::write_atomic(&file(ws, JOURNEYS_FILE), &bytes)
}

/// The journeys the project keeps; none when it keeps no file.
/// The journeys a builder writes, inside the app it may change: the builder changes nothing
/// outside app/, so its journeys can't go under redesign/.
pub const APP_JOURNEYS_FILE: &str = "redesign-journeys.json";

/// Whichever of two files was written last, among those that exist.
pub(super) fn newest(candidates: [PathBuf; 2]) -> Option<PathBuf> {
    candidates.into_iter().filter(|p| p.is_file()).max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
}

/// The journeys the project keeps: the newer of those `redesign_journeys_run` was given
/// (redesign/quality/journeys.json) and those the builder wrote (app/redesign-journeys.json).
pub fn load(ws: &Workspace) -> Result<Vec<Journey>, ToolError> {
    let Some(path) = newest([file(ws, JOURNEYS_FILE), ws.app_dir().join(APP_JOURNEYS_FILE)]) else { return Ok(Vec::new()) };
    let bytes = fs::read(&path).map_err(|e| ToolError::io(path.display(), e))?;
    let shown = path.strip_prefix(&ws.root).unwrap_or(&path).display().to_string();
    let kept: JourneyFile = serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("{shown}: {e}")))?;
    Ok(kept.journeys)
}

/// What stands for the built site on disk: every file's place and content. `None` before a build.
pub fn built_fingerprint(ws: &Workspace) -> Option<String> {
    let site = super::site_dir(&ws.app_dir())?;
    let mut files = Vec::new();
    super::redesign_checks::files_under(&site, 0, &mut files);
    files.sort();
    let mut all = String::new();
    for f in &files {
        let content = fs::read(f).ok()?;
        all.push_str(&format!("{}\0{}\n", f.strip_prefix(&site).unwrap_or(f).display(), sha256_hex(&content)));
    }
    Some(sha256_hex(all.as_bytes()))
}

pub fn write_report(ws: &Workspace, report: &Report) -> Result<PathBuf, ToolError> {
    super::redesign_quality::ensure_quality_dir(ws)?;
    let path = file(ws, REPORT_FILE);
    craft_anything_engine::project::write_atomic(&path, &serde_json::to_vec_pretty(report).map_err(|e| ToolError::internal(e.to_string()))?)?;
    Ok(path)
}

/// The last run's results by scenario, when they are about the build on disk; else none, and why
/// a report that exists was not used.
pub fn fresh_results(ws: &Workspace) -> (BTreeMap<String, JourneyResult>, Option<String>) {
    let path = file(ws, REPORT_FILE);
    if !path.is_file() {
        return (BTreeMap::new(), None);
    }
    let unread = |why: &str| (BTreeMap::new(), Some(format!("redesign/{REPORT_FILE} was not used: {why}; run redesign_journeys_run again")));
    let Some(report) = fs::read(&path).ok().and_then(|b| serde_json::from_slice::<Report>(&b).ok()) else { return unread("it could not be read") };
    match built_fingerprint(ws) {
        Some(now) if now == report.built => (report.journeys.into_iter().map(|j| (j.scenario_id.clone(), j)).collect(), None),
        Some(_) => unread("the prototype was built again since it was written"),
        None => unread("there is no built prototype"),
    }
}

/// A journey's result from what `web_site_journey_run` answered, or from why it could not run.
pub fn result_of(scenario_id: &str, ran: Result<Value, ToolError>) -> JourneyResult {
    match ran {
        Ok(v) => JourneyResult {
            scenario_id: scenario_id.into(),
            passed: v["passed"] == true,
            failed_at: v["failed_at"].as_u64(),
            steps: v["steps"].as_array().cloned().unwrap_or_default(),
            error: None,
        },
        Err(e) => JourneyResult { scenario_id: scenario_id.into(), passed: false, failed_at: None, steps: Vec::new(), error: Some(e.message) },
    }
}

/// The folder a journey's end picture goes in, removed first so the browser tool can make it.
pub fn fresh_picture_dir(ws: &Workspace, scenario_id: &str) -> Result<PathBuf, ToolError> {
    let name: String = scenario_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    let dir = file(ws, "quality/journeys").join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| ToolError::io(dir.display(), e))?;
    }
    fs::create_dir_all(dir.parent().unwrap_or(Path::new("."))).map_err(|e| ToolError::io(dir.display(), e))?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::TargetRef;
    use serde_json::json;

    fn project_with_scenarios(ids: &[&str]) -> (tempfile::TempDir, Workspace) {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "J", "kind=redesign", TargetRef::None).unwrap();
        let oracle = file(&ws, "oracle");
        fs::create_dir_all(&oracle).unwrap();
        let scenarios: Vec<Value> = ids.iter().map(|id| json!({"id": id, "capability_id": "cap", "then": ["Search works"], "executable": true})).collect();
        fs::write(oracle.join("scenarios.json"), json!({"scenarios": scenarios}).to_string()).unwrap();
        (d, ws)
    }

    fn journey(id: &str, steps: Value) -> Journey {
        Journey { scenario_id: id.into(), steps: steps.as_array().unwrap().clone() }
    }

    #[test]
    fn a_journey_shows_a_scenario_of_the_oracle_and_ends_by_checking_something() {
        let (_d, ws) = project_with_scenarios(&["search", "donate"]);
        let good = journey("search", json!([{"click": "Search"}, {"expect": "Results"}]));
        assert!(save(&ws, std::slice::from_ref(&good)).is_ok());
        assert_eq!(load(&ws).unwrap(), vec![good.clone()]);
        assert!(check(&ws, &[journey("unknown", json!([{"expect": "Results"}]))]).is_err(), "not a scenario of the oracle");
        assert!(check(&ws, &[journey("search", json!([{"expect": "Results"}, {"click": "Search"}]))]).is_err(), "nothing checked after the last click");
        assert!(check(&ws, &[journey("search", json!([]))]).is_err());
        assert!(check(&ws, &[good.clone(), good]).is_err(), "one journey per scenario");
    }

    /// A builder writes its journeys in the app it may change; the newer of its file and the
    /// tool's is the one kept.
    #[test]
    fn the_newer_of_the_builders_journeys_and_the_tools_is_kept() {
        let (_d, ws) = project_with_scenarios(&["search"]);
        let given = journey("search", json!([{"click": "Search"}, {"expect": "Results"}]));
        save(&ws, std::slice::from_ref(&given)).unwrap();
        let written = journey("search", json!([{"press": "Enter"}, {"expect": "Results"}]));
        fs::create_dir_all(ws.app_dir()).unwrap();
        let app_file = ws.app_dir().join(APP_JOURNEYS_FILE);
        fs::write(&app_file, json!({"journeys": [written.clone()]}).to_string()).unwrap();
        let later = fs::metadata(file(&ws, JOURNEYS_FILE)).unwrap().modified().unwrap() + std::time::Duration::from_secs(5);
        fs::File::options().write(true).open(&app_file).unwrap().set_modified(later).unwrap();
        assert_eq!(load(&ws).unwrap(), vec![written]);
        save(&ws, std::slice::from_ref(&given)).unwrap();
        let newer = later + std::time::Duration::from_secs(5);
        fs::File::options().write(true).open(file(&ws, JOURNEYS_FILE)).unwrap().set_modified(newer).unwrap();
        assert_eq!(load(&ws).unwrap(), vec![given], "given to the tool after: the tool's");
    }

    #[test]
    fn a_report_is_read_only_while_the_build_it_ran_in_is_on_disk() {
        let (_d, ws) = project_with_scenarios(&["search"]);
        assert!(built_fingerprint(&ws).is_none(), "nothing built yet");
        let dist = ws.app_dir().join("dist");
        fs::create_dir_all(dist.join("assets")).unwrap();
        fs::write(dist.join("index.html"), "<script src=assets/app.js></script>").unwrap();
        fs::write(dist.join("assets/app.js"), "let a = 1;").unwrap();
        assert!(fresh_results(&ws).0.is_empty() && fresh_results(&ws).1.is_none(), "no report, nothing to say");

        let ran = result_of("search", Ok(json!({"passed": true, "steps": [{"step": {"expect": "Results"}, "ok": true, "note": "\"Results\" shows"}]})));
        write_report(&ws, &Report { built: built_fingerprint(&ws).unwrap(), width: 1440, journeys: vec![ran.clone()] }).unwrap();
        assert_eq!(fresh_results(&ws).0.get("search"), Some(&ran));

        // The same page with another script is another build: what ran before says nothing about it.
        fs::write(dist.join("assets/app.js"), "let a = 2;").unwrap();
        let (results, why) = fresh_results(&ws);
        assert!(results.is_empty(), "{results:?}");
        assert!(why.is_some_and(|w| w.contains("built again")));
    }

    #[test]
    fn a_result_says_which_step_failed_or_why_nothing_ran() {
        let failed = result_of(
            "s",
            Ok(
                json!({"passed": false, "failed_at": 2, "steps": [{"ok": true, "note": "clicked"}, {"ok": false, "note": "\"Results\" did not show within 3000 ms"}]}),
            ),
        );
        assert!(!failed.passed && failed.note().contains("step 2") && failed.note().contains("did not show"), "{}", failed.note());
        let broken = result_of("s", Err(ToolError::invalid("no website is open")));
        assert!(!broken.passed && broken.note().contains("no website is open"), "{}", broken.note());
    }
}
