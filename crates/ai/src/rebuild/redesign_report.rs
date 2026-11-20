//! Exportable redesign report (markdown + HTML) for offline fixture / Ready flows.
//! Does not invent pass scores; lists checks as recorded.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

use super::redesign_checks::ReadyReport;
use super::redesign_quality::QualityCheck;
use super::redesign_store;

pub fn write_reports(ws: &Workspace, fixture_id: Option<&str>, ready: &ReadyReport, quality: &[QualityCheck], extra: &Value) -> Result<Value, ToolError> {
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

fn rel(ws: &Workspace, path: &PathBuf) -> String {
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
        let out = write_reports(&ws, Some("dense-form"), &ready, &[], &json!({})).unwrap();
        assert!(ws.root.join(out["html"].as_str().unwrap()).is_file());
        assert!(ws.root.join(out["markdown"].as_str().unwrap()).is_file());
        let html = fs::read_to_string(ws.root.join(out["html"].as_str().unwrap())).unwrap();
        assert!(html.contains("Redesign Anything"));
        assert!(html.contains("home.tsx"));
    }
}
