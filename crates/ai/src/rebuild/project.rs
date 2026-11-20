//! Starting a project from a request: the target, the folder, the stack the person asked for, the
//! asset budget from settings and whether assets may be generated. Shared by the CLI's `rebuild` and the `rebuild_start` tool.

use super::*;
use craft_anything_engine::Settings;
use craft_anything_engine::workspace::{ExistingFiles, suggest_name};

/// Where a new project goes.
#[derive(Debug, Clone, Copy, Default)]
pub enum Place<'a> {
    /// A new folder in the projects folder from settings.
    #[default]
    Projects,
    /// A new folder inside this one.
    In(&'a Path),
    /// This folder itself ([`Workspace::create_at`]), with what its files are if it has any.
    At(&'a Path, Option<ExistingFiles>),
}

/// A stack picked by name for `--build-with` / `build_with` (an engine id or a stack's full name).
pub fn build_with(name: &str) -> Result<&'static stacks::Stack, ToolError> {
    stacks::resolve(name).ok_or_else(|| {
        let known = stacks::catalogue().iter().map(|s| s.engine.clone().unwrap_or_else(|| s.name.clone())).collect::<Vec<_>>().join(", ");
        ToolError::invalid(format!("Unknown build-with {name:?}.")).with_remediation(format!("Use one of: {known}."))
    })
}

/// The stack a project starts with. One edited in place keeps what its files are built with,
/// unless its request names another; any other goes by its request and target.
pub(super) fn first_stack(kinds: &KindRegistry, ws: &Workspace, deep: bool) -> Option<&'static str> {
    let m = &ws.manifest;
    if let Some(id) = m.kind.as_deref()
        && let Some(kind) = kinds.get(id)
    {
        let cx = DetectCx { deep, ..DetectCx::new(&m.request, &m.target, "") };
        return Some(kind.named_stack(&cx).unwrap_or_else(|| kind.default_stack(&cx)));
    }
    let own = m.app_in_root && kinds.named(&m.request, &m.target).is_none();
    let found = own.then(|| stacks::found_in(&ws.app_dir())).flatten();
    found.map(|s| s.name.as_str()).or_else(|| kinds.choose(&m.request, &m.target, deep).map(|(_, stack)| stack))
}

/// Resolve the TargetKind for a project: forced run option, saved manifest kind, then detection.
pub fn resolve_kind(kinds: &KindRegistry, ws: &Workspace, force_kind: Option<&str>) -> Option<std::sync::Arc<dyn TargetKind>> {
    if let Some(id) = force_kind.or(ws.manifest.kind.as_deref()) {
        return kinds.get(id);
    }
    kinds.detect(&ws.manifest.request, &ws.manifest.target, &ws.manifest.stack)
}

/// Validate and normalise an explicit kind id (`redesign`, `website`, …).
pub fn parse_kind(kinds: &KindRegistry, kind: &str) -> Result<&'static str, ToolError> {
    let id = kind.trim();
    kinds.get(id).map(|k| k.id()).ok_or_else(|| {
        let known = kinds.ids().join(", ");
        ToolError::invalid(format!("Unknown kind {id:?}.")).with_remediation(format!("Use one of: {known}."))
    })
}

/// Whether `folder` already holds a project (its `craft.json` is there).
pub fn has_project(folder: &Path) -> bool {
    folder.join(craft_anything_engine::workspace::MANIFEST).is_file()
}

/// What a run says after opening a folder: a request typed for one that already held a project
/// is a follow-up, not a new project's request.
pub fn attached_request(attached: bool, request: &str) -> Option<String> {
    attached.then(|| request.to_string())
}

/// What a created project starts with: its stack, the asset budget from settings, whether its
/// pictures and models may be generated, and its builder. A folder that already held a project
/// keeps all of them; attaching never rewrites what the person set.
pub fn apply_new_project_fields(
    ws: &mut Workspace,
    attached: bool,
    stack: Option<&str>,
    kind: Option<&str>,
    budget: f64,
    generate_assets: bool,
    builder: Option<&str>,
) -> Result<(), ToolError> {
    if attached {
        return Ok(());
    }
    if let Some(name) = stack {
        ws.manifest.stack = name.to_string();
    }
    if let Some(id) = kind {
        ws.manifest.kind = Some(id.to_string());
    }
    ws.set_person(|m| {
        m.asset_budget_usd = budget;
        m.generate_assets = generate_assets;
        m.builder = builder.map(str::to_string);
    })
}

/// What the person picked for a new project besides what it is about and where it goes.
#[derive(Debug, Clone, Copy, Default)]
pub struct NewProjectOpts<'a> {
    /// The stack the person picked.
    pub build_with: Option<&'a str>,
    /// The project kind id.
    pub kind: Option<&'a str>,
    /// The person lets the builder generate pictures and 3D models.
    pub generate_assets: bool,
    /// The builder the person picked.
    pub builder: Option<&'a str>,
}

/// A new project for `request` about `target` (a web address, an absolute path, or `-` for none),
/// at `place`, with the stack the person picked (`opts.build_with`), the asset budget from
/// settings, and `opts.generate_assets`: the person lets the builder generate pictures and 3D models.
pub fn new_project(settings: &Settings, target: &str, request: &str, place: Place, opts: NewProjectOpts) -> Result<Workspace, ToolError> {
    let NewProjectOpts { build_with, kind, generate_assets, builder } = opts;
    let kinds = KindRegistry::built_in();
    let kind_id = kind.map(|k| parse_kind(&kinds, k)).transpose()?;
    let stack = match build_with {
        Some(name) => Some(self::build_with(name)?),
        None if kind_id == Some(crate::rebuild::RedesignKind::ID) => stacks::get(crate::rebuild::RedesignKind::DEFAULT_STACK),
        None => None,
    };
    let target_ref = if target.trim() == "-" { TargetRef::None } else { TargetRef::parse(target) };
    if target_ref == TargetRef::None && target.trim() != "-" {
        return Err(ToolError::invalid(format!("{target:?} is neither a web address nor an existing absolute path"))
            .with_remediation("Pass https://… for a website, a full path for a program, or - for none."));
    }
    let name = suggest_name(request, &target_ref);
    // A folder that already holds a project is attached, not recreated: its saved stack and the
    // fields the person owns stay as they are.
    let attached = matches!(&place, Place::At(folder, _) if has_project(folder));
    let mut ws = match place {
        Place::Projects => Workspace::create(&settings.projects_dir(), &name, request, target_ref)?,
        Place::In(dir) => Workspace::create(dir, &name, request, target_ref)?,
        Place::At(folder, existing) => {
            if attached {
                Workspace::open(folder)?
            } else {
                Workspace::create_at(folder, &name, request, target_ref, existing)?
            }
        }
    };
    apply_new_project_fields(&mut ws, attached, stack.map(|s| s.name.as_str()), kind_id, settings.assets.budget(), generate_assets, builder)?;
    Ok(ws)
}
