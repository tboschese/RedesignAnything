//! Judging one benchmark trial by mechanical measures, as `BENCHMARK_RUN_PLAN_2026-11-21.md`
//! registers: no human rater and no LLM decides a result.
//!
//! Each critical scenario has fixed checks (`benchmark/expectations/2026-11-21.json`): texts from
//! the oracle that must show, controls that must be disabled or absent, values a field must keep.
//! The evaluating agent writes the journey that reaches them in the build under test, naming the
//! controls as that build labels them; a journey that leaves out a fixed check, changes its text or
//! reorders the checks does not count. The rubric then comes from the journeys, the captures and,
//! for the design-system fixture, the design-system check.

use serde::Serialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The registered expectations, frozen with the run plan (v2: finer measures, before any
/// confirmatory trial).
pub const EXPECTATIONS: &str = "REDESIGN_SPEC/benchmark/expectations/2026-11-21-v2.json";

/// An announced error at most this far from its field counts as shown next to it.
pub const NEAR_PX: u64 = 120;

/// The expectations file in this checkout.
pub fn expectations_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(EXPECTATIONS)
}

fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Whether journey step `step` is the fixed check `check`, and the name it gave the control or
/// field the check is about (if any).
fn satisfies(check: &Value, step: &Value) -> Option<Option<String>> {
    let s = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
    let named = |is: &[&str]| -> Option<Option<String>> {
        let state = s(step, "is")?;
        is.contains(&state.as_str()).then(|| s(step, "check"))
    };
    if let Some(text) = s(check, "shows") {
        return (s(step, "expect").map(|e| norm(&e)) == Some(norm(&text))).then_some(None);
    }
    if check.get("alert").is_some() {
        return step.get("alert").is_some().then_some(None);
    }
    if let Some(path) = s(check, "open") {
        return (s(step, "open").map(|o| o.trim().to_string()) == Some(path)).then_some(None);
    }
    if check.get("present").is_some() {
        return named(&["present", "enabled", "disabled"]);
    }
    if check.get("absent").is_some() {
        return named(&["absent"]);
    }
    if check.get("enabled").is_some() {
        return named(&["enabled"]);
    }
    if check.get("disabled").is_some() {
        return named(&["disabled"]);
    }
    if check.get("disabled_or_absent").is_some() {
        return named(&["unusable", "disabled", "absent"]);
    }
    if check.get("value").is_some() {
        let equals = s(check, "equals")?;
        return (s(step, "equals").map(|e| norm(&e)) == Some(norm(&equals))).then(|| s(step, "value_of")).filter(|n| n.is_some());
    }
    None
}

/// What a check is about, for the report: its role ("submit claim") or its text.
fn role_of(check: &Value) -> String {
    ["present", "absent", "enabled", "disabled", "disabled_or_absent", "value", "shows", "open"]
        .iter()
        .find_map(|k| check.get(*k).and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| "an announced error".into())
}

/// Whether `steps` reach every fixed check of a scenario, in order. `Ok` holds, for each check
/// about a control or field, its role and the name the journey used; `Err` says the first fixed
/// check the journey does not reach.
pub fn reaches(checks: &[Value], steps: &[Value]) -> Result<Vec<(String, String)>, String> {
    let mut names = Vec::new();
    let mut from = 0;
    for check in checks {
        let hit = steps[from..].iter().enumerate().find_map(|(i, step)| satisfies(check, step).map(|n| (i, n)));
        match hit {
            Some((i, name)) => {
                if let Some(name) = name {
                    names.push((role_of(check), name));
                }
                from += i + 1;
            }
            None => return Err(format!("the journey does not reach the fixed check {check} (after the ones before it)")),
        }
    }
    Ok(names)
}

/// One page's measures from its `capture.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PageMeasures {
    /// Whether the page is no wider than its window at 390 px; None when it wasn't captured there.
    pub fits_phone: Option<bool>,
    pub unnamed_controls: u64,
    /// reached ÷ controls, or 1 when the walk hit its press limit; None when not walked.
    pub keyboard_reach: Option<f64>,
    /// 1 − controls without a visible focus ÷ reached; None when not walked.
    pub focus_visible: Option<f64>,
    /// texts too faint ÷ texts checked; None when not measured.
    pub faint: Option<f64>,
}

/// The measures a capture recorded.
pub fn page_measures(capture: &Value) -> PageMeasures {
    let n = |p: &str| capture.pointer(p).and_then(Value::as_u64);
    let fits_phone = capture
        .get("layouts")
        .and_then(Value::as_array)
        .and_then(|ls| ls.iter().find(|l| l["width"].as_u64() == Some(390)).and_then(|l| Some(l["page_w"].as_u64()? <= 391)));
    let (keyboard_reach, focus_visible) = match capture.pointer("/a11y/keyboard") {
        Some(k) => {
            let (controls, reached, hidden) =
                (k["controls"].as_u64().unwrap_or(0), k["reached"].as_u64().unwrap_or(0), k["focus_not_visible"].as_u64().unwrap_or(0));
            let reach = if k["stopped"] == "limit" || controls == 0 { 1.0 } else { (reached as f64 / controls as f64).min(1.0) };
            let visible = if reached == 0 { 1.0 } else { 1.0 - hidden as f64 / reached as f64 };
            (Some(reach), Some(visible))
        }
        None => (None, None),
    };
    let faint = match (n("/a11y/low_contrast"), n("/a11y/text_checked")) {
        (Some(low), Some(checked)) if checked > 0 => Some(low as f64 / checked as f64),
        _ => None,
    };
    PageMeasures { fits_phone, unnamed_controls: n("/a11y/unnamed_controls").unwrap_or(0), keyboard_reach, focus_visible, faint }
}

/// The plan's rubric for one trial, each dimension 1–5. D6–D8 are left out when the fixture gives
/// nothing to measure them on, and the mean is over the dimensions measured.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rubric {
    pub d1_task_success: f64,
    pub d2_error_prevention: f64,
    pub d3_fits_the_screen: f64,
    pub d4_consistency: f64,
    pub d5_keyboard_and_contrast: f64,
    pub d6_interaction_cost: Option<f64>,
    pub d7_main_action_in_first_view: Option<f64>,
    pub d8_error_next_to_its_field: Option<f64>,
    pub mean: f64,
}

/// The finer measures of v2, each 1–5 or None when not measurable on this fixture.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Finer {
    pub d6: Option<f64>,
    pub d7: Option<f64>,
    pub d8: Option<f64>,
}

/// The user actions of a journey: clicks, typing and key presses.
pub fn actions(steps: &[Value]) -> usize {
    steps.iter().filter(|s| ["click", "type", "press"].iter().any(|k| s.get(*k).is_some())).count()
}

/// D6 for one scenario: 3 when it takes the original's number of actions, 5 at half or fewer, 1 at
/// twice or more, on a log scale.
pub fn cost_score(original: usize, arm: usize) -> f64 {
    if arm == 0 {
        return 5.0;
    }
    (3.0 + 2.0 * (original as f64 / arm as f64).log2()).clamp(1.0, 5.0)
}

/// D6: the mean cost score over the scenarios that need actions on the original; a scenario that
/// failed scores 1.
pub fn interaction_cost(passed: &[(String, bool)], journeys: &[(String, Vec<Value>)], original: &serde_json::Map<String, Value>) -> Option<f64> {
    mean(passed.iter().filter_map(|(id, ok)| {
        let base = original.get(id)?.as_u64().filter(|n| *n > 0)? as usize;
        let steps = journeys.iter().find(|(j, _)| j == id).map(|(_, s)| s);
        Some(match (ok, steps) {
            (true, Some(steps)) => cost_score(base, actions(steps)),
            _ => 1.0,
        })
    }))
}

/// D7: 1 + 4 · the share of captured pages × widths (390 and 1440) whose first window shows the
/// main action.
pub fn main_action_in_first_view(captures: &[Value], name: &str) -> Option<f64> {
    let want = norm(name);
    let seen: Vec<bool> = captures
        .iter()
        .flat_map(|c| c["layouts"].as_array().cloned().unwrap_or_default())
        .filter(|l| matches!(l["width"].as_u64(), Some(390 | 1440)))
        .filter_map(|l| l["first_view_controls"].as_array().map(|names| names.iter().filter_map(Value::as_str).any(|n| norm(n) == want)))
        .collect();
    (!seen.is_empty()).then(|| 1.0 + 4.0 * seen.iter().filter(|s| **s).count() as f64 / seen.len() as f64)
}

/// D8: 1 + 4 · the share of announced errors in the field-error scenarios shown within
/// [`NEAR_PX`] of their field. `rows` are the journey result rows of those scenarios; an alert with
/// no field to measure against is not next to one.
pub fn error_next_to_field(rows: &[Value]) -> Option<f64> {
    let near: Vec<bool> =
        rows.iter().filter(|r| r["ok"] == true && r["step"].get("alert").is_some()).map(|r| r["alert_gap_px"].as_u64().is_some_and(|g| g <= NEAR_PX)).collect();
    (!near.is_empty()).then(|| 1.0 + 4.0 * near.iter().filter(|n| **n).count() as f64 / near.len() as f64)
}

fn mean(xs: impl Iterator<Item = f64>) -> Option<f64> {
    let v: Vec<f64> = xs.collect();
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// Compute the rubric (`BENCHMARK_RUN_PLAN_2026-11-21.md`, "Rubric"). `passed` holds each critical
/// scenario and whether it passed; `ds_status` is the design-system check's status for the DS
/// fixture, None for the others. A measure no page recorded counts as its best value, and the
/// report says which ones were missing.
pub fn rubric(passed: &[(String, bool)], error_scenarios: &[String], pages: &[PageMeasures], ds_status: Option<&str>, finer: &Finer) -> Rubric {
    let rate = |items: Vec<bool>| if items.is_empty() { 1.0 } else { items.iter().filter(|p| **p).count() as f64 / items.len() as f64 };
    let d1 = 1.0 + 4.0 * rate(passed.iter().map(|(_, p)| *p).collect());
    let d2 = 1.0 + 4.0 * rate(passed.iter().filter(|(id, _)| error_scenarios.contains(id)).map(|(_, p)| *p).collect());
    let fits: Vec<bool> = pages.iter().filter_map(|p| p.fits_phone).collect();
    let d3 = if fits.iter().all(|f| *f) { 5.0 } else { 1.0 + 4.0 * rate(fits) };
    let d4 = match ds_status {
        Some("pass") => 5.0,
        Some("warn") => 3.0,
        Some(_) => 1.0,
        None => 5.0 - (pages.iter().map(|p| p.unnamed_controls).max().unwrap_or(0).min(4) as f64),
    };
    let r = mean(pages.iter().filter_map(|p| p.keyboard_reach)).unwrap_or(1.0);
    let v = mean(pages.iter().filter_map(|p| p.focus_visible)).unwrap_or(1.0);
    let f = mean(pages.iter().filter_map(|p| p.faint)).unwrap_or(0.0);
    let d5 = 1.0 + 4.0 * r * v * (1.0 - f);
    let round = |x: f64| (x * 100.0).round() / 100.0;
    let all = [Some(d1), Some(d2), Some(d3), Some(d4), Some(d5), finer.d6, finer.d7, finer.d8];
    let measured: Vec<f64> = all.iter().flatten().copied().collect();
    let mean = round(measured.iter().sum::<f64>() / measured.len() as f64);
    Rubric {
        d1_task_success: round(d1),
        d2_error_prevention: round(d2),
        d3_fits_the_screen: round(d3),
        d4_consistency: round(d4),
        d5_keyboard_and_contrast: round(d5),
        d6_interaction_cost: finer.d6.map(round),
        d7_main_action_in_first_view: finer.d7.map(round),
        d8_error_next_to_its_field: finer.d8.map(round),
        mean,
    }
}

/// The measures no captured page recorded, which the rubric took at their best value.
pub fn missing_measures(pages: &[PageMeasures]) -> Vec<&'static str> {
    let mut out = Vec::new();
    if pages.iter().all(|p| p.fits_phone.is_none()) {
        out.push("page width at 390 px");
    }
    if pages.iter().all(|p| p.keyboard_reach.is_none()) {
        out.push("keyboard walk");
    }
    if pages.iter().all(|p| p.faint.is_none()) {
        out.push("text contrast");
    }
    out
}

/// The source files of a built app that the design-system check reads: scripts and styles,
/// outside node_modules and build output.
pub fn app_sources(app: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    super::redesign_checks::files_under(app, 0, &mut files);
    let skip = ["node_modules", "dist", "build", "out", ".git", "redesign", "capture"];
    files
        .into_iter()
        .filter(|p| {
            let rel = p.strip_prefix(app).unwrap_or(p);
            !rel.components().any(|c| skip.contains(&c.as_os_str().to_string_lossy().as_ref()))
                && matches!(p.extension().and_then(|e| e.to_str()), Some("tsx" | "ts" | "jsx" | "js" | "css" | "html"))
        })
        .filter_map(|p| Some((p.strip_prefix(app).unwrap_or(&p).display().to_string(), std::fs::read_to_string(&p).ok()?)))
        .collect()
}

/// Each critical scenario's id and its fixed checks.
pub type ScenarioChecks = Vec<(String, Vec<Value>)>;

/// The fixed checks of `fixture` from the expectations, by scenario, and its error scenarios.
pub fn fixture_checks(expectations: &Value, fixture: &str) -> Option<(ScenarioChecks, Vec<String>)> {
    let scenarios = expectations.pointer(&format!("/fixtures/{fixture}"))?.as_object()?;
    let checks = scenarios.iter().map(|(id, c)| (id.clone(), c.as_array().cloned().unwrap_or_default())).collect();
    let errors = expectations["error_scenarios"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect();
    Some((checks, errors))
}

/// The record of one judged trial.
pub fn record(fixture: &str, arm: &str, trial: u32, scenarios: Vec<Value>, rubric: &Rubric, missing: &[&str], ds: Option<&Value>) -> Value {
    json!({
        "plan": "REDESIGN_SPEC/benchmark/BENCHMARK_RUN_PLAN_2026-11-21.md",
        "expectations": EXPECTATIONS,
        "fixture_id": fixture,
        "arm": arm,
        "trial": trial,
        "scenarios": scenarios,
        "rubric": rubric,
        "measures_missing": missing,
        "design_system": ds,
        "judged_by": "mechanical checks in Craft Anything's browser; no human rater and no LLM decided a result",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checks() -> Vec<Value> {
        vec![json!({"open": "/?role=viewer"}), json!({"shows": "Viewers cannot submit claims.", "after": "…"}), json!({"disabled_or_absent": "submit claim"})]
    }

    /// A journey counts only when it reaches every fixed check, in order, with the oracle's own
    /// texts; the controls may carry the build's own names, and those names are reported.
    #[test]
    fn a_journey_counts_only_when_it_reaches_the_fixed_checks_in_order() {
        let steps = |v: Value| v.as_array().unwrap().clone();
        let good = steps(json!([
            {"open": "/?role=viewer"}, {"type": "Taxi", "into": "What was it for"}, {"click": "Next"},
            {"expect": "viewers  cannot submit CLAIMS."}, {"check": "Send claim", "is": "disabled"}
        ]));
        assert_eq!(reaches(&checks(), &good).unwrap(), vec![("submit claim".to_string(), "Send claim".to_string())]);

        let reworded = steps(json!([{"open": "/?role=viewer"}, {"expect": "Viewers can't submit."}, {"check": "Send claim", "is": "disabled"}]));
        assert!(reaches(&checks(), &reworded).is_err(), "the oracle's text is fixed");
        let enabled = steps(json!([{"open": "/?role=viewer"}, {"expect": "Viewers cannot submit claims."}, {"check": "Send claim", "is": "enabled"}]));
        assert!(reaches(&checks(), &enabled).is_err(), "the wrong state is not the check");
        let reordered = steps(json!([{"check": "Send claim", "is": "absent"}, {"open": "/?role=viewer"}, {"expect": "Viewers cannot submit claims."}]));
        assert!(reaches(&checks(), &reordered).is_err(), "checks keep their order");
        let value = [json!({"value": "amount field", "equals": "40"})];
        assert!(reaches(&value, &steps(json!([{"value_of": "Amount", "equals": " 40 "}]))).is_ok());
        assert!(reaches(&value, &steps(json!([{"value_of": "Amount", "equals": "41"}]))).is_err());
    }

    /// The rubric follows the plan: passed scenarios, error scenarios, pages that fit, the design
    /// system (or names), and keyboard reach × visible focus × contrast.
    #[test]
    fn the_rubric_is_computed_from_the_measures() {
        let page = |fits: bool, reach: f64, visible: f64, faint: f64| PageMeasures {
            fits_phone: Some(fits),
            unnamed_controls: 0,
            keyboard_reach: Some(reach),
            focus_visible: Some(visible),
            faint: Some(faint),
        };
        let passed = vec![("a".to_string(), true), ("b".to_string(), false), ("c".to_string(), true), ("d".to_string(), true)];
        let perfect = rubric(&passed, &["b".into()], &[page(true, 1.0, 1.0, 0.0)], Some("pass"), &Finer::default());
        assert_eq!(
            (perfect.d1_task_success, perfect.d2_error_prevention, perfect.d3_fits_the_screen, perfect.d4_consistency, perfect.d5_keyboard_and_contrast),
            (4.0, 1.0, 5.0, 5.0, 5.0)
        );
        let worse = rubric(&passed, &["b".into()], &[page(true, 1.0, 1.0, 0.0), page(false, 0.5, 0.5, 0.5)], Some("fail"), &Finer::default());
        assert!(worse.d3_fits_the_screen < 5.0 && worse.d4_consistency == 1.0 && worse.d5_keyboard_and_contrast < perfect.d5_keyboard_and_contrast);
        assert!(worse.mean < perfect.mean);
        let unnamed = PageMeasures { unnamed_controls: 7, ..Default::default() };
        assert_eq!(
            rubric(&passed, &[], std::slice::from_ref(&unnamed), None, &Finer::default()).d4_consistency,
            1.0,
            "without a design system, names count, capped"
        );
        assert_eq!(missing_measures(&[unnamed]), ["page width at 390 px", "keyboard walk", "text contrast"]);
    }

    /// The finer measures: fewer actions than the original score above 3, more below; the main
    /// action counts where the first window shows it; an error counts next to its field within
    /// the limit. They join the mean only when measured.
    #[test]
    fn the_finer_measures_reward_fewer_actions_a_visible_main_action_and_errors_by_their_field() {
        assert_eq!(cost_score(4, 4), 3.0);
        assert!(cost_score(4, 2) > cost_score(4, 3) && cost_score(4, 3) > 3.0 && cost_score(4, 8) < 3.0);
        assert_eq!((cost_score(4, 1), cost_score(4, 40)), (5.0, 1.0), "capped both ways");
        let original = json!({"a": 4, "b": 4, "z": 0});
        let journeys =
            vec![("a".to_string(), vec![json!({"click": "Go"}), json!({"type": "x", "into": "y"})]), ("b".to_string(), vec![json!({"click": "Go"})])];
        let d6 = interaction_cost(&[("a".into(), true), ("b".into(), false), ("z".into(), true)], &journeys, original.as_object().unwrap()).unwrap();
        assert_eq!(d6, (5.0 + 1.0) / 2.0, "a failed scenario scores 1; one with no actions on the original is left out");

        let layout = |w: u64, names: &[&str]| json!({"width": w, "first_view_controls": names});
        let captures = [json!({"layouts": [layout(1440, &["Submit claim"]), layout(390, &["Menu"]), layout(768, &["Submit claim"])]})];
        assert_eq!(main_action_in_first_view(&captures, "submit  CLAIM"), Some(3.0), "seen at 1440, not at 390; 768 doesn't count");
        assert_eq!(main_action_in_first_view(&[json!({"layouts": [{"width": 390}]})], "Go"), None);

        let row = |gap: Option<u64>| {
            let mut r = json!({"ok": true, "step": {"alert": ""}});
            if let Some(g) = gap {
                r["alert_gap_px"] = json!(g);
            }
            r
        };
        assert_eq!(error_next_to_field(&[row(Some(8)), row(Some(500)), row(None), json!({"ok": true, "step": {"click": "Go"}})]), Some(1.0 + 4.0 / 3.0));
        assert_eq!(error_next_to_field(&[]), None);

        let passed = [("a".to_string(), true)];
        let base = rubric(&passed, &[], &[], None, &Finer::default());
        let finer = rubric(&passed, &[], &[], None, &Finer { d6: Some(1.0), d7: None, d8: None });
        assert!(finer.mean < base.mean && finer.d7_main_action_in_first_view.is_none(), "a measured dimension joins the mean, an unmeasured one doesn't");
    }

    #[test]
    fn a_capture_gives_its_measures() {
        let c = json!({"layouts": [{"width": 1440, "page_w": 1440}, {"width": 390, "page_w": 412}],
            "a11y": {"unnamed_controls": 2, "low_contrast": 3, "text_checked": 30, "keyboard": {"controls": 10, "reached": 8, "focus_not_visible": 2, "stopped": "wrapped"}}});
        let m = page_measures(&c);
        assert_eq!((m.fits_phone, m.unnamed_controls, m.keyboard_reach, m.focus_visible), (Some(false), 2, Some(0.8), Some(0.75)));
        assert!((m.faint.unwrap() - 0.1).abs() < 1e-9);
    }

    /// The reference journeys run on the unchanged originals reach every fixed check, and their
    /// action counts are the registered baseline.
    #[test]
    fn the_reference_journeys_reach_the_fixed_checks_and_give_the_baseline() {
        let e: Value = serde_json::from_slice(&std::fs::read(expectations_path()).unwrap()).unwrap();
        for f in ["dense-form", "multi-step-journey", "ds-claims-journey"] {
            let (checks, _) = fixture_checks(&e, f).unwrap();
            let reference = e["reference_journeys"][f].as_array().unwrap();
            for (id, fixed) in &checks {
                let j = reference.iter().find(|j| j["scenario_id"] == id.as_str()).unwrap_or_else(|| panic!("{f}: no reference journey for {id}"));
                let steps = j["steps"].as_array().unwrap();
                assert!(reaches(fixed, steps).is_ok(), "{f}: {id}: {:?}", reaches(fixed, steps));
                assert_eq!(e["original_actions"][f][id].as_u64(), Some(actions(steps) as u64), "{f}: {id}");
            }
        }
    }

    /// The registered expectations hold every critical scenario of the three fixtures' oracles.
    #[test]
    fn the_expectations_cover_every_critical_scenario_of_the_fixtures() {
        let e: Value = serde_json::from_slice(&std::fs::read(expectations_path()).unwrap()).unwrap();
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures");
        for f in ["dense-form", "multi-step-journey", "ds-claims-journey"] {
            let (checks, _) = fixture_checks(&e, f).unwrap();
            let oracle: Value = serde_json::from_slice(&std::fs::read(fixtures.join(f).join("oracle/scenarios.json")).unwrap()).unwrap();
            for s in oracle["scenarios"].as_array().unwrap() {
                let id = s["id"].as_str().unwrap();
                assert!(checks.iter().any(|(c, list)| c == id && !list.is_empty()), "{f}: {id} has fixed checks");
            }
        }
    }
}
