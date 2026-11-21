//! Exportable redesign report (markdown + HTML) for any redesign project.
//! Does not invent pass scores; lists checks as recorded, and says first what waits for the
//! person, how each scenario was judged and how alike each page is to the original.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

use super::redesign_checks::ReadyReport;
use super::redesign_quality::QualityCheck;
use super::redesign_store;

/// A titled list of plain lines, for both the markdown and the HTML twin.
pub type Section = (String, Vec<String>);

fn strings(v: &Value) -> Vec<String> {
    v.as_array().into_iter().flatten().filter_map(|x| x.as_str().map(str::to_string)).collect()
}

/// What a person reads before the list of checks: the mode, what waits for their approval, what
/// is not passing, each scenario with how it was judged, and likeness to the original.
pub fn project_sections(ws: &Workspace, ready: &ReadyReport) -> Vec<Section> {
    let check = |id: &str| ready.checks.iter().find(|c| c.id == id);
    let mut out = Vec::new();

    let mode = super::redesign_mode::RedesignMode::of(ws);
    let mut about = vec![format!("Mode: {} (`{}`)", mode.label(), mode.as_str())];
    if let Ok(Some(variant)) = super::redesign_variants::variant_of(ws) {
        about.push(format!("A variant of the study in `{}`", variant.source_project));
    }
    out.push(("This version".to_string(), about));

    let mut waiting = Vec::new();
    if let Some(c) = check("traceability") {
        waiting.extend(strings(&c.details["needs_approval"]).into_iter().map(|l| format!("Needs your approval: {l}")));
        waiting.extend(strings(&c.details["blocked"]).into_iter().map(|l| format!("Blocked: {l}")));
    }
    if let Some(c) = check("backend_contracts") {
        waiting.extend(strings(&c.details["blocking"]).into_iter().map(|l| format!("Needs backend work that an approval does not create: {l}")));
    }
    if let Some(c) = check("identity_essence").filter(|c| c.status == "fail") {
        waiting.push(format!("The target's essence: {}", c.summary));
    }
    if waiting.is_empty() {
        waiting.push("Nothing waits for an approval.".into());
    }
    out.push(("Waiting for you".to_string(), waiting));

    let not_passing: Vec<String> = ready.checks.iter().filter(|c| c.status != "pass").map(|c| format!("`{}` is {}: {}", c.id, c.status, c.summary)).collect();
    out.push(("Not passing".to_string(), if not_passing.is_empty() { vec!["Every check passes.".into()] } else { not_passing }));

    let evals = check("functional_scenarios").map(|c| c.details["evals"].as_array().cloned().unwrap_or_default()).unwrap_or_default();
    let scenarios: Vec<String> = evals
        .iter()
        .map(|e| {
            let note = e["notes"][0].as_str().unwrap_or("");
            // A journey's own note says what ran; a word hint needs saying that nothing did.
            let how = match e["method"].as_str() {
                Some("journey_run") | Some("retired") => note.to_string(),
                Some("word_hint") => format!("a word was found in the sources; no journey was run ({note})"),
                _ => "not judged".to_string(),
            };
            format!("`{}` — {} — {how}", e["scenario_id"].as_str().unwrap_or("scenario"), e["status"].as_str().unwrap_or("unknown"))
        })
        .collect();
    out.push(("Scenarios".to_string(), if scenarios.is_empty() { vec!["No scenario was evaluated.".into()] } else { scenarios }));

    let whole = |v: Option<f64>| v.map_or("not measured".to_string(), |n| format!("{n:.0}"));
    let likeness: Vec<String> = match super::redesign_likeness::load(ws) {
        None => vec!["The prototype was not measured against the original (redesign_prototype_capture).".into()],
        Some(report) => report
            .pages
            .iter()
            .map(|p| {
                let mut line = format!(
                    "`{}` — overall {} · structure {} · picture {} · text {}",
                    p.page,
                    whole(p.overall),
                    whole(p.structure),
                    whole(p.picture),
                    whole(p.text)
                );
                if !p.not_measured.is_empty() {
                    line.push_str(&format!(" · not measured: {}", p.not_measured.join("; ")));
                }
                line
            })
            .collect(),
    };
    out.push(("Likeness to the original (0–100; reported, not judged)".to_string(), likeness));
    out
}

pub fn write_reports(
    ws: &Workspace,
    fixture_id: Option<&str>,
    ready: &ReadyReport,
    quality: &[QualityCheck],
    sections: &[Section],
    extra: &Value,
) -> Result<Value, ToolError> {
    redesign_store::ensure(ws)?;
    let dir = redesign_store::redesign_dir(ws).join("report");
    fs::create_dir_all(&dir).map_err(|e| ToolError::io(dir.display(), e))?;

    let app_rel = "app/";
    let routes: Vec<String> = fs::read_dir(ws.app_dir().join("src/routes"))
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("tsx"))
                .map(|e| format!("app/src/routes/{}", e.file_name().to_string_lossy()))
                .collect()
        })
        .unwrap_or_default();

    let maturity = if ready.engineering_ready {
        "engineering_ready"
    } else if ready.prototype_ready {
        "prototype_ready"
    } else if ready.concept_ready {
        "concept_ready"
    } else {
        "not_ready"
    };
    let gate_pass = ready.concept_ready;

    let mut md = String::new();
    md.push_str("# Redesign Anything — project report\n\n");
    md.push_str(&format!("- Generated for project: `{}`\n", ws.root.display()));
    if let Some(f) = fixture_id {
        md.push_str(&format!("- Fixture: `{f}`\n"));
    }
    md.push_str(&format!("- Ready maturity claim: **{maturity}**\n"));
    md.push_str(&format!(
        "- concept_ready={} · prototype_ready={} · engineering_ready={}\n\n",
        ready.concept_ready, ready.prototype_ready, ready.engineering_ready
    ));
    for (title, lines) in sections {
        md.push_str(&format!("## {title}\n\n"));
        for line in lines {
            md.push_str(&format!("- {line}\n"));
        }
        md.push('\n');
    }
    md.push_str("## Preview\n\n");
    md.push_str(&format!("- Scaffold app directory: `{app_rel}` (open with Vite after `npm install` in that folder)\n"));
    md.push_str("- Hash routes are listed below; this report does not start a browser.\n");
    if routes.is_empty() {
        md.push_str("- No `src/routes/*.tsx` pages found.\n");
    } else {
        for r in &routes {
            md.push_str(&format!("- `{r}`\n"));
        }
    }
    md.push_str("\n## Native Ready checks\n\n");
    for c in &ready.checks {
        md.push_str(&format!("- `{}` — **{}** — {}\n", c.id, c.status, c.summary));
    }
    md.push_str("\n## Quality checks\n\n");
    if quality.is_empty() {
        md.push_str("- (none)\n");
    } else {
        for c in quality {
            md.push_str(&format!("- `{}` — **{}** — {}\n", c.id, c.status, c.summary));
        }
    }
    md.push_str("\n## Notes\n\n");
    for n in &ready.notes {
        md.push_str(&format!("- {n}\n"));
    }
    md.push_str("- Pixel likeness is not a gate.\n");
    md.push_str("- This file is not a First public release impact benchmark by itself.\n");

    let md_path = dir.join("report.md");
    craft_anything_engine::project::write_atomic(&md_path, md.as_bytes())?;

    let html = format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>Redesign Anything report</title>
  <style>
    body {{ font-family: ui-sans-serif, system-ui, sans-serif; margin: 2rem; max-width: 52rem; line-height: 1.45; }}
    code {{ background: #f4f4f5; padding: 0.1rem 0.3rem; }}
    .pass {{ color: #166534; }} .fail {{ color: #991b1b; }} .warn {{ color: #9a3412; }}
    .ne {{ color: #52525b; }}
  </style>
</head>
<body>
  <h1>Redesign Anything — project report</h1>
  <p>Project: <code>{root}</code>{fixture}</p>
  <p>Ready maturity: <strong>{maturity}</strong> · concept gate: <strong class="{gate_class}">{gate}</strong></p>
  {sections_html}
  <h2>Preview</h2>
  <p>Scaffold under <code>app/</code>. Start locally with <code>npm install &amp;&amp; npm run dev</code> in that folder. Hash routes:</p>
  <ul>{routes_html}</ul>
  <h2>Native Ready checks</h2>
  <ul>{ready_html}</ul>
  <h2>Quality checks</h2>
  <ul>{quality_html}</ul>
  <p><em>Offline report — no invented scores. Open <code>redesign/report/report.md</code> for the text twin.</em></p>
</body>
</html>
"#,
        root = html_escape(&ws.root.display().to_string()),
        fixture = fixture_id.map(|f| format!(" · Fixture: <code>{}</code>", html_escape(f))).unwrap_or_default(),
        maturity = html_escape(maturity),
        gate = if gate_pass { "pass" } else { "fail" },
        gate_class = if gate_pass { "pass" } else { "fail" },
        sections_html = sections
            .iter()
            .map(|(title, lines)| {
                format!("<h2>{}</h2><ul>{}</ul>", html_escape(title), lines.iter().map(|l| format!("<li>{}</li>", html_escape(l))).collect::<Vec<_>>().join(""))
            })
            .collect::<Vec<_>>()
            .join("\n  "),
        routes_html = if routes.is_empty() {
            "<li><em>No route pages</em></li>".into()
        } else {
            routes.iter().map(|r| format!("<li><code>{}</code></li>", html_escape(r))).collect::<Vec<_>>().join("")
        },
        ready_html = ready
            .checks
            .iter()
            .map(|c| {
                format!(
                    "<li><code>{}</code> — <strong class=\"{}\">{}</strong> — {}</li>",
                    html_escape(&c.id),
                    status_class(&c.status),
                    html_escape(&c.status),
                    html_escape(&c.summary)
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        quality_html = if quality.is_empty() {
            "<li><em>none</em></li>".into()
        } else {
            quality
                .iter()
                .map(|c| {
                    format!(
                        "<li><code>{}</code> — <strong class=\"{}\">{}</strong> — {}</li>",
                        html_escape(&c.id),
                        status_class(&c.status),
                        html_escape(&c.status),
                        html_escape(&c.summary)
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        },
    );
    let html_path = dir.join("report.html");
    craft_anything_engine::project::write_atomic(&html_path, html.as_bytes())?;

    let index = json!({
        "markdown": rel(ws, &md_path),
        "html": rel(ws, &html_path),
        "preview_app": app_rel,
        "preview_hint": "cd app && npm install && npm run dev",
        "routes": routes,
        "extra": extra,
    });
    let index_path = dir.join("index.json");
    craft_anything_engine::project::write_atomic(&index_path, serde_json::to_vec_pretty(&index).map_err(|e| ToolError::internal(e.to_string()))?.as_slice())?;

    Ok(index)
}

fn rel(ws: &Workspace, path: &Path) -> String {
    path.strip_prefix(&ws.root).unwrap_or(path).display().to_string()
}

fn status_class(status: &str) -> &'static str {
    match status {
        "pass" => "pass",
        "fail" | "blocked" => "fail",
        "warn" => "warn",
        _ => "ne",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rebuild::redesign_checks::{CheckResult, ReadyReport};
    use craft_anything_engine::workspace::{TargetRef, Workspace};

    #[test]
    fn writes_md_and_html() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        fs::create_dir_all(ws.app_dir().join("src/routes")).unwrap();
        fs::write(ws.app_dir().join("src/routes/home.tsx"), "//").unwrap();
        let ready = ReadyReport {
            concept_ready: true,
            prototype_ready: false,
            engineering_ready: false,
            checks: vec![CheckResult { id: "build_scaffold".into(), status: "pass".into(), summary: "ok".into(), details: json!({}) }],
            notes: vec!["native".into()],
        };
        let out = write_reports(&ws, Some("dense-form"), &ready, &[], &project_sections(&ws, &ready), &json!({})).unwrap();
        assert!(ws.root.join(out["html"].as_str().unwrap()).is_file());
        assert!(ws.root.join(out["markdown"].as_str().unwrap()).is_file());
        let html = fs::read_to_string(ws.root.join(out["html"].as_str().unwrap())).unwrap();
        assert!(html.contains("Redesign Anything"));
        assert!(html.contains("home.tsx"));
    }

    /// The report says first what the person has to do and how each scenario was judged, in both
    /// of its files.
    #[test]
    fn a_report_says_what_waits_for_the_person_and_how_each_scenario_was_judged() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::None).unwrap();
        let check = |id: &str, status: &str, details: Value| CheckResult { id: id.into(), status: status.into(), summary: format!("{id} summary"), details };
        let ready = ReadyReport {
            concept_ready: true,
            prototype_ready: false,
            engineering_ready: false,
            checks: vec![
                check("build_scaffold", "pass", json!({})),
                check("traceability", "blocked", json!({"needs_approval": ["cap_search: divergent decision dec_search status=proposed"], "blocked": []})),
                check(
                    "functional_scenarios",
                    "warn",
                    json!({"evals": [
                        {"scenario_id": "search", "status": "pass", "method": "journey_run", "notes": ["journey run in the built prototype: 3 step(s) passed"]},
                        {"scenario_id": "donate", "status": "pass", "method": "word_hint", "notes": ["found hint \"donate\""]},
                    ]}),
                ),
            ],
            notes: vec![],
        };
        let sections = project_sections(&ws, &ready);
        let lines = |title: &str| sections.iter().find(|(t, _)| t.starts_with(title)).map(|(_, l)| l.join("\n")).unwrap_or_default();
        assert!(lines("Waiting for you").contains("dec_search"), "{sections:?}");
        assert!(lines("Not passing").contains("traceability") && !lines("Not passing").contains("build_scaffold"), "{sections:?}");
        assert!(
            lines("Scenarios").contains("`search` — pass — journey run in the built prototype")
                && lines("Scenarios").contains("`donate` — pass — a word was found"),
            "{sections:?}"
        );
        assert!(lines("Likeness").contains("not measured"), "{sections:?}");

        let out = write_reports(&ws, None, &ready, &[], &sections, &json!({})).unwrap();
        for file in ["markdown", "html"] {
            let text = fs::read_to_string(ws.root.join(out[file].as_str().unwrap())).unwrap();
            assert!(text.contains("Waiting for you") && text.contains("dec_search"), "{file}: {text}");
        }
        let approved = ReadyReport { checks: vec![check("traceability", "pass", json!({"needs_approval": [], "blocked": []}))], ..ready };
        assert!(project_sections(&ws, &approved)[1].1.join("").contains("Nothing waits"));
    }
}
