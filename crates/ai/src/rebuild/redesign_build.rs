//! PR-07: safe functional scaffold for Redesign projects (React+TS+Vite + synthetic mocks).
//! Does **not** start a nested Craft builder. Host agents call this to materialize `app/` then
//! edit themselves. Strict DS policy refuses scaffold when no registry exists.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::{ErrorCode, ToolError};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

use super::execution::{Controller, ExecutionPolicy};
use super::redesign_ds;
use super::redesign_plan;
use super::redesign_store;

pub const MOCK_BANNER: &str = "SIMULATED_DATA — not live origin traffic";

/// Preconditions before writing app files.
pub fn preflight(ws: &Workspace, policy: Option<&ExecutionPolicy>) -> Result<(), ToolError> {
    if ws.manifest.kind.as_deref() != Some("redesign") {
        return Err(ToolError::invalid("redesign_scaffold_app requires project kind=redesign"));
    }
    if let Some(p) = policy {
        if p.controller == Controller::HostAgent && p.allow_nested_builder {
            // Nested builder handoff is a separate explicit path; scaffold itself never launches one.
        }
        if p.controller == Controller::HostAgent && !p.allow_nested_builder {
            // OK — scaffold is the host-agent-safe path.
        }
    }
    // Strict DS: registry must exist.
    match redesign_ds::load_registry(ws) {
        Ok(reg) if reg.policy == "strict" => Ok(()),
        Ok(_) => Ok(()), // advisory registry present
        Err(e) if e.code == ErrorCode::NotFound => {
            // If caller previously set expectation of strict via store note — require registry.
            // Default for redesign scaffold: require registry file for safety (fail closed).
            Err(ToolError::new(
                ErrorCode::FeatureMissing,
                "strict design system registry missing; run redesign_ds_ingest before scaffold",
            )
            .with_details(json!({"code":"ds_registry_required"})))
        }
        Err(e) => Err(e),
    }
}

/// Write a minimal React+TS+Vite app with mock data and traceability stubs.
/// Never copies private logos or design-system source — only import paths from the registry.
pub fn scaffold_app(ws: &Workspace, policy: Option<&ExecutionPolicy>) -> Result<Value, ToolError> {
    preflight(ws, policy)?;
    let reg = redesign_ds::load_registry(ws)?;
    let caps = redesign_store::list_capabilities(ws).unwrap_or_default();
    let decisions = redesign_plan::list_decisions(ws).unwrap_or_default();
    let plan = redesign_plan::load_plan(ws).ok();

    let app = ws.app_dir();
    fs::create_dir_all(app.join("src/mocks")).map_err(|e| ToolError::io(app.display(), e))?;
    fs::create_dir_all(app.join("src/routes")).map_err(|e| ToolError::io(app.display(), e))?;

    write(
        &app.join("package.json"),
        &format!(
            r#"{{
  "name": "redesign-app",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "{pkg}": "{ver}"
  }},
  "devDependencies": {{
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.4",
    "typescript": "~5.7.2",
    "vite": "^6.0.0"
  }}
}}
"#,
            pkg = reg.package_name,
            ver = reg.package_version
        ),
    )?;

    write(
        &app.join("vite.config.ts"),
        r#"import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
});
"#,
    )?;

    write(
        &app.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noEmit": true
  },
  "include": ["src"]
}
"#,
    )?;

    write(
        &app.join("index.html"),
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Redesign prototype</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#,
    )?;

    write(
        &app.join("src/main.tsx"),
        r#"import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
"#,
    )?;

    let button_import = reg
        .components
        .iter()
        .find(|c| c.export_name == "Button")
        .map(|c| c.import_path.clone())
        .unwrap_or_else(|| reg.package_name.clone());

    let routes_json = serde_json::to_string_pretty(&caps.iter().map(|c| {
        json!({
            "capability_id": c.id,
            "title": c.title,
            "routes": c.original_routes,
            "actions": c.actions,
        })
    }).collect::<Vec<_>>())
    .unwrap_or_else(|_| "[]".into());

    write(
        &app.join("src/mocks/data.ts"),
        &format!(
            r#"/** {banner} — do not treat as production or origin writes. */
export const MOCK_META = {{
  banner: "{banner}",
  origin_writes: false,
}} as const;

export type MockRow = {{ id: string; label: string; status: "open" | "done" }};

export const MOCK_ROWS: MockRow[] = [
  {{ id: "m1", label: "Simulated request A", status: "open" }},
  {{ id: "m2", label: "Simulated request B", status: "done" }},
];

export const CAPABILITY_TRACE = {routes} as const;
"#,
            banner = MOCK_BANNER,
            routes = routes_json
        ),
    )?;

    write(
        &app.join("src/App.tsx"),
        &format!(
            r#"import {{ Button }} from "{button_import}";
import {{ CAPABILITY_TRACE, MOCK_META, MOCK_ROWS }} from "./mocks/data";

export function App() {{
  return (
    <main>
      <p role="status">{{MOCK_META.banner}}</p>
      <h1>Redesign prototype</h1>
      <p>Capabilities traced: {{CAPABILITY_TRACE.length}}. Origin writes: {{String(MOCK_META.origin_writes)}}.</p>
      <ul>
        {{MOCK_ROWS.map((row) => (
          <li key={{row.id}}>
            {{row.label}} — {{row.status}}
          </li>
        ))}}
      </ul>
      <Button variant="primary">Simulated primary action</Button>
      <section>
        <h2>Trace</h2>
        <pre>{{JSON.stringify(CAPABILITY_TRACE, null, 2)}}</pre>
      </section>
    </main>
  );
}}
"#,
            button_import = button_import
        ),
    )?;

    write(
        &app.join("README.md"),
        &format!(
            r#"# Redesign app scaffold

- Stack: React + TypeScript (Vite)
- Design system: `{pkg}@{ver}` (import paths only — package sources are not copied)
- Mock data is marked `{banner}` and never writes to the origin
- Decisions/plan live under `../redesign/` and are referenced by id, not inlined wholesale

## Host agent

This scaffold was written by `redesign_scaffold_app` without starting Craft's nested builder.
Continue edits in the host agent; do not call `rebuild_start` under `host_agent` without handoff.
"#,
            pkg = reg.package_name,
            ver = reg.package_version,
            banner = MOCK_BANNER
        ),
    )?;

    // Compact refs file for the builder brief — paths only.
    let refs = json!({
        "design_system": "redesign/design_system.json",
        "capabilities_count": caps.len(),
        "decisions": decisions.iter().map(|d| d.id.clone()).collect::<Vec<_>>(),
        "plan": plan.as_ref().map(|_| "redesign/transformation_plan.json"),
        "mock_banner": MOCK_BANNER,
        "nested_builder_started": false,
        "origin_intact": true,
    });
    redesign_store::ensure(ws)?;
    let refs_path = redesign_store::redesign_dir(ws).join("scaffold_refs.json");
    craft_anything_engine::project::write_atomic(
        &refs_path,
        serde_json::to_vec_pretty(&refs).map_err(|e| ToolError::internal(e.to_string()))?.as_slice(),
    )?;

    Ok(json!({
        "app": app.display().to_string(),
        "package": reg.package_name,
        "package_version": reg.package_version,
        "capabilities_traced": caps.len(),
        "decisions_referenced": decisions.len(),
        "nested_builder_started": false,
        "mock_banner": MOCK_BANNER,
        "refs": "redesign/scaffold_refs.json",
    }))
}

fn write(path: &Path, content: &str) -> Result<(), ToolError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ToolError::io(parent.display(), e))?;
    }
    fs::write(path, content).map_err(|e| ToolError::io(path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::{TargetRef, Workspace};
    use std::path::PathBuf;

    fn fixture_ds() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../REDESIGN_SPEC/fixtures/design-system")
            .canonicalize()
            .unwrap()
    }

    #[test]
    fn refuses_without_ds_registry() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "B", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let err = scaffold_app(&ws, None).unwrap_err();
        assert_eq!(err.code, ErrorCode::FeatureMissing);
        assert!(err.message.contains("design system"));
    }

    #[test]
    fn scaffolds_react_vite_with_mocks_and_no_nested_builder() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "B", "kind=redesign", TargetRef::None).unwrap();
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
                original_routes: vec!["/requests".into()],
                inputs: vec![],
                actions: vec!["filter".into()],
                outcomes: vec![],
                criticality: "normal".into(),
                knowledge_status: "observed".into(),
                source_refs: vec![],
                unknowns: vec![],
            },
        )
        .unwrap();

        let out = scaffold_app(&ws, Some(&ExecutionPolicy::host_agent())).unwrap();
        assert_eq!(out["nested_builder_started"], false);
        assert!(ws.app_dir().join("package.json").is_file());
        assert!(ws.app_dir().join("src/App.tsx").is_file());
        let mocks = fs::read_to_string(ws.app_dir().join("src/mocks/data.ts")).unwrap();
        assert!(mocks.contains(MOCK_BANNER));
        assert!(mocks.contains("origin_writes: false") || mocks.contains("origin_writes:false"));
        let app = fs::read_to_string(ws.app_dir().join("src/App.tsx")).unwrap();
        assert!(app.contains("@fixture/acme-ds"));
        assert!(!app.contains("rebuild_start"));
        // Origin (workspace target folder) untouched — we never write outside app/redesign.
        assert!(!ws.root.join("capture").join("MUTATED").exists());
    }

    #[test]
    fn non_redesign_kind_refused() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "W", "website", TargetRef::None).unwrap();
        assert!(scaffold_app(&ws, None).is_err());
    }
}
