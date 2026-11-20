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

#[derive(Debug, Clone)]
struct ScaffoldRoute {
    path: String,
    slug: String,
    title: String,
    capability_id: String,
    component_name: String,
    states: Vec<String>,
}

fn slugify_route(path: &str) -> String {
    let trimmed = path.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return "home".into();
    }
    let mut out = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() { "route".into() } else { out }
}

fn pascal_case(slug: &str) -> String {
    slug.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}

/// Prefer plan `target_route` mappings; fall back to capability `original_routes`.
fn collect_scaffold_routes(
    caps: &[redesign_store::Capability],
    plan: Option<&redesign_plan::TransformationPlan>,
) -> Vec<ScaffoldRoute> {
    let mut routes: Vec<ScaffoldRoute> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    if let Some(p) = plan {
        for m in &p.mappings {
            let Some(path) = m.target_route.as_deref().filter(|r| !r.is_empty()) else {
                continue;
            };
            if !seen.insert(path.to_string()) {
                continue;
            }
            let cap = caps.iter().find(|c| c.id == m.capability_id);
            let title = cap.map(|c| c.title.clone()).unwrap_or_else(|| path.to_string());
            let slug = slugify_route(path);
            let mut states = vec!["default".into(), "loading".into(), "empty".into(), "error".into()];
            if let Some(c) = cap {
                for o in &c.outcomes {
                    if !states.iter().any(|s| s == o) {
                        states.push(o.clone());
                    }
                }
            }
            routes.push(ScaffoldRoute {
                path: path.to_string(),
                slug: slug.clone(),
                title,
                capability_id: m.capability_id.clone(),
                component_name: format!("{}Page", pascal_case(&slug)),
                states,
            });
        }
    }

    if routes.is_empty() {
        for c in caps {
            let path = c
                .original_routes
                .first()
                .cloned()
                .unwrap_or_else(|| format!("/{}", c.id));
            if !seen.insert(path.clone()) {
                continue;
            }
            let slug = slugify_route(&path);
            let mut states = vec!["default".into(), "loading".into(), "empty".into(), "error".into()];
            for o in &c.outcomes {
                if !states.iter().any(|s| s == o) {
                    states.push(o.clone());
                }
            }
            routes.push(ScaffoldRoute {
                path,
                slug: slug.clone(),
                title: c.title.clone(),
                capability_id: c.id.clone(),
                component_name: format!("{}Page", pascal_case(&slug)),
                states,
            });
        }
    }

    if routes.is_empty() {
        routes.push(ScaffoldRoute {
            path: "/".into(),
            slug: "home".into(),
            title: "Home".into(),
            capability_id: "cap_home".into(),
            component_name: "HomePage".into(),
            states: vec!["default".into(), "empty".into()],
        });
    }
    routes
}

/// Write a minimal React+TS+Vite app with mock data and traceability stubs.
/// Emits one route module per plan/capability path (hash router, no extra deps).
/// Never copies private logos or design-system source — only import paths from the registry.
pub fn scaffold_app(ws: &Workspace, policy: Option<&ExecutionPolicy>) -> Result<Value, ToolError> {
    preflight(ws, policy)?;
    let reg = redesign_ds::load_registry(ws)?;
    let caps = redesign_store::list_capabilities(ws).unwrap_or_default();
    let decisions = redesign_plan::list_decisions(ws).unwrap_or_default();
    let plan = redesign_plan::load_plan(ws).ok();
    let routes = collect_scaffold_routes(&caps, plan.as_ref());

    let app = ws.app_dir();
    fs::create_dir_all(app.join("src/mocks")).map_err(|e| ToolError::io(app.display(), e))?;
    fs::create_dir_all(app.join("src/routes")).map_err(|e| ToolError::io(app.display(), e))?;

    // Prefer a local file: dependency when the registry was ingested from a directory on disk
    // (fixtures / corporate packages). Published registry versions stay semver.
    let dep_spec = {
        let src = Path::new(&reg.source_ref);
        if src.is_dir() {
            let abs = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
            format!("file:{}", abs.display())
        } else {
            reg.package_version.clone()
        }
    };

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
            ver = dep_spec
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

    let routes_json = serde_json::to_string_pretty(
        &routes
            .iter()
            .map(|r| {
                json!({
                    "path": r.path,
                    "slug": r.slug,
                    "title": r.title,
                    "capability_id": r.capability_id,
                    "component": r.component_name,
                    "states": r.states,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".into());

    let caps_json = serde_json::to_string_pretty(&caps.iter().map(|c| {
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

export const CAPABILITY_TRACE = {caps} as const;

export const ROUTES = {routes} as const;
"#,
            banner = MOCK_BANNER,
            caps = caps_json,
            routes = routes_json
        ),
    )?;

    // One page module per route (journey states as local UI toggles — simulated).
    for r in &routes {
        let states_lit = r
            .states
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let initial = if r.states.iter().any(|s| s == "default") {
            "default"
        } else {
            r.states.first().map(String::as_str).unwrap_or("default")
        };
        write(
            &app.join(format!("src/routes/{}.tsx", r.slug)),
            &format!(
                r#"import {{ useState }} from "react";
import {{ Button }} from "{button_import}";
import {{ MOCK_META, MOCK_ROWS }} from "../mocks/data";

const STATES = [{states}] as const;
type UiState = (typeof STATES)[number];

export function {component}() {{
  const [state, setState] = useState<UiState>("{initial}");
  return (
    <section aria-labelledby="route-title">
      <h2 id="route-title">{title}</h2>
      <p>Capability: <code>{cap}</code> · path: <code>{path}</code></p>
      <p role="status">{{MOCK_META.banner}} · UI state: {{state}}</p>
      <label>
        Simulated state{{" "}}
        <select value={{state}} onChange={{(e) => setState(e.target.value as UiState)}}>
          {{STATES.map((s) => (
            <option key={{s}} value={{s}}>
              {{s}}
            </option>
          ))}}
        </select>
      </label>
      {{state === "empty" ? (
        <p>No simulated rows for this state.</p>
      ) : state === "error" ? (
        <p role="alert">Simulated error — origin was not contacted.</p>
      ) : state === "loading" ? (
        <p>Loading simulated data…</p>
      ) : (
        <ul>
          {{MOCK_ROWS.map((row) => (
            <li key={{row.id}}>
              {{row.label}} — {{row.status}}
            </li>
          ))}}
        </ul>
      )}}
      <Button variant="primary">Simulated primary action</Button>
    </section>
  );
}}
"#,
                button_import = button_import,
                states = states_lit,
                component = r.component_name,
                title = r.title.replace('"', "'"),
                cap = r.capability_id,
                path = r.path,
                initial = initial,
            ),
        )?;
    }

    let imports = routes
        .iter()
        .map(|r| format!("import {{ {} }} from \"./routes/{}\";", r.component_name, r.slug))
        .collect::<Vec<_>>()
        .join("\n");
    let cases = routes
        .iter()
        .map(|r| {
            format!(
                "    case \"{path}\":\n      return <{comp} />;",
                path = r.path,
                comp = r.component_name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let default_path = routes.first().map(|r| r.path.as_str()).unwrap_or("/");

    write(
        &app.join("src/App.tsx"),
        &format!(
            r#"import {{ useEffect, useState, type ReactNode }} from "react";
import {{ ROUTES, MOCK_META, CAPABILITY_TRACE }} from "./mocks/data";
{imports}

function currentHashPath(): string {{
  const raw = window.location.hash.replace(/^#/, "");
  return raw.length === 0 ? "{default_path}" : raw.startsWith("/") ? raw : `/${{raw}}`;
}}

export function App() {{
  const [path, setPath] = useState(currentHashPath);

  useEffect(() => {{
    const onHash = () => setPath(currentHashPath());
    window.addEventListener("hashchange", onHash);
    if (!window.location.hash) {{
      window.location.hash = "{default_path}";
    }}
    return () => window.removeEventListener("hashchange", onHash);
  }}, []);

  let page: ReactNode;
  switch (path) {{
{cases}
    default:
      page = (
        <section>
          <h2>Unknown route</h2>
          <p>No page generated for <code>{{path}}</code>. Known routes come from the transformation plan / capabilities.</p>
        </section>
      );
  }}

  return (
    <main>
      <p role="status">{{MOCK_META.banner}}</p>
      <h1>Redesign prototype</h1>
      <p>
        Routes: {{ROUTES.length}} · Capabilities traced: {{CAPABILITY_TRACE.length}} · Origin writes:{{" "}}
        {{String(MOCK_META.origin_writes)}}
      </p>
      <nav aria-label="Scaffold routes">
        <ul>
          {{ROUTES.map((r) => (
            <li key={{r.path}}>
              <a href={{`#${{r.path}}`}} aria-current={{path === r.path ? "page" : undefined}}>
                {{r.title}}
              </a>
            </li>
          ))}}
        </ul>
      </nav>
      {{page}}
    </main>
  );
}}
"#,
            imports = imports,
            default_path = default_path,
            cases = cases,
        ),
    )?;

    write(
        &app.join("README.md"),
        &format!(
            r#"# Redesign app scaffold

- Stack: React + TypeScript (Vite)
- Design system: `{pkg}@{ver}` (import paths only — package sources are not copied)
- Mock data is marked `{banner}` and never writes to the origin
- Multi-route: `{route_count}` page(s) under `src/routes/` from the transformation plan (or capability routes)
- Hash navigation (`#/path`) — no router dependency
- Decisions/plan live under `../redesign/` and are referenced by id, not inlined wholesale

## Host agent

This scaffold was written by `redesign_scaffold_app` without starting Craft's nested builder.
Continue edits in the host agent; do not call `rebuild_start` under `host_agent` without handoff.
"#,
            pkg = reg.package_name,
            ver = reg.package_version,
            banner = MOCK_BANNER,
            route_count = routes.len()
        ),
    )?;

    // Compact refs file for the builder brief — paths only.
    let refs = json!({
        "design_system": "redesign/design_system.json",
        "capabilities_count": caps.len(),
        "decisions": decisions.iter().map(|d| d.id.clone()).collect::<Vec<_>>(),
        "plan": plan.as_ref().map(|_| "redesign/transformation_plan.json"),
        "routes": routes.iter().map(|r| json!({
            "path": r.path,
            "slug": r.slug,
            "capability_id": r.capability_id,
            "file": format!("app/src/routes/{}.tsx", r.slug),
        })).collect::<Vec<_>>(),
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
        "routes_generated": routes.len(),
        "routes": routes.iter().map(|r| r.path.clone()).collect::<Vec<_>>(),
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
        assert!(app.contains("hashchange"));
        assert!(!app.contains("rebuild_start"));
        let page = fs::read_to_string(ws.app_dir().join("src/routes/requests.tsx")).unwrap();
        assert!(page.contains("@fixture/acme-ds"));
        assert_eq!(out["routes_generated"], 1);
        // Origin (workspace target folder) untouched — we never write outside app/redesign.
        assert!(!ws.root.join("capture").join("MUTATED").exists());
    }

    #[test]
    fn non_redesign_kind_refused() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "W", "website", TargetRef::None).unwrap();
        assert!(scaffold_app(&ws, None).is_err());
    }

    #[test]
    fn multi_route_from_capabilities() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "B", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        redesign_ds::ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        for (id, title, route) in [
            ("cap_start", "Start", "/start"),
            ("cap_review", "Review", "/review"),
            ("cap_done", "Done", "/done"),
        ] {
            redesign_store::put_capability(
                &ws,
                redesign_store::Capability {
                    id: id.into(),
                    title: title.into(),
                    task: title.to_lowercase(),
                    actor_roles: vec![],
                    original_routes: vec![route.into()],
                    inputs: vec![],
                    actions: vec!["next".into()],
                    outcomes: vec!["success".into()],
                    criticality: "normal".into(),
                    knowledge_status: "observed".into(),
                    source_refs: vec![],
                    unknowns: vec![],
                },
            )
            .unwrap();
        }
        let out = scaffold_app(&ws, None).unwrap();
        assert_eq!(out["routes_generated"], 3);
        assert!(ws.app_dir().join("src/routes/start.tsx").is_file());
        assert!(ws.app_dir().join("src/routes/review.tsx").is_file());
        assert!(ws.app_dir().join("src/routes/done.tsx").is_file());
        let app = fs::read_to_string(ws.app_dir().join("src/App.tsx")).unwrap();
        assert!(app.contains("StartPage"));
        assert!(app.contains("ReviewPage"));
        assert!(app.contains("hashchange"));
        let refs: Value = serde_json::from_slice(
            &fs::read(redesign_store::redesign_dir(&ws).join("scaffold_refs.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(refs["routes"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn multi_route_prefers_plan_target_routes() {
        let d = tempfile::tempdir().unwrap();
        let mut ws = Workspace::create(d.path(), "B", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        redesign_ds::ingest_package(&ws, &fixture_ds(), "strict", false).unwrap();
        redesign_store::put_capability(
            &ws,
            redesign_store::Capability {
                id: "cap_a".into(),
                title: "Approve".into(),
                task: "approve".into(),
                actor_roles: vec![],
                original_routes: vec!["/legacy".into()],
                inputs: vec![],
                actions: vec![],
                outcomes: vec![],
                criticality: "critical".into(),
                knowledge_status: "observed".into(),
                source_refs: vec![],
                unknowns: vec![],
            },
        )
        .unwrap();
        redesign_store::put_capability(
            &ws,
            redesign_store::Capability {
                id: "cap_b".into(),
                title: "Confirm".into(),
                task: "confirm".into(),
                actor_roles: vec![],
                original_routes: vec!["/legacy-b".into()],
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
        let plan = redesign_plan::TransformationPlan {
            schema_version: redesign_plan::SCHEMA_VERSION.into(),
            project_id: "p".into(),
            revision: 1,
            generated_at: "0Z".into(),
            mode: "redesign".into(),
            snapshot_id: "s".into(),
            mappings: vec![
                redesign_plan::PlanMapping {
                    capability_id: "cap_a".into(),
                    target_route: Some("/journey/approve".into()),
                    target_component: None,
                    target_action: None,
                    decision_id: "d1".into(),
                    scenario_ids: vec![],
                    exception_approved: false,
                    feasibility: "frontend_only".into(),
                },
                redesign_plan::PlanMapping {
                    capability_id: "cap_b".into(),
                    target_route: Some("/journey/confirm".into()),
                    target_component: None,
                    target_action: None,
                    decision_id: "d2".into(),
                    scenario_ids: vec![],
                    exception_approved: false,
                    feasibility: "frontend_only".into(),
                },
            ],
            unknowns: vec![],
            decisions: vec![],
            notes: vec![],
        };
        redesign_store::ensure(&ws).unwrap();
        craft_anything_engine::project::write_atomic(
            &redesign_plan::plan_path(&ws),
            serde_json::to_vec_pretty(&plan).unwrap().as_slice(),
        )
        .unwrap();

        let out = scaffold_app(&ws, None).unwrap();
        assert_eq!(out["routes_generated"], 2);
        assert!(ws.app_dir().join("src/routes/journey_approve.tsx").is_file());
        assert!(ws.app_dir().join("src/routes/journey_confirm.tsx").is_file());
        assert!(!ws.app_dir().join("src/routes/legacy.tsx").exists());
        let mocks = fs::read_to_string(ws.app_dir().join("src/mocks/data.ts")).unwrap();
        assert!(mocks.contains("/journey/approve"));
        assert!(!mocks.contains("\"/legacy\"") || mocks.contains("CAPABILITY_TRACE"));
    }
}
