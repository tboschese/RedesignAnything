//! One redesign per mode, on request: sibling projects made from a studied one, so a person can
//! compare how far each mode goes. A variant carries the study (the capture and reference, and
//! from `redesign/` the evidence, capabilities, identity, design system and oracle; decisions come
//! back as proposals) and none of the outputs: no plan, app, checks or reports. Every existing
//! reader of a project's plan and app works on a variant as it is.
//!
//! A variant says where it came from in `redesign/variant.json`; the source lists its variants in
//! `redesign/variants.json`. Both name folders beside the project, never absolute paths, and a
//! listed folder only counts when it points back.

use craft_anything_engine::workspace::{Workspace, copy_folder};
use craft_anything_evidence::{ErrorCode, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

use super::redesign_checks::{CHECKS_FILE, ReadyReport};
use super::redesign_mode::RedesignMode;
use super::redesign_plan;
use super::redesign_store::{self, redesign_dir, reject_symlink, safe_child, write_record};

pub const VARIANT_FILE: &str = "variant.json";
pub const VARIANTS_FILE: &str = "variants.json";
const SCHEMA_VERSION: &str = "1.0.0";
/// Shared inputs under `redesign/`. Everything else there is one project's own output.
const CARRIED: [&str; 5] = ["evidence", "capabilities", "oracle", "identity.json", "design_system.json"];
const NOT_CARRIED: [&str; 6] = ["transformation_plan.json", "scaffold_refs.json", "check_results.json", "quality", "report", "app"];

/// In a variant: the project it was made from and its mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantOf {
    pub schema_version: String,
    /// Folder name of the source project, beside this one.
    pub source_project: String,
    pub mode: RedesignMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantRef {
    pub mode: RedesignMode,
    /// Folder name of the variant, beside the source.
    pub project: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct Variants {
    schema_version: String,
    variants: Vec<VariantRef>,
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, ToolError> {
    reject_symlink(path)?;
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|e| ToolError::invalid(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ToolError::io(path.display(), e)),
    }
}

/// What this project is a variant of, if it is one.
pub fn variant_of(ws: &Workspace) -> Result<Option<VariantOf>, ToolError> {
    read_json(&redesign_dir(ws).join(VARIANT_FILE))
}

fn folder_name(ws: &Workspace) -> Result<String, ToolError> {
    ws.root.file_name().and_then(|n| n.to_str()).map(str::to_string).ok_or_else(|| ToolError::invalid(format!("{} has no folder name", ws.root.display())))
}

fn parent(ws: &Workspace) -> Result<&Path, ToolError> {
    ws.root.parent().ok_or_else(|| ToolError::invalid(format!("{} has no parent folder to put variants in", ws.root.display())))
}

/// A listed variant that is a project beside `source` and points back to it.
fn resolve(source: &Workspace, listed: &VariantRef) -> Result<Option<Workspace>, ToolError> {
    let root = safe_child(parent(source)?, &listed.project)?;
    if !root.join("craft.json").is_file() {
        return Ok(None);
    }
    reject_symlink(&root)?;
    let ws = Workspace::open(&root)?;
    let back = variant_of(&ws)?.is_some_and(|v| v.source_project == folder_name(source).unwrap_or_default() && v.mode == listed.mode);
    Ok(back.then_some(ws))
}

fn listed(source: &Workspace) -> Result<Vec<VariantRef>, ToolError> {
    Ok(read_json::<Variants>(&redesign_dir(source).join(VARIANTS_FILE))?.map(|v| v.variants).unwrap_or_default())
}

/// Make one variant per mode (all three when `modes` is empty) from a studied redesign project.
pub fn create(ws: &Workspace, modes: &[RedesignMode]) -> Result<Value, ToolError> {
    if ws.manifest.kind.as_deref() != Some(super::RedesignKind::ID) {
        return Err(ToolError::invalid("variants are made from a redesign project (kind=redesign)"));
    }
    if let Some(v) = variant_of(ws)? {
        return Err(ToolError::invalid(format!(
            "this project is already the `{}` variant of {}; make variants from that project",
            v.mode.as_str(),
            v.source_project
        )));
    }
    if redesign_store::list_capabilities(ws)?.is_empty() {
        return Err(ToolError::new(ErrorCode::FeatureMissing, "nothing to carry into a variant yet: study the target and record its capabilities first"));
    }
    let mut wanted: Vec<RedesignMode> = if modes.is_empty() { RedesignMode::ALL.to_vec() } else { modes.to_vec() };
    wanted.dedup();
    let mut known = Vec::new();
    for l in listed(ws)? {
        if resolve(ws, &l)?.is_some() {
            if wanted.contains(&l.mode) {
                return Err(ToolError::invalid(format!("this project already has a `{}` variant in {}", l.mode.as_str(), l.project)));
            }
            known.push(l);
        }
    }

    let source_folder = folder_name(ws)?;
    let from = redesign_dir(ws);
    let decisions = redesign_plan::list_decisions(ws)?;
    let mut made = Vec::new();
    for mode in wanted {
        let m = &ws.manifest;
        let mut v = Workspace::create(parent(ws)?, &format!("{} ({})", m.name, mode.label()), &m.request, m.target.clone())?;
        v.manifest.kind.clone_from(&m.kind);
        v.manifest.stack.clone_from(&m.stack);
        v.save()?;
        redesign_store::ensure(&v)?;
        for dir in ["capture", "reference"] {
            let src = ws.root.join(dir);
            if src.is_dir() {
                reject_symlink(&src)?;
                copy_folder(&src, &v.root.join(dir))?;
            }
        }
        let to = redesign_dir(&v);
        for name in CARRIED {
            let src = safe_child(&from, name)?;
            reject_symlink(&src)?;
            if src.is_dir() {
                copy_folder(&src, &to.join(name))?;
            } else if src.is_file() {
                fs::copy(&src, to.join(name)).map_err(|e| ToolError::io(src.display(), e))?;
            }
        }
        // A choice and its approval were made for the source's plan, not for this mode.
        for mut d in decisions.clone() {
            if d.options.iter().any(|o| o.id == "keep_as_is") {
                d.chosen_option_id = "keep_as_is".into();
            }
            d.status = "proposed".into();
            d.approval_evidence_id = None;
            d.rationale = format!("Carried into the `{}` variant as a proposal; choose and approve it for this mode.", mode.as_str());
            redesign_plan::save_decision(&v, &d)?;
        }
        write_record(&to.join(VARIANT_FILE), &VariantOf { schema_version: SCHEMA_VERSION.into(), source_project: source_folder.clone(), mode })?;
        made.push((mode, v));
    }

    known.extend(made.iter().filter_map(|(mode, v)| Some(VariantRef { mode: *mode, project: folder_name(v).ok()? })));
    known.sort_by_key(|r| RedesignMode::ALL.iter().position(|m| *m == r.mode));
    write_record(&from.join(VARIANTS_FILE), &Variants { schema_version: SCHEMA_VERSION.into(), variants: known })?;

    Ok(json!({
        "source": ws.root.display().to_string(),
        "variants": made.iter().map(|(mode, v)| json!({
            "mode": mode.as_str(),
            "label": mode.label(),
            "scope": mode.scope(),
            "project": v.root.display().to_string(),
        })).collect::<Vec<_>>(),
        "carried": ["capture/", "reference/", "redesign/evidence/", "redesign/capabilities/", "redesign/oracle/", "redesign/identity.json", "redesign/design_system.json", "redesign/decisions/ (as proposals)"],
        "not_carried": NOT_CARRIED,
        "next": "In each variant: choose its decisions, redesign_plan_build (the variant's mode is used), redesign_scaffold_app, build the UI, redesign_checks_run. Then redesign_variants_show on any of them to compare.",
    }))
}

fn paths(source: &Workspace) -> Result<Vec<(VariantRef, Option<Workspace>)>, ToolError> {
    listed(source)?.into_iter().map(|l| resolve(source, &l).map(|ws| (l, ws))).collect()
}

fn row(id: &str, report: &ReadyReport) -> Value {
    report.checks.iter().find(|c| c.id == id).map(|c| json!({"status": c.status, "summary": c.summary})).unwrap_or(Value::Null)
}

fn summary(ws: &Workspace) -> Value {
    let checks = redesign_dir(ws).join(CHECKS_FILE);
    let report: Option<ReadyReport> = read_json(&checks).ok().flatten();
    json!({
        "project": ws.root.display().to_string(),
        "name": ws.manifest.name,
        "plan": redesign_plan::plan_path(ws).is_file(),
        "app": ws.app_dir().join("package.json").is_file(),
        "checks": match &report {
            Some(r) => json!({
                "concept_ready": r.concept_ready,
                "prototype_ready": r.prototype_ready,
                "engineering_ready": r.engineering_ready,
                "identity_essence": row("identity_essence", r),
                "identity_drift": row("identity_drift", r),
                "mode_scope": row("mode_scope", r),
            }),
            None => json!({"note": "redesign_checks_run has not been run in this project"}),
        },
    })
}

/// The variants of a project (asked on the source or on any variant), with what its last checks said.
pub fn show(ws: &Workspace) -> Result<Value, ToolError> {
    let source: Workspace = match variant_of(ws)? {
        Some(v) => {
            let root: PathBuf = safe_child(parent(ws)?, &v.source_project)?;
            Workspace::open(&root)
                .map_err(|e| ToolError::new(ErrorCode::NotFound, format!("the source project {} can't be opened: {}", v.source_project, e.message)))?
        }
        None => Workspace::open(&ws.root)?,
    };
    let mut variants = Vec::new();
    let mut not_found = Vec::new();
    for (listed, found) in paths(&source)? {
        match found {
            Some(v) => {
                let mut s = summary(&v);
                s["mode"] = json!(listed.mode.as_str());
                s["label"] = json!(listed.mode.label());
                variants.push(s);
            }
            None => not_found
                .push(json!({"mode": listed.mode.as_str(), "project": listed.project, "why": "not a project beside the source that points back to it"})),
        }
    }
    Ok(json!({
        "source": summary(&source),
        "variants": variants,
        "not_found": not_found,
        "note": if variants.is_empty() { "No variants yet; make them with redesign_variants_create." } else { "Checks are what each project's last redesign_checks_run wrote; this tool runs none." },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::TargetRef;

    fn studied(dir: &Path) -> Workspace {
        let mut ws = Workspace::create(dir, "Catalogue", "kind=redesign", TargetRef::None).unwrap();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let ds = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/design-system").canonicalize().unwrap();
        super::super::redesign_ds::ingest_package(&ws, &ds, "advisory", false).unwrap();
        redesign_store::put_capability(
            &ws,
            redesign_store::Capability {
                id: "cap_search".into(),
                title: "Search".into(),
                task: "search".into(),
                actor_roles: vec![],
                original_routes: vec!["/search".into()],
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
        fs::create_dir_all(ws.root.join("capture")).unwrap();
        fs::write(ws.root.join("capture/capture.json"), "{}").unwrap();
        ws
    }

    fn only(out: &Value) -> Workspace {
        Workspace::open(Path::new(out["variants"][0]["project"].as_str().unwrap())).unwrap()
    }

    #[test]
    fn a_variant_carries_the_study_but_not_the_plan_app_or_results_and_can_be_scaffolded() {
        let d = tempfile::tempdir().unwrap();
        let ws = studied(d.path());
        redesign_plan::build_plan(&ws, "redesign", vec![], vec![]).unwrap();
        super::super::redesign_build::scaffold_app(&ws, None).unwrap();
        super::super::redesign_checks::run_native_checks(&ws).unwrap();

        let v = only(&create(&ws, &[RedesignMode::Refresh]).unwrap());
        assert_eq!(redesign_store::list_capabilities(&v).unwrap().len(), 1, "the study came along");
        assert!(v.root.join("capture/capture.json").is_file());
        for output in ["redesign/transformation_plan.json", "redesign/check_results.json", "redesign/scaffold_refs.json", "app/package.json"] {
            assert!(ws.root.join(output).exists() && !v.root.join(output).exists(), "{output} is the source's own");
        }
        assert_eq!(RedesignMode::of(&v), RedesignMode::Refresh, "before any plan, a variant's mode is the one it was made for");
        super::super::redesign_build::scaffold_app(&v, None).expect("a variant is a working redesign project without a new study");
    }

    #[test]
    fn carried_decisions_lose_their_choice_and_approval() {
        let d = tempfile::tempdir().unwrap();
        let ws = studied(d.path());
        let critic = super::super::redesign_knowledge::critic(
            "dense table",
            &super::super::redesign_knowledge::retrieve("dense table", &["table".into()], &super::super::redesign_knowledge::bundled_catalog(), 5).unwrap(),
        );
        let dec = redesign_plan::propose_decision("dense table", &["cap_search".into()], &critic, "ui_only", &[], &[]);
        redesign_plan::save_decision(&ws, &dec).unwrap();
        redesign_plan::choose_option(&ws, &dec.id, "targeted_improvement").unwrap();
        redesign_plan::approve_decision(&ws, &dec.id, "ev_human_ok").unwrap();

        let v = only(&create(&ws, &[RedesignMode::Reimagine]).unwrap());
        let carried = redesign_plan::load_decision(&v, &dec.id).unwrap();
        assert_eq!((carried.status.as_str(), carried.approval_evidence_id.as_deref()), ("proposed", None), "{carried:?}");
        assert_ne!(carried.chosen_option_id, "targeted_improvement");
        assert_eq!(redesign_plan::load_decision(&ws, &dec.id).unwrap().status, "approved", "the source keeps its own");
    }

    #[test]
    fn a_variant_is_not_forked_again_and_a_mode_is_not_made_twice() {
        let d = tempfile::tempdir().unwrap();
        let ws = studied(d.path());
        let v = only(&create(&ws, &[RedesignMode::Refresh]).unwrap());
        assert!(create(&v, &[]).is_err(), "variants come from the source");
        assert!(create(&ws, &[RedesignMode::Refresh]).is_err(), "already made");
        let rest = create(&ws, &[]).map(|_| ()).err();
        assert!(rest.is_some(), "asking for all three includes the one that exists");
        create(&ws, &[RedesignMode::Redesign, RedesignMode::Reimagine]).unwrap();
        assert_eq!(show(&v).unwrap()["variants"].as_array().unwrap().len(), 3, "asked on a variant, the source's three are listed");
    }

    #[test]
    fn show_lists_only_projects_that_point_back_to_the_source() {
        let d = tempfile::tempdir().unwrap();
        let ws = studied(d.path());
        create(&ws, &[RedesignMode::Refresh]).unwrap();
        let stranger = studied(d.path());
        let mut list = listed(&ws).unwrap();
        list.push(VariantRef { mode: RedesignMode::Reimagine, project: folder_name(&stranger).unwrap() });
        list.push(VariantRef { mode: RedesignMode::Redesign, project: "../elsewhere".into() });
        write_record(&redesign_dir(&ws).join(VARIANTS_FILE), &Variants { schema_version: SCHEMA_VERSION.into(), variants: list[..2].to_vec() }).unwrap();
        let shown = show(&ws).unwrap();
        assert_eq!(shown["variants"].as_array().unwrap().len(), 1, "{shown}");
        assert_eq!(shown["not_found"].as_array().unwrap().len(), 1, "the stranger is named, not trusted: {shown}");

        write_record(&redesign_dir(&ws).join(VARIANTS_FILE), &Variants { schema_version: SCHEMA_VERSION.into(), variants: list }).unwrap();
        assert!(show(&ws).is_err(), "a listed path that leaves the folder is refused");
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_store_entry_is_refused() {
        let d = tempfile::tempdir().unwrap();
        let ws = studied(d.path());
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret.json"), "{}").unwrap();
        let evidence = redesign_dir(&ws).join("evidence");
        fs::remove_dir_all(&evidence).ok();
        std::os::unix::fs::symlink(outside.path(), &evidence).unwrap();
        assert!(create(&ws, &[RedesignMode::Refresh]).is_err());
    }
}
