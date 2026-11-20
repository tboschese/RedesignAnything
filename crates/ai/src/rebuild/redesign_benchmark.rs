//! PR-11: Benchmark harness — controlled arms A/B/C/D with honest accounting.
//! Never invents KPI values; missing telemetry stays `unavailable`.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageAccounting {
    pub quality: String, // reported | estimated | unavailable
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_tokens: Option<u64>,
    pub tool_calls: Option<u64>,
    pub wall_ms: Option<u64>,
}

impl UsageAccounting {
    pub fn unavailable() -> Self {
        Self { quality: "unavailable".into(), input_tokens: None, output_tokens: None, cache_tokens: None, tool_calls: None, wall_ms: None }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkArmResult {
    pub arm: String, // A|B|C|D
    pub label: String,
    pub fixture_id: String,
    pub controller: String,
    pub model: Option<String>,
    pub usage: UsageAccounting,
    pub quality_notes: Vec<String>,
    pub claims: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkRun {
    pub schema_version: String,
    pub upstream_pin: String,
    pub environment: Value,
    pub arms: Vec<BenchmarkArmResult>,
    pub notes: Vec<String>,
}

pub fn arm_label(arm: &str) -> &'static str {
    match arm {
        "A" => "Craft base",
        "B" => "Craft + UX prompt",
        "C" => "Redesign Anything Lean",
        "D" => "Optional REA Deep",
        _ => "unknown",
    }
}

/// Build a run record. Pass measured usage only when actually observed; otherwise unavailable.
pub fn build_run(upstream_pin: &str, environment: Value, arms: Vec<BenchmarkArmResult>) -> BenchmarkRun {
    BenchmarkRun {
        schema_version: "1.0.0".into(),
        upstream_pin: upstream_pin.into(),
        environment,
        arms,
        notes: vec![
            "No KPI values are invented; unavailable telemetry stays unavailable (not zero).".into(),
            "REA Deep (D) is optional and must not strengthen claims without new tests.".into(),
            "Blind evaluation protocol: see REDESIGN_SPEC/BENCHMARK_PROTOCOL.md.".into(),
        ],
    }
}

pub fn validate_arm(arm: &BenchmarkArmResult) -> Result<(), String> {
    if !matches!(arm.arm.as_str(), "A" | "B" | "C" | "D") {
        return Err(format!("unknown arm {}", arm.arm));
    }
    if arm.usage.quality == "unavailable" {
        if arm.usage.input_tokens.is_some() || arm.usage.output_tokens.is_some() {
            return Err("unavailable usage must not carry token counts".into());
        }
    }
    if arm.usage.quality == "unavailable" {
        // Treating unavailable as zero is forbidden in claims.
        for c in &arm.claims {
            if c.to_ascii_lowercase().contains("0 tokens") || c.contains("zero tokens") {
                return Err("must not claim zero tokens when usage is unavailable".into());
            }
        }
    }
    if arm.arm == "D" {
        for c in &arm.claims {
            if c.to_ascii_lowercase().contains("proven superior") && arm.quality_notes.is_empty() {
                return Err("arm D cannot claim superiority without measured notes".into());
            }
        }
    }
    Ok(())
}

pub fn dry_run_template(fixture_id: &str) -> BenchmarkRun {
    let mk = |arm: &str, controller: &str| BenchmarkArmResult {
        arm: arm.into(),
        label: arm_label(arm).into(),
        fixture_id: fixture_id.into(),
        controller: controller.into(),
        model: None,
        usage: UsageAccounting::unavailable(),
        quality_notes: vec!["not executed in this dry-run".into()],
        claims: vec![],
        limitations: vec!["results pending controlled execution".into()],
    };
    build_run(
        "827be601e781daea91ce2013fa0f64d658abd028",
        json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "status": "dry_run",
        }),
        vec![mk("A", "craft_runner"), mk("B", "craft_runner"), mk("C", "host_agent"), mk("D", "host_agent")],
    )
}

/// Publishable run record for First public release packaging.
/// Human blind scores stay `unavailable` — never invent KPIs or “gain vs B”.
pub fn publish_template(fixture_id: &str, arm_c_wall_ms: Option<u64>, arm_c_quality_notes: Vec<String>) -> BenchmarkRun {
    let mut run = dry_run_template(fixture_id);
    run.environment = json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "status": "published_partial",
        "human_blind_eval": "unavailable",
        "impact_verdict": "unevaluated",
    });
    run.notes.push("Human A/B/C blind evaluation is unavailable in this publish; do not claim gain vs arm B.".into());
    run.notes.push("Arm C may include automated fixture smoke wall time only; tokens remain unavailable without host telemetry.".into());
    for arm in &mut run.arms {
        match arm.arm.as_str() {
            "C" => {
                arm.quality_notes = arm_c_quality_notes.clone();
                if arm.quality_notes.is_empty() {
                    arm.quality_notes.push("automated fixture path executed or scheduled".into());
                }
                arm.limitations = vec!["human_blind_score=unavailable".into(), "no claim of superiority vs B".into()];
                if let Some(ms) = arm_c_wall_ms {
                    arm.usage = UsageAccounting {
                        quality: "estimated".into(),
                        input_tokens: None,
                        output_tokens: None,
                        cache_tokens: None,
                        tool_calls: None,
                        wall_ms: Some(ms),
                    };
                }
            }
            "D" => {
                arm.limitations = vec!["optional REA Deep — fixture adapter available; not required for lean".into(), "human_blind_score=unavailable".into()];
            }
            _ => {
                arm.limitations = vec!["arm not re-executed in this publish; usage unavailable".into(), "human_blind_score=unavailable".into()];
            }
        }
    }
    run
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_not_zero_tokens_claim() {
        let mut arm = BenchmarkArmResult {
            arm: "C".into(),
            label: arm_label("C").into(),
            fixture_id: "dense-form".into(),
            controller: "host_agent".into(),
            model: None,
            usage: UsageAccounting::unavailable(),
            quality_notes: vec![],
            claims: vec!["used 0 tokens".into()],
            limitations: vec![],
        };
        assert!(validate_arm(&arm).is_err());
        arm.claims.clear();
        assert!(validate_arm(&arm).is_ok());
    }

    #[test]
    fn dry_run_has_four_arms_no_invented_kpis() {
        let run = dry_run_template("dense-form");
        assert_eq!(run.arms.len(), 4);
        for a in &run.arms {
            assert_eq!(a.usage.quality, "unavailable");
            assert!(a.usage.input_tokens.is_none());
            assert!(a.claims.is_empty());
            validate_arm(a).unwrap();
        }
        assert!(run.notes.iter().any(|n| n.contains("unavailable")));
    }

    #[test]
    fn arm_d_cannot_claim_superiority_without_notes() {
        let arm = BenchmarkArmResult {
            arm: "D".into(),
            label: arm_label("D").into(),
            fixture_id: "x".into(),
            controller: "host_agent".into(),
            model: None,
            usage: UsageAccounting::unavailable(),
            quality_notes: vec![],
            claims: vec!["proven superior to lean".into()],
            limitations: vec![],
        };
        assert!(validate_arm(&arm).is_err());
    }

    #[test]
    fn publish_template_keeps_impact_unevaluated() {
        let run = publish_template("dense-form", Some(1200), vec!["fixture smoke".into()]);
        assert_eq!(run.environment["impact_verdict"], "unevaluated");
        assert_eq!(run.environment["human_blind_eval"], "unavailable");
        let c = run.arms.iter().find(|a| a.arm == "C").unwrap();
        assert_eq!(c.usage.wall_ms, Some(1200));
        assert!(c.usage.input_tokens.is_none());
        assert!(c.limitations.iter().any(|l| l.contains("no claim of superiority")));
        for a in &run.arms {
            validate_arm(a).unwrap();
        }
    }
}
