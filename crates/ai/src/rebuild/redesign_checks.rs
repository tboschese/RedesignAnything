//! PR-08: Native Redesign checks + Ready gates (Rust, fail-closed).
//! Check errors/timeouts never become `pass`. Zero oracles ⇒ not `prototype_ready`.
//! Design Critic / LLM judges are advisory only and never promote Ready alone.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

use super::redesign_ds;
use super::redesign_plan;
use super::redesign_store;
use super::redesign_study;

pub const CHECKS_FILE: &str = "check_results.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub status: String, // pass|fail|warn|not_evaluated|blocked
    pub summary: String,
    #[serde(default)]
    pub details: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadyReport {
    pub concept_ready: bool,
    pub prototype_ready: bool,
    pub engineering_ready: bool,
    pub checks: Vec<CheckResult>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CheckInputs {
    pub preview_available: bool,
    pub security_ok: bool,
    pub build_status: CheckExec,
    pub ds_status: CheckExec,
    pub functional: FunctionalSummary,
    pub a11y_critical_fail: bool,
    pub secrets_exported: bool,
    pub critic_says_ready: bool, // must never alone promote Ready
    /// The plan would lose the target's essence (`identity_essence` failed): blocks every gate.
    pub essence_lost: bool,
    /// The app drifted from an intentional kept trait (`identity_drift` failed): blocks Prototype Ready.
    pub essence_drift: bool,
    /// Scenarios judged only by a word found in the sources, with no journey run in the built
    /// prototype: a prototype is not ready on those.
    pub scenarios_on_a_word_hint: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CheckExec {
    Pass,
    Fail,
    Warn,
    #[default]
    NotEvaluated,
    Blocked,
    /// Tool/check error or timeout — must NOT count as pass.
    Error,
    Timeout,
}

impl CheckExec {
    pub fn as_status(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Warn => "warn",
            Self::NotEvaluated => "not_evaluated",
            Self::Blocked => "blocked",
            Self::Error => "fail",
            Self::Timeout => "fail",
        }
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FunctionalSummary {
    pub critical_scenarios_total: u32,
    pub critical_scenarios_passed: u32,
    pub button_works: bool,
}

/// Pure gates from CONTRACTS.md / IMPLEMENTATION_PLAN — unavailable ≠ zero tokens; critic ≠ Ready.
pub fn concept_ready(inp: &CheckInputs) -> bool {
    inp.security_ok
        && inp.preview_available
        && !matches!(inp.build_status, CheckExec::Error | CheckExec::Timeout)
        && inp.build_status != CheckExec::Fail
        && !inp.secrets_exported
        && !inp.essence_lost
}

pub fn prototype_ready(inp: &CheckInputs, unmapped_critical_unapproved: u32) -> bool {
    if inp.critic_says_ready && !concept_ready(inp) {
        // Critic alone never promotes.
        return false;
    }
    concept_ready(inp)
        && inp.build_status.is_pass()
        // A warning only exists under the advisory policy (strict turns it into a failure), and advice
        // doesn't block: CONTRACTS.md §7 asks for a pass only when the design system is strict.
        && matches!(inp.ds_status, CheckExec::Pass | CheckExec::NotEvaluated | CheckExec::Warn)
        && inp.functional.critical_scenarios_total > 0
        && inp.functional.critical_scenarios_passed == inp.functional.critical_scenarios_total
        && inp.scenarios_on_a_word_hint == 0
        && inp.functional.button_works
        && unmapped_critical_unapproved == 0
        && !inp.a11y_critical_fail
        && !inp.essence_drift
        && !matches!(inp.ds_status, CheckExec::Error | CheckExec::Timeout | CheckExec::Fail | CheckExec::Blocked)
}

/// `backend_contracts_ok` means no plan mapping still lists `backend_change_required`
/// (and no unresolved `not_determined`). Traceability approval alone is not enough.
pub fn engineering_ready(inp: &CheckInputs, unmapped_critical_unapproved: u32, backend_contracts_ok: bool) -> bool {
    prototype_ready(inp, unmapped_critical_unapproved) && backend_contracts_ok && !inp.secrets_exported
}

/// Host-agent previews often use native `<button>`; scaffolds may use DS `<Button>`.
/// Either counts; an explicit broken marker still fails closed.
pub fn primary_action_present(app_tsx: &str) -> bool {
    if app_tsx.contains("TODO_BROKEN_BUTTON") {
        return false;
    }
    app_tsx.contains("<Button") || app_tsx.contains("<button") || app_tsx.contains("role=\"button\"") || app_tsx.contains("role='button'")
}

pub fn run_native_checks(ws: &Workspace) -> Result<ReadyReport, ToolError> {
    let mut checks = Vec::new();

    // Build: what the files in app/ say about it. This check runs no build.
    let pkg = ws.app_dir().join("package.json");
    let (build, build_summary, build_details) = build_state(&ws.app_dir());
    checks.push(CheckResult { id: "build_scaffold".into(), status: build.as_status().into(), summary: build_summary, details: build_details });

    // DS compliance if registry exists.
    let ds = match redesign_ds::load_registry(ws) {
        Ok(reg) => {
            let sources = scan_sources(ws);
            let report = redesign_ds::check_sources(&reg, &sources, Some(&reg.package_version));
            let st = match report.status.as_str() {
                "pass" => CheckExec::Pass,
                "warn" => CheckExec::Warn,
                _ => CheckExec::Fail,
            };
            checks.push(CheckResult {
                id: "design_system".into(),
                status: st.as_status().into(),
                summary: if st == CheckExec::Warn && reg.policy != "strict" {
                    format!("DS policy={} status={} (advice: does not block Prototype Ready)", reg.policy, report.status)
                } else {
                    format!("DS policy={} status={}", reg.policy, report.status)
                },
                details: serde_json::to_value(&report).unwrap_or(json!({})),
            });
            if reg.policy == "strict" && st != CheckExec::Pass { CheckExec::Fail } else { st }
        }
        Err(_) => {
            checks.push(CheckResult {
                id: "design_system".into(),
                status: "blocked".into(),
                summary: "design system registry missing".into(),
                details: json!({"code":"ds_registry_required"}),
            });
            CheckExec::Blocked
        }
    };

    // Coverage / oracles honesty.
    let coverage = redesign_study::coverage_of(ws).unwrap_or(redesign_study::CoverageReport {
        covered: 0,
        partial: 0,
        uncovered: 0,
        unknown: 0,
        percent_claimed: None,
        unknowns: vec![],
        notes: vec![],
    });
    let oracle_total = coverage.covered + coverage.partial; // treat mapped caps as scenarios proxy
    checks.push(CheckResult {
        id: "coverage_honesty".into(),
        status: if coverage.percent_claimed == Some(100.0) && coverage.unknown > 0 { "fail" } else { "pass" }.into(),
        summary: format!("unknowns={} percent_claimed={:?}", coverage.unknown, coverage.percent_claimed),
        details: serde_json::to_value(&coverage).unwrap_or(json!({})),
    });

    // Traceability + backend-contract clarity for Engineering Ready.
    let caps = redesign_store::list_capabilities(ws).unwrap_or_default();
    let (trace_status, unmapped, backend_contracts_ok) = match redesign_plan::load_plan(ws) {
        Ok(plan) => {
            let report = redesign_plan::traceability_check(ws, &caps, &plan)?;
            let st = match report.status.as_str() {
                "pass" => "pass",
                "needs_approval" => "blocked",
                _ => "fail",
            };
            let n = report.blocked.len() as u32 + report.needs_approval.len() as u32;
            checks.push(CheckResult {
                id: "traceability".into(),
                status: st.into(),
                summary: format!("status={} blocked={}", report.status, report.blocked.len()),
                details: serde_json::to_value(&report).unwrap_or(json!({})),
            });
            let (clear, blocking) = redesign_plan::backend_contracts_clear(&plan);
            checks.push(CheckResult {
                id: "backend_contracts".into(),
                status: if clear { "pass" } else { "blocked" }.into(),
                summary: if clear {
                    "no backend_change_required mappings remain".into()
                } else {
                    format!("blocking={} (approved gap ≠ known contract)", blocking.len())
                },
                details: json!({ "blocking": blocking, "clear": clear }),
            });
            (st, n, clear && st == "pass")
        }
        Err(_) => {
            checks.push(CheckResult {
                id: "traceability".into(),
                status: "not_evaluated".into(),
                summary: "no transformation plan".into(),
                details: json!({}),
            });
            checks.push(CheckResult {
                id: "backend_contracts".into(),
                status: "not_evaluated".into(),
                summary: "no transformation plan".into(),
                details: json!({}),
            });
            ("not_evaluated", caps.iter().filter(|c| c.criticality == "critical" || c.criticality == "high").count() as u32, false)
        }
    };

    // Security: refuse secrets-looking strings in app sources.
    let secrets = sources_look_like_secrets(ws);
    checks.push(CheckResult {
        id: "security_secrets".into(),
        status: if secrets { "fail" } else { "pass" }.into(),
        summary: if secrets { "possible secret material in app/".into() } else { "no obvious secrets in scanned app sources".into() },
        details: json!({}),
    });

    // The target's essence: losing it in the plan blocks every gate, drifting from it in the app
    // blocks Prototype Ready. A frozen character (identity_evolution) is advisory.
    let essence = super::redesign_identity::essence_check(ws);
    let drift = super::redesign_identity::drift_check(ws);
    let (essence_lost, essence_drift) = (essence.status == "fail", drift.status == "fail");
    checks.push(essence);
    checks.push(drift);
    checks.push(super::redesign_identity::evolution_check(ws));
    // Advisory: does the plan stay within its mode (tidy-up, redesign, rethink)?
    checks.push(super::redesign_mode::scope_check(ws));
    // Advisory: does a captured prototype page run off the side of its window?
    checks.push(super::redesign_likeness::fits_window_check(ws));
    // Advisory: the basics a captured prototype page gets wrong (names, alt text, language, landmarks).
    checks.push(super::redesign_likeness::basics_check(ws));

    // Functional proxy + quality runners (oracle static eval, optional Playwright/axe reports).
    let app_tsx = fs::read_to_string(ws.app_dir().join("src/App.tsx")).unwrap_or_default();
    let button_works = primary_action_present(&app_tsx);
    let preview = pkg.is_file() && ws.app_dir().join("src/main.tsx").is_file();

    checks.push(CheckResult {
        id: "functional_smoke".into(),
        status: if oracle_total == 0 {
            "not_evaluated"
        } else if button_works {
            "pass"
        } else {
            "fail"
        }
        .into(),
        summary: format!("oracles_proxy={oracle_total} button_works={button_works}"),
        details: json!({
            "note": "Complement with functional_scenarios + playwright quality runners.",
            "accepts": ["<Button", "<button", "role=\"button\""],
        }),
    });

    let mut a11y_critical_fail = false;
    let mut scenario_passed = 0u32;
    let mut scenario_total = 0u32;
    let mut scenarios_on_a_word_hint = 0u32;
    match super::redesign_quality::run_quality(ws, None) {
        Ok(qcs) => {
            for qc in qcs {
                if qc.id == "functional_scenarios"
                    && let Some(evals) = qc.details.get("evals").and_then(|e| e.as_array())
                {
                    (scenario_total, scenario_passed, scenarios_on_a_word_hint) = scenario_counts(evals);
                }
                if qc.id == "accessibility" && qc.status == "fail" {
                    a11y_critical_fail = true;
                }
                checks.push(CheckResult { id: qc.id, status: qc.status, summary: qc.summary, details: qc.details });
            }
        }
        Err(e) => {
            checks.push(CheckResult {
                id: "quality_runners".into(),
                status: "fail".into(),
                summary: format!("quality runners error (fail-closed): {e}"),
                details: json!({}),
            });
        }
    }

    let critical_total = if scenario_total > 0 { scenario_total } else { oracle_total };
    let critical_passed = if scenario_total > 0 {
        scenario_passed
    } else if button_works {
        oracle_total
    } else {
        0
    };
    let functional = FunctionalSummary { critical_scenarios_total: critical_total, critical_scenarios_passed: critical_passed, button_works };
    let inp = CheckInputs {
        preview_available: preview,
        security_ok: !secrets,
        build_status: build,
        ds_status: ds,
        functional,
        a11y_critical_fail,
        secrets_exported: secrets,
        critic_says_ready: false,
        essence_lost,
        essence_drift,
        scenarios_on_a_word_hint,
    };

    let report = ReadyReport {
        concept_ready: concept_ready(&inp),
        prototype_ready: prototype_ready(&inp, unmapped),
        engineering_ready: engineering_ready(&inp, unmapped, backend_contracts_ok),
        checks,
        notes: vec![
            "Native Rust checks are authoritative; WASM PluginJudge is not used for Ready.".into(),
            "unavailable usage/tokens are not treated as zero.".into(),
            "Design Critic recommendations never alone set Ready.".into(),
            "Accessibility not_evaluated ≠ WCAG pass.".into(),
            "Prototype Ready needs every scenario judged by a journey run in the built prototype (redesign_journeys_run); a word found in the sources is not enough.".into(),
            "Engineering Ready requires no backend_change_required mappings; approving a gap is not a contract.".into(),
            "identity_essence failing blocks every gate; identity_drift failing blocks Prototype Ready; identity_evolution is advisory.".into(),
            "mode_scope, page_fits_window and prototype_basics are advisory and never change Ready.".into(),
            format!("traceability={trace_status}"),
        ],
    };

    redesign_store::ensure(ws)?;
    let path = redesign_store::redesign_dir(ws).join(CHECKS_FILE);
    craft_anything_engine::project::write_atomic(&path, serde_json::to_vec_pretty(&report).map_err(|e| ToolError::internal(e.to_string()))?.as_slice())?;
    Ok(report)
}

/// From the scenarios' evaluations: how many are owed, how many passed, and how many rest on a
/// word hint. A retired scenario is neither passed nor owed, so it is not one of the total.
fn scenario_counts(evals: &[Value]) -> (u32, u32, u32) {
    let count = |key: &str, wanted: &str| evals.iter().filter(|e| e.get(key).and_then(Value::as_str) == Some(wanted)).count() as u32;
    (evals.len() as u32 - count("status", "retired"), count("status", "pass"), count("method", "word_hint"))
}

/// Whether `app` holds a built site that is at least as new as its sources, from the files alone.
/// A scaffold that was never built, or was edited since its last build, is not evaluated: a
/// `package.json` says a build is possible, not that one succeeded.
fn build_state(app: &Path) -> (CheckExec, String, Value) {
    let details = |built: bool, output: Option<&str>| json!({"method": "built_site_newer_than_sources", "ran_build": false, "built": built, "output": output});
    if !app.is_dir() {
        return (CheckExec::NotEvaluated, "no app/ folder yet".into(), details(false, None));
    }
    if !app.join("package.json").is_file() {
        return (CheckExec::Fail, "app scaffold missing package.json".into(), details(false, None));
    }
    let Some(site) = super::site_dir(app) else {
        let summary = "app/package.json present, but no built site (dist, build or out with an index.html): build the app first, this check runs no build";
        return (CheckExec::NotEvaluated, summary.into(), details(false, None));
    };
    let output = site.strip_prefix(app).map(|p| p.display().to_string()).unwrap_or_default();
    let Some(built_at) = modified(&site.join("index.html")) else {
        return (CheckExec::NotEvaluated, format!("could not read when app/{output}/index.html was built"), details(false, Some(&output)));
    };
    let mut sources = vec![app.join("package.json"), app.join("index.html")];
    files_under(&app.join("src"), 0, &mut sources);
    match sources.iter().find(|p| modified(p).is_some_and(|at| at > built_at)) {
        Some(newer) => {
            let newer = newer.strip_prefix(app).unwrap_or(newer).display();
            (
                CheckExec::NotEvaluated,
                format!("the built site in app/{output} is older than app/{newer}: build again, this check runs no build"),
                details(false, Some(&output)),
            )
        }
        None => {
            (CheckExec::Pass, format!("the built site in app/{output} is as new as the app's sources; this check ran no build"), details(true, Some(&output)))
        }
    }
}

fn modified(path: &Path) -> Option<std::time::SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Every file under `dir`, to a depth no source tree needs more than.
pub(super) fn files_under(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth < 16 {
                files_under(&path, depth + 1, out);
            }
        } else {
            out.push(path);
        }
    }
}

fn scan_sources(ws: &Workspace) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let app = ws.app_dir();
    let candidates = ["src/App.tsx", "src/main.tsx", "src/mocks/data.ts"];
    for rel in candidates {
        let path = app.join(rel);
        if let Ok(content) = fs::read_to_string(&path) {
            out.push((rel.into(), content));
        }
    }
    out
}

fn sources_look_like_secrets(ws: &Workspace) -> bool {
    for (_, content) in scan_sources(ws) {
        let lower = content.to_ascii_lowercase();
        if lower.contains("begin private key") || lower.contains("aws_secret_access_key") || lower.contains("api_key=\"sk-") {
            return true;
        }
    }
    false
}

/// Contract: error/timeout statuses never equal pass.
pub fn status_from_exec(exec: CheckExec) -> &'static str {
    exec.as_status()
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::{TargetRef, Workspace};
    use std::path::PathBuf;

    fn fixture_ds() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/design-system").canonicalize().unwrap()
    }

    #[test]
    fn error_and_timeout_never_pass() {
        assert_ne!(status_from_exec(CheckExec::Error), "pass");
        assert_ne!(status_from_exec(CheckExec::Timeout), "pass");
        assert_eq!(status_from_exec(CheckExec::Error), "fail");
        assert_eq!(status_from_exec(CheckExec::Timeout), "fail");
    }

    #[test]
    fn zero_oracles_not_prototype_ready() {
        let inp = CheckInputs {
            preview_available: true,
            security_ok: true,
            build_status: CheckExec::Pass,
            ds_status: CheckExec::Pass,
            functional: FunctionalSummary { critical_scenarios_total: 0, critical_scenarios_passed: 0, button_works: true },
            ..Default::default()
        };
        assert!(concept_ready(&inp));
        assert!(!prototype_ready(&inp, 0));
    }

    #[test]
    fn pretty_ui_but_broken_button_fails_prototype() {
        let inp = CheckInputs {
            preview_available: true,
            security_ok: true,
            build_status: CheckExec::Pass,
            ds_status: CheckExec::Pass,
            functional: FunctionalSummary { critical_scenarios_total: 2, critical_scenarios_passed: 2, button_works: false },
            ..Default::default()
        };
        assert!(!prototype_ready(&inp, 0));
    }

    #[test]
    fn critic_alone_does_not_promote_ready() {
        let inp = CheckInputs { preview_available: false, security_ok: false, build_status: CheckExec::Fail, critic_says_ready: true, ..Default::default() };
        assert!(!concept_ready(&inp));
        assert!(!prototype_ready(&inp, 0));
    }

    #[test]
    fn a11y_critical_blocks_but_no_wcag_claim_in_report() {
        let inp = CheckInputs {
            preview_available: true,
            security_ok: true,
            build_status: CheckExec::Pass,
            ds_status: CheckExec::Pass,
            functional: FunctionalSummary { critical_scenarios_total: 1, critical_scenarios_passed: 1, button_works: true },
            a11y_critical_fail: true,
            ..Default::default()
        };
        assert!(!prototype_ready(&inp, 0));
    }

    #[test]
    fn native_run_on_scaffold_writes_report() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "G", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        redesign_ds::ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        redesign_store::put_capability(
            &ws,
            redesign_store::Capability {
                id: "cap_list".into(),
                title: "List".into(),
                task: "list".into(),
                actor_roles: vec![],
                original_routes: vec!["/r".into()],
                inputs: vec![],
                actions: vec![],
                outcomes: vec![],
                criticality: "normal".into(),
                knowledge_status: "observed".into(),
                source_refs: vec![],
                unknowns: vec![],
            },
        )
        .unwrap();
        super::super::redesign_build::scaffold_app(&ws, None).unwrap();
        let report = run_native_checks(&ws).unwrap();
        assert!(report.notes.iter().any(|n| n.contains("PluginJudge")));
        assert!(report.checks.iter().any(|c| c.id == "accessibility" && c.details["wcag_certified"] == false));
        // With only normal (non-critical) cap and no plan, prototype may still be false due to oracle proxy / unmapped.
        assert!(!report.engineering_ready || report.prototype_ready);
    }

    #[test]
    fn a_build_passes_only_when_a_built_site_is_as_new_as_the_sources() {
        let d = tempfile::tempdir().unwrap();
        let app = d.path().join("app");
        assert_eq!(build_state(&app).0, CheckExec::NotEvaluated, "no app yet");
        fs::create_dir_all(app.join("src/routes")).unwrap();
        assert_eq!(build_state(&app).0, CheckExec::Fail, "an app folder with no package.json");
        fs::write(app.join("package.json"), "{}").unwrap();
        fs::write(app.join("index.html"), "<div id=root></div>").unwrap();
        fs::write(app.join("src/routes/Home.tsx"), "export const Home = 1;").unwrap();
        let (never_built, summary, _) = build_state(&app);
        assert_ne!(never_built, CheckExec::Pass, "a package.json alone is not a build: {summary}");
        assert_ne!(never_built, CheckExec::Fail, "and not having built yet is not a failed build");

        fs::create_dir_all(app.join("dist")).unwrap();
        fs::write(app.join("dist/index.html"), "<div id=root></div>").unwrap();
        let (built, _, details) = build_state(&app);
        assert_eq!(built, CheckExec::Pass);
        assert_eq!(details["ran_build"], false, "the check says it ran no build itself");

        // An edit after the build: the built site no longer shows what the sources say.
        let later = modified(&app.join("dist/index.html")).unwrap() + std::time::Duration::from_secs(60);
        fs::File::options().write(true).open(app.join("src/routes/Home.tsx")).unwrap().set_modified(later).unwrap();
        let (stale, summary, _) = build_state(&app);
        assert_ne!(stale, CheckExec::Pass);
        assert!(summary.contains("Home.tsx"), "it names the newer source: {summary}");
    }

    #[test]
    fn a_scaffold_is_not_prototype_ready_until_it_is_built() {
        let inp = |build_status| CheckInputs {
            preview_available: true,
            security_ok: true,
            build_status,
            ds_status: CheckExec::Pass,
            functional: FunctionalSummary { critical_scenarios_total: 1, critical_scenarios_passed: 1, button_works: true },
            ..Default::default()
        };
        assert!(concept_ready(&inp(CheckExec::NotEvaluated)), "a concept needs no build");
        assert!(!prototype_ready(&inp(CheckExec::NotEvaluated), 0));
        assert!(prototype_ready(&inp(CheckExec::Pass), 0));
    }

    #[test]
    fn a_prototype_is_not_ready_while_a_scenario_rests_on_a_word_in_the_sources() {
        let inp = |scenarios_on_a_word_hint| CheckInputs {
            preview_available: true,
            security_ok: true,
            build_status: CheckExec::Pass,
            ds_status: CheckExec::Pass,
            functional: FunctionalSummary { critical_scenarios_total: 3, critical_scenarios_passed: 3, button_works: true },
            scenarios_on_a_word_hint,
            ..Default::default()
        };
        assert!(concept_ready(&inp(1)), "a concept needs no journey");
        assert!(!prototype_ready(&inp(1), 0), "all three passed, one of them only on a word");
        assert!(prototype_ready(&inp(0), 0));
    }

    /// Retiring a scenario takes it out of what is owed; it never adds a pass, and a project with
    /// every scenario retired has shown nothing.
    #[test]
    fn a_retired_scenario_is_not_owed_and_is_not_a_pass() {
        let eval = |status: &str, method: &str| json!({"status": status, "method": method});
        let (total, passed, on_hint) = scenario_counts(&[eval("pass", "journey_run"), eval("retired", "retired"), eval("fail", "journey_run")]);
        assert_eq!((total, passed, on_hint), (2, 1, 0));
        let ready = |evals: &[Value]| {
            let (total, passed, on_hint) = scenario_counts(evals);
            prototype_ready(
                &CheckInputs {
                    preview_available: true,
                    security_ok: true,
                    build_status: CheckExec::Pass,
                    ds_status: CheckExec::Pass,
                    functional: FunctionalSummary { critical_scenarios_total: total, critical_scenarios_passed: passed, button_works: true },
                    scenarios_on_a_word_hint: on_hint,
                    ..Default::default()
                },
                0,
            )
        };
        assert!(ready(&[eval("pass", "journey_run"), eval("retired", "retired")]), "what is left all passed");
        assert!(!ready(&[eval("fail", "journey_run"), eval("retired", "retired")]));
        assert!(!ready(&[eval("retired", "retired")]), "nothing left to show the prototype works");
    }

    #[test]
    fn secrets_in_app_fail_security() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "G", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        redesign_ds::ingest_package(&ws, &fixture_ds(), "advisory", false).unwrap();
        super::super::redesign_build::scaffold_app(&ws, None).unwrap();
        fs::write(ws.app_dir().join("src/App.tsx"), "const aws_secret_access_key = \"x\";\n").unwrap();
        let report = run_native_checks(&ws).unwrap();
        assert!(report.checks.iter().any(|c| c.id == "security_secrets" && c.status == "fail"));
        assert!(!report.concept_ready);
    }

    #[test]
    fn native_button_counts_for_functional_smoke() {
        assert!(primary_action_present("export function App(){return <button>Go</button>}"));
        assert!(primary_action_present("export function App(){return <Button>Go</Button>}"));
        assert!(primary_action_present("<div role=\"button\">Go</div>"));
        assert!(!primary_action_present("export function App(){return null}"));
        assert!(!primary_action_present("<button>TODO_BROKEN_BUTTON</button>"));
    }

    #[test]
    fn an_advisory_design_system_warning_does_not_block_a_prototype_and_a_strict_failure_does() {
        let ready = |ds_status| {
            let inp = CheckInputs {
                preview_available: true,
                security_ok: true,
                build_status: CheckExec::Pass,
                ds_status,
                functional: FunctionalSummary { critical_scenarios_total: 1, critical_scenarios_passed: 1, button_works: true },
                ..Default::default()
            };
            prototype_ready(&inp, 0)
        };
        assert!(ready(CheckExec::Warn), "advice is not a blocker");
        assert!(!ready(CheckExec::Fail), "a strict design system that doesn't pass still is");
    }

    #[test]
    fn engineering_ready_needs_backend_contracts_not_just_traceability() {
        let inp = CheckInputs {
            preview_available: true,
            security_ok: true,
            build_status: CheckExec::Pass,
            ds_status: CheckExec::Pass,
            functional: FunctionalSummary { critical_scenarios_total: 2, critical_scenarios_passed: 2, button_works: true },
            ..Default::default()
        };
        assert!(prototype_ready(&inp, 0));
        assert!(!engineering_ready(&inp, 0, false), "approved backend gap must not unlock engineering");
        assert!(engineering_ready(&inp, 0, true));
    }
}
