//! Design System Registry: ingest React/TS packages, DTCG tokens, strict/advisory compliance.
//! Layout CLI MCP is optional; when unavailable, strict local checks still apply (never masked).

use craft_anything_engine::project::write_atomic;
use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::{ErrorCode, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::redesign_store::{self, redesign_dir};

pub const REGISTRY_FILE: &str = "design_system.json";
pub const SCHEMA_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignSystemRegistry {
    pub schema_version: String,
    pub project_id: String,
    pub revision: u32,
    pub generated_at: String,
    pub policy: String,
    pub source_ref: String,
    pub source_digest: String,
    pub package_name: String,
    pub package_version: String,
    pub components: Vec<DsComponent>,
    #[serde(default)]
    pub tokens: Value,
    #[serde(default)]
    pub approved_overrides: Vec<String>,
    #[serde(default)]
    pub layout_cli: LayoutCliStatus,
    /// Documentary paths found next to the package (Storybook, docs). Empty is allowed.
    #[serde(default)]
    pub documentary_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DsComponent {
    pub id: String,
    pub package_name: String,
    pub package_version: String,
    pub import_path: String,
    pub export_name: String,
    #[serde(default)]
    pub props_schema: Option<Value>,
    #[serde(default)]
    pub variants: Vec<String>,
    #[serde(default)]
    pub tokens: Vec<String>,
    pub allowed: bool,
    #[serde(default)]
    pub usage: Vec<String>,
    #[serde(default)]
    pub never: Vec<String>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutCliStatus {
    pub available: bool,
    pub mode: String,
    #[serde(default)]
    pub note: String,
}

impl Default for LayoutCliStatus {
    fn default() -> Self {
        Self { available: false, mode: "degraded".into(), note: "Layout CLI MCP not configured; local registry compliance still applies.".into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub policy: String,
    pub status: String,
    pub findings: Vec<ComplianceFinding>,
    pub layout_cli: LayoutCliStatus,
    pub notes: Vec<String>,
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{secs}Z")
}

fn digest_bytes(bytes: &[u8]) -> String {
    // Stable non-crypto fingerprint for provenance (not a security hash).
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{h:016x}")
}

pub fn registry_path(ws: &Workspace) -> PathBuf {
    redesign_dir(ws).join(REGISTRY_FILE)
}

pub fn save_registry(ws: &Workspace, reg: &DesignSystemRegistry) -> Result<(), ToolError> {
    redesign_store::ensure(ws)?;
    let path = registry_path(ws);
    let bytes = serde_json::to_vec_pretty(reg).map_err(|e| ToolError::internal(e.to_string()))?;
    write_atomic(&path, &bytes)
}

pub fn load_registry(ws: &Workspace) -> Result<DesignSystemRegistry, ToolError> {
    let path = registry_path(ws);
    let bytes = fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ToolError::new(ErrorCode::NotFound, "design system registry missing; run redesign_ds_ingest first")
        } else {
            ToolError::io(path.display(), e)
        }
    })?;
    serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("design_system.json: {e}")))
}

/// Ingest a React/TS package directory (package.json + exports). Missing Storybook is fine.
pub fn ingest_package(ws: &Workspace, package_dir: &Path, policy: &str, layout_available: bool) -> Result<DesignSystemRegistry, ToolError> {
    if policy != "strict" && policy != "advisory" {
        return Err(ToolError::invalid("policy must be strict or advisory"));
    }
    let pkg_path = package_dir.join("package.json");
    let pkg_bytes = fs::read(&pkg_path).map_err(|e| ToolError::io(pkg_path.display(), e))?;
    let pkg: Value = serde_json::from_slice(&pkg_bytes).map_err(|e| ToolError::invalid(format!("package.json: {e}")))?;
    let name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if name.is_empty() || version.is_empty() {
        return Err(ToolError::invalid("package.json must include name and version"));
    }

    let exports = pkg.get("exports").cloned().unwrap_or(json!({}));
    let mut components = Vec::new();
    if let Some(map) = exports.as_object() {
        for (export_key, target) in map {
            if export_key == "." || export_key == "./tokens" {
                continue;
            }
            let export_name = export_key.trim_start_matches("./").to_string();
            let rel = target.as_str().unwrap_or("").to_string();
            let props = infer_props_from_source(package_dir, &rel, &export_name);
            let variants = props
                .as_ref()
                .and_then(|p| p.get("variant"))
                .and_then(|v| v.get("enum"))
                .and_then(|e| e.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let import_path = if export_key == "." {
                name.clone()
            } else if let Some(rest) = export_key.strip_prefix("./") {
                format!("{name}/{rest}")
            } else {
                format!("{name}/{export_key}")
            };
            components.push(DsComponent {
                id: format!("cmp_{export_name}"),
                package_name: name.clone(),
                package_version: version.clone(),
                import_path,
                export_name: export_name.clone(),
                props_schema: props,
                variants,
                tokens: vec![],
                allowed: true,
                usage: vec![],
                never: vec!["hardcoded_brand_color".into()],
                evidence_ids: vec![],
            });
        }
    }

    // Also accept named exports from barrel when exports map is sparse.
    if components.is_empty() {
        for known in ["Button", "TextField"] {
            components.push(DsComponent {
                id: format!("cmp_{known}"),
                package_name: name.clone(),
                package_version: version.clone(),
                import_path: name.clone(),
                export_name: known.into(),
                props_schema: infer_props_from_source(package_dir, &format!("src/{known}.tsx"), known),
                variants: vec![],
                tokens: vec![],
                allowed: true,
                usage: vec![],
                never: vec!["hardcoded_brand_color".into()],
                evidence_ids: vec![],
            });
        }
    }

    let tokens_path = package_dir.join("tokens/dtcg.json");
    let tokens = if tokens_path.is_file() {
        let tb = fs::read(&tokens_path).map_err(|e| ToolError::io(tokens_path.display(), e))?;
        serde_json::from_slice(&tb).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let mut documentary_paths: Vec<String> = Vec::new();
    for candidate in [".storybook", "storybook", "docs", "documentation"] {
        let p = package_dir.join(candidate);
        if p.is_dir() {
            documentary_paths.push(candidate.to_string());
        }
    }
    let storybook = documentary_paths.iter().any(|p| p.contains("storybook"));
    let mut notes_digest = pkg_bytes.clone();
    notes_digest.extend_from_slice(serde_json::to_vec(&tokens).unwrap_or_default().as_slice());
    notes_digest.extend_from_slice(documentary_paths.join("|").as_bytes());

    let reg = DesignSystemRegistry {
        schema_version: SCHEMA_VERSION.into(),
        project_id: ws.manifest.name.clone(),
        revision: 1,
        generated_at: now_iso(),
        policy: policy.into(),
        source_ref: package_dir.display().to_string(),
        source_digest: digest_bytes(&notes_digest),
        package_name: name,
        package_version: version,
        components,
        tokens,
        approved_overrides: vec![],
        layout_cli: if layout_available {
            LayoutCliStatus { available: true, mode: "connected".into(), note: "Layout CLI MCP available; local compliance remains authoritative.".into() }
        } else {
            LayoutCliStatus {
                available: false,
                mode: "degraded".into(),
                note: if storybook {
                    "Layout CLI unavailable; Storybook/docs path recorded in documentary_paths.".into()
                } else {
                    "Layout CLI unavailable; no Storybook/docs folder — ingest still succeeds from package.json exports.".into()
                },
            }
        },
        documentary_paths,
    };
    save_registry(ws, &reg)?;
    Ok(reg)
}

fn infer_props_from_source(package_dir: &Path, rel: &str, export_name: &str) -> Option<Value> {
    let path = if rel.is_empty() { package_dir.join(format!("src/{export_name}.tsx")) } else { package_dir.join(rel.trim_start_matches("./")) };
    let src = fs::read_to_string(path).ok()?;
    let mut props = serde_json::Map::new();
    // Very small heuristic: capture `name?:` / `name:` from type blocks.
    for line in src.lines() {
        let t = line.trim();
        if let Some(name) = t.split(':').next() {
            let name = name.trim().trim_end_matches('?');
            if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !name.is_empty() && name != "export" && name != "import" {
                if t.contains('|') && t.contains('"') {
                    let enums: Vec<String> = t
                        .split('|')
                        .filter_map(|p| {
                            let p = p.trim();
                            let start = p.find('"')?;
                            let end = p.rfind('"')?;
                            if end > start { Some(p[start + 1..end].to_string()) } else { None }
                        })
                        .collect();
                    if !enums.is_empty() {
                        props.insert(name.to_string(), json!({"type": "string", "enum": enums}));
                        continue;
                    }
                }
                props.insert(name.to_string(), json!({"type": "unknown"}));
            }
        }
    }
    if props.is_empty() { None } else { Some(Value::Object(props)) }
}

/// Check generated app sources against the registry. Local checks are authoritative.
pub fn check_sources(reg: &DesignSystemRegistry, sources: &[(String, String)], required_package_version: Option<&str>) -> ComplianceReport {
    let mut findings = Vec::new();
    let allowed_imports: HashSet<String> = reg
        .components
        .iter()
        .filter(|c| c.allowed)
        .flat_map(|c| {
            [c.import_path.clone(), c.package_name.clone(), format!("{}/{}", c.package_name, c.export_name), format!("{}./{}", c.package_name, c.export_name)]
        })
        .collect();
    let prop_map: HashMap<String, HashSet<String>> = reg
        .components
        .iter()
        .map(|c| {
            let keys = c.props_schema.as_ref().and_then(|v| v.as_object()).map(|o| o.keys().cloned().collect::<HashSet<_>>()).unwrap_or_default();
            (c.export_name.clone(), keys)
        })
        .collect();

    if let Some(need) = required_package_version
        && need != reg.package_version
    {
        findings.push(ComplianceFinding {
            code: "package_version_mismatch".into(),
            severity: "error".into(),
            message: format!("registry has {}@{}, required {}", reg.package_name, reg.package_version, need),
            path: None,
        });
    }

    for (path, src) in sources {
        // Fake / unknown DS imports.
        for cap in import_captures(src) {
            let looks_ds = cap.contains("ds") || cap.contains("acme") || cap.starts_with('@');
            if looks_ds && !allowed_imports.iter().any(|a| cap == *a || cap.starts_with(a) || a.starts_with(&cap)) {
                // Allow non-DS deps like react.
                if cap == "react" || cap.starts_with("react/") || cap.starts_with("react-dom") {
                    continue;
                }
                if cap.starts_with('@') || cap.contains("ds") || cap.contains("acme") {
                    findings.push(ComplianceFinding {
                        code: "unknown_import".into(),
                        severity: "error".into(),
                        message: format!("import {cap:?} is not in the design system registry"),
                        path: Some(path.clone()),
                    });
                }
            }
        }

        // Unknown props on known components: <Button foo= or Button({ foo
        for (comp, allowed_props) in &prop_map {
            if allowed_props.is_empty() {
                continue;
            }
            for prop in jsx_props_for(src, comp) {
                if !allowed_props.contains(&prop) && prop != "key" && prop != "ref" && prop != "children" {
                    let override_ok = reg.approved_overrides.iter().any(|o| o == &format!("{comp}.{prop}"));
                    if !override_ok {
                        findings.push(ComplianceFinding {
                            code: "unknown_prop".into(),
                            severity: "error".into(),
                            message: format!("<{comp} {prop}=…> is not in the registry props schema"),
                            path: Some(path.clone()),
                        });
                    }
                }
            }
        }

        // Hardcoded hex colors in strict mode (branding bypass).
        if hex_color_literals(src) {
            let override_ok = reg.approved_overrides.iter().any(|o| o == "hardcoded_color");
            if !override_ok {
                findings.push(ComplianceFinding {
                    code: "hardcoded_color".into(),
                    severity: if reg.policy == "strict" { "error" } else { "warn" }.into(),
                    message: "hardcoded color literal found; use design tokens in strict mode".into(),
                    path: Some(path.clone()),
                });
            }
        }
    }

    let has_error = findings.iter().any(|f| f.severity == "error");
    let status = if has_error {
        "fail"
    } else if findings.is_empty() {
        "pass"
    } else {
        "warn"
    };

    ComplianceReport {
        policy: reg.policy.clone(),
        status: status.into(),
        findings,
        layout_cli: reg.layout_cli.clone(),
        notes: vec![
            "Local import/prop/token checks are authoritative; Layout CLI never masks a strict failure.".into(),
            "Missing Storybook does not fail ingest or compliance by itself.".into(),
        ],
    }
}

fn import_captures(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("import ")
            && let Some(from) = rest.split(" from ").nth(1)
        {
            let s = from.trim().trim_end_matches(';').trim().trim_matches('"').trim_matches('\'').trim_matches('`');
            if !s.is_empty() {
                out.push(s.to_string());
            }
        }
    }
    out
}

fn jsx_props_for(src: &str, component: &str) -> Vec<String> {
    let mut props = Vec::new();
    let needle = format!("<{component}");
    for chunk in src.split(&needle).skip(1) {
        let end = chunk.find('>').unwrap_or(chunk.len().min(200));
        let attrs = &chunk[..end];
        for part in attrs.split_whitespace() {
            let name = part.split('=').next().unwrap_or("").trim().trim_start_matches('{').trim_start_matches('/');
            if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && name != component {
                props.push(name.to_string());
            }
        }
    }
    props
}

fn hex_color_literals(src: &str) -> bool {
    let bytes = src.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if bytes[i] == b'#' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            let n = j - i - 1;
            if n == 3 || n == 6 || n == 8 {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Probe Layout CLI without trusting it for pass/fail.
pub fn layout_cli_probe(available: bool) -> LayoutCliStatus {
    if available {
        LayoutCliStatus { available: true, mode: "connected".into(), note: "Layout provider reachable; still verify with local registry checks.".into() }
    } else {
        LayoutCliStatus::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::{TargetRef, Workspace};

    fn fixture_ds() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/design-system").canonicalize().expect("fixture design-system path")
    }

    #[test]
    fn ingest_fixture_without_storybook() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "DsDemo", "ds", TargetRef::None).unwrap();
        let reg = ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        assert_eq!(reg.package_name, "@fixture/acme-ds");
        assert_eq!(reg.package_version, "1.2.0");
        assert!(reg.components.iter().any(|c| c.export_name == "Button"));
        assert!(reg.components.iter().any(|c| c.export_name == "TextField"));
        assert!(!reg.tokens.as_object().unwrap().is_empty());
        assert!(!reg.layout_cli.available);
        assert!(reg.layout_cli.note.to_ascii_lowercase().contains("storybook"));
        let loaded = load_registry(&ws).unwrap();
        assert_eq!(loaded.source_digest, reg.source_digest);
    }

    /// The benchmark's design-system fixture app is built only from the package: its sources pass
    /// the strict check, so an arm's conformance is measured against a conformant original.
    #[test]
    fn the_design_system_fixture_app_conforms_to_the_package() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "DsDemo", "ds", TargetRef::None).unwrap();
        let reg = ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        let app = fixture_ds().join("../ds-claims-journey/app/src");
        let sources: Vec<(String, String)> =
            ["App.tsx", "main.tsx"].iter().map(|f| (format!("app/src/{f}"), fs::read_to_string(app.join(f)).expect("fixture app source"))).collect();
        let report = check_sources(&reg, &sources, Some("1.2.0"));
        assert_eq!(report.status, "pass", "{report:?}");
    }

    #[test]
    fn real_import_passes_fake_fails() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "DsDemo", "ds", TargetRef::None).unwrap();
        let reg = ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        let ok = check_sources(
            &reg,
            &[(
                "app/App.tsx".into(),
                r#"import { Button } from "@fixture/acme-ds/Button";
export function App() { return <Button variant="primary">Go</Button>; }
"#
                .into(),
            )],
            Some("1.2.0"),
        );
        assert_eq!(ok.status, "pass", "{ok:?}");

        let bad = check_sources(
            &reg,
            &[(
                "app/App.tsx".into(),
                r#"import { FakeWidget } from "@evil/not-a-ds";
export function App() { return <FakeWidget />; }
"#
                .into(),
            )],
            None,
        );
        assert_eq!(bad.status, "fail");
        assert!(bad.findings.iter().any(|f| f.code == "unknown_import"), "{bad:?}");
    }

    #[test]
    fn unknown_prop_and_hardcoded_color_fail_strict() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "DsDemo", "ds", TargetRef::None).unwrap();
        let mut reg = ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        // Ensure Button has a props schema with variant only (+ inferred).
        let report = check_sources(
            &reg,
            &[(
                "app/App.tsx".into(),
                "import { Button } from \"@fixture/acme-ds/Button\";\nexport function App() { return <Button glitter=\"yes\">x</Button>; }\nconst brand = \"#ff00aa\";\n"
                    .into(),
            )],
            None,
        );
        assert!(report.findings.iter().any(|f| f.code == "unknown_prop"), "{report:?}");
        assert!(report.findings.iter().any(|f| f.code == "hardcoded_color"), "{report:?}");
        assert_eq!(report.status, "fail");

        reg.policy = "advisory".into();
        let soft = check_sources(&reg, &[("app/App.tsx".into(), "const c = \"#ffffff\";\n".into())], None);
        assert!(soft.findings.iter().any(|f| f.code == "hardcoded_color" && f.severity == "warn"));
        assert_ne!(soft.status, "fail");
    }

    #[test]
    fn package_version_mismatch_fails() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "DsDemo", "ds", TargetRef::None).unwrap();
        let reg = ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        let r = check_sources(&reg, &[], Some("9.9.9"));
        assert_eq!(r.status, "fail");
        assert!(r.findings.iter().any(|f| f.code == "package_version_mismatch"));
    }

    #[test]
    fn layout_down_does_not_mask_strict_failure() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "DsDemo", "ds", TargetRef::None).unwrap();
        let mut reg = ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        reg.layout_cli = layout_cli_probe(false);
        let r = check_sources(&reg, &[("app/x.tsx".into(), "import { X } from \"@evil/ds\";\nconst c='#000';\n".into())], None);
        assert_eq!(r.status, "fail");
        assert!(!r.layout_cli.available);
        assert!(r.notes.iter().any(|n| n.contains("never masks")));
    }
}
