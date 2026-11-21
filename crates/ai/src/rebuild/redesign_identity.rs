//! Visual identity traits: which look-and-feel traits of the original are intentional, and
//! whether the redesign keeps, evolves or replaces them. Stored at `redesign/identity.json`.
//!
//! Traits constrain style by context, not structure: a kept trait still evolves (`evolve_within`).
//! An intentional trait left `undecided` counts as `keep` everywhere.
//!
//! The intentional traits are the target's essence, and no redesign may lose it. Replacing one
//! needs a rationale and the operator's approval, recorded as evidence (`approval_evidence_id`,
//! a record with `source: "user"`); without it the trait still counts as kept. The record is an
//! audit trail, not proof: the store can't tell who wrote it. `identity_essence` fails, and blocks
//! Concept Ready, when a replacement lacks that approval or the profile can't be read.
//! `identity_drift` fails, and blocks Prototype Ready, when the app drifts from an intentional
//! kept trait; drift on other kept traits is a warning. `identity_evolution` is advisory.

use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::{ErrorCode, ToolError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::fs;

use super::redesign_checks::CheckResult;
use super::redesign_store::{self, redesign_dir};

pub const IDENTITY_FILE: &str = "identity.json";
pub const SCHEMA_VERSION: &str = "1.0.0";
const TRAIT_ID_PREFIX: &str = "trait_";
const EVIDENCE_ID_PREFIX: &str = "ev_";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TraitCategory {
    Chrome,
    Typography,
    Color,
    Shape,
    Density,
    Imagery,
    Layout,
    Voice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TraitIntent {
    Intentional,
    Incidental,
    #[default]
    Unknown,
}

impl TraitIntent {
    pub const ALL: [Self; 3] = [Self::Intentional, Self::Incidental, Self::Unknown];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Intentional => "intentional",
            Self::Incidental => "incidental",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TraitDecision {
    Keep,
    Evolve,
    Replace,
    #[default]
    Undecided,
}

impl TraitDecision {
    pub const ALL: [Self; 4] = [Self::Keep, Self::Evolve, Self::Replace, Self::Undecided];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::Evolve => "evolve",
            Self::Replace => "replace",
            Self::Undecided => "undecided",
        }
    }
}

/// Static checks `identity_drift` runs against app sources for a kept trait.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TraitChecks {
    /// At least one of these font families must be declared or imported.
    #[serde(default)]
    pub font_families_any: Vec<String>,
    /// None of these font families may be declared or imported.
    #[serde(default)]
    pub forbidden_font_families: Vec<String>,
    /// No `border-radius` / `--radius` value may exceed this many px (rem/em count as 16px).
    #[serde(default)]
    pub max_radius_px: Option<u32>,
    /// At most this many `box-shadow` declarations other than `none`.
    #[serde(default)]
    pub max_shadows: Option<u32>,
    /// Hex colours (`#rgb`, `#rrggbb`…) that must each appear in app sources.
    #[serde(default)]
    pub required_colors: Vec<String>,
}

impl TraitChecks {
    pub fn is_empty(&self) -> bool {
        self.font_families_any.is_empty()
            && self.forbidden_font_families.is_empty()
            && self.max_radius_px.is_none()
            && self.max_shadows.is_none()
            && self.required_colors.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdentityTrait {
    /// `trait_<alnum/_/->`.
    pub id: String,
    pub label: String,
    pub category: TraitCategory,
    /// What Study observed, in plain words.
    pub observation: String,
    #[serde(default)]
    pub intent: TraitIntent,
    #[serde(default)]
    pub decision: TraitDecision,
    /// Required when an intentional trait is replaced.
    #[serde(default)]
    pub rationale: Option<String>,
    /// `ev_…` id of the operator's approval in redesign/evidence/ (`source: "user"`). Required to
    /// replace an intentional trait, or to stop calling a recorded trait intentional.
    #[serde(default)]
    pub approval_evidence_id: Option<String>,
    /// How a kept trait gets easier to use without losing its style.
    #[serde(default)]
    pub evolve_within: Option<String>,
    /// `ev_…` ids in redesign/evidence/ backing the observation.
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub checks: TraitChecks,
}

impl IdentityTrait {
    /// The decision the redesign acts on: an intentional trait is kept when it is undecided, and
    /// when it is marked for replacement without the operator's approval.
    pub fn effective_decision(&self) -> TraitDecision {
        match (self.intent, self.decision) {
            (TraitIntent::Intentional, TraitDecision::Undecided) => TraitDecision::Keep,
            (TraitIntent::Intentional, TraitDecision::Replace) if self.approval_id().is_none() => TraitDecision::Keep,
            (_, d) => d,
        }
    }

    fn approval_id(&self) -> Option<&str> {
        self.approval_evidence_id.as_deref().map(str::trim).filter(|a| !a.is_empty())
    }

    /// An intentional trait the profile wants replaced.
    fn replaces_essence(&self) -> bool {
        self.intent == TraitIntent::Intentional && self.decision == TraitDecision::Replace
    }

    fn rationale_text(&self) -> Option<&str> {
        self.rationale.as_deref().map(str::trim).filter(|r| !r.is_empty())
    }

    fn evolve_text(&self) -> Option<&str> {
        self.evolve_within.as_deref().map(str::trim).filter(|r| !r.is_empty())
    }

    fn is_kept(&self) -> bool {
        self.effective_decision() == TraitDecision::Keep
    }
}

/// Ids of kept traits with no `evolve_within`: character kept, but no way forward written down.
pub fn missing_evolution(profile: &IdentityProfile) -> Vec<&str> {
    profile.traits.iter().filter(|t| t.is_kept() && t.evolve_text().is_none()).map(|t| t.id.as_str()).collect()
}

const STYLE_NOT_STRUCTURE: &str = "Identity traits constrain style, not structure: keep the look by context and still simplify navigation and journeys.";
const MISSING_EVOLUTION_NOTE: &str =
    "Some kept traits say nothing about how they get easier to use; add evolve_within so keeping the look doesn't freeze the product.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct IdentityProfile {
    pub schema_version: String,
    #[serde(default)]
    pub traits: Vec<IdentityTrait>,
    #[serde(default)]
    pub notes: Vec<String>,
}

impl Default for IdentityProfile {
    fn default() -> Self {
        Self { schema_version: SCHEMA_VERSION.into(), traits: Vec::new(), notes: Vec::new() }
    }
}

fn validate_trait(t: &IdentityTrait) -> Result<(), ToolError> {
    redesign_store::validate_id(TRAIT_ID_PREFIX, &t.id)?;
    if t.label.trim().is_empty() {
        return Err(ToolError::invalid(format!("trait {}: label must not be empty", t.id)));
    }
    if t.observation.trim().is_empty() {
        return Err(ToolError::invalid(format!("trait {}: observation must not be empty", t.id)));
    }
    if t.replaces_essence() && t.rationale_text().is_none() {
        return Err(ToolError::new(ErrorCode::BadInput, format!("trait {}: replacing an intentional trait needs a rationale", t.id))
            .with_details(json!({"code": "rationale_required", "trait_id": t.id})));
    }
    if let Some(a) = t.approval_id() {
        redesign_store::validate_id(EVIDENCE_ID_PREFIX, a)?;
    }
    redesign_store::refuse_secretish(&format!("trait {} label", t.id), &t.label)?;
    redesign_store::refuse_secretish(&format!("trait {} observation", t.id), &t.observation)?;
    if let Some(r) = &t.rationale {
        redesign_store::refuse_secretish(&format!("trait {} rationale", t.id), r)?;
    }
    if let Some(e) = &t.evolve_within {
        redesign_store::refuse_secretish(&format!("trait {} evolve_within", t.id), e)?;
    }
    for ev in &t.evidence_ids {
        redesign_store::validate_id(EVIDENCE_ID_PREFIX, ev)?;
    }
    for c in &t.checks.required_colors {
        if !is_hex_color(c) {
            return Err(ToolError::invalid(format!("trait {}: required color {c:?} must be a hex colour like #1a2b3c", t.id)));
        }
    }
    for f in t.checks.font_families_any.iter().chain(&t.checks.forbidden_font_families) {
        if f.trim().is_empty() {
            return Err(ToolError::invalid(format!("trait {}: font family names must not be empty", t.id)));
        }
    }
    Ok(())
}

fn is_hex_color(s: &str) -> bool {
    s.strip_prefix('#').is_some_and(|h| matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The saved profile, or `None` when no identity was recorded.
pub fn load(ws: &Workspace) -> Result<Option<IdentityProfile>, ToolError> {
    let root = redesign_dir(ws);
    if !root.is_dir() {
        return Ok(None);
    }
    let path = redesign_store::safe_child(&root, IDENTITY_FILE)?;
    if !path.exists() {
        return Ok(None);
    }
    redesign_store::reject_symlink(&path)?;
    let bytes = fs::read(&path).map_err(|e| ToolError::io(path.display(), e))?;
    let profile = serde_json::from_slice(&bytes).map_err(|e| ToolError::invalid(format!("redesign/{IDENTITY_FILE}: {e}")))?;
    Ok(Some(profile))
}

/// Whether `id` names the operator's approval: an evidence record with `source: "user"`.
fn approved_by_operator(ws: &Workspace, id: Option<&str>) -> Result<bool, ToolError> {
    let Some(id) = id else { return Ok(false) };
    Ok(redesign_store::get_evidence(ws, id)?.is_some_and(|e| e.source == "user"))
}

fn approval_required(t: &IdentityTrait, what: &str) -> ToolError {
    ToolError::new(
        ErrorCode::BadInput,
        format!(
            "trait {}: {what} needs the operator's approval. Ask them, record their answer with redesign_evidence_put (source \"user\") and pass its id as approval_evidence_id.",
            t.id
        ),
    )
    .with_details(json!({"code": "approval_required", "trait_id": t.id}))
}

/// The essence can't be dropped by relabelling: what was recorded as intentional stays so, and
/// is replaced, only with the operator's approval.
fn guard_essence(ws: &Workspace, stored: &IdentityProfile, traits: &[IdentityTrait], replace_all: bool) -> Result<(), ToolError> {
    for t in traits.iter().filter(|t| t.replaces_essence()) {
        if !approved_by_operator(ws, t.approval_id())? {
            return Err(approval_required(t, "replacing an intentional trait"));
        }
    }
    for old in stored.traits.iter().filter(|t| t.intent == TraitIntent::Intentional) {
        match traits.iter().find(|t| t.id == old.id) {
            Some(new) if new.intent != TraitIntent::Intentional && !approved_by_operator(ws, new.approval_id())? => {
                return Err(approval_required(new, "no longer calling a recorded intentional trait intentional"));
            }
            None if replace_all => {
                return Err(ToolError::new(ErrorCode::BadInput, format!("replace_all would drop the intentional trait {}; include it in traits", old.id))
                    .with_details(json!({"code": "approval_required", "trait_id": old.id})));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Upsert traits by id (or replace the whole set), validating everything before writing.
pub fn record(ws: &Workspace, traits: Vec<IdentityTrait>, replace_all: bool, notes: Option<Vec<String>>) -> Result<Value, ToolError> {
    if traits.is_empty() && !replace_all && notes.is_none() {
        return Err(ToolError::invalid("nothing to record: pass traits, notes or replace_all"));
    }
    let mut seen = HashSet::new();
    for t in &traits {
        validate_trait(t)?;
        if !seen.insert(t.id.as_str()) {
            return Err(ToolError::invalid(format!("trait {} is listed twice", t.id)));
        }
    }
    let stored = load(ws)?.unwrap_or_default();
    guard_essence(ws, &stored, &traits, replace_all)?;
    for n in notes.iter().flatten() {
        redesign_store::refuse_secretish("identity note", n)?;
    }

    let mut profile = if replace_all { IdentityProfile::default() } else { stored };
    profile.schema_version = SCHEMA_VERSION.into();
    let (mut added, mut updated) = (0usize, 0usize);
    for t in traits {
        match profile.traits.iter_mut().find(|e| e.id == t.id) {
            Some(existing) => {
                *existing = t;
                updated += 1;
            }
            None => {
                profile.traits.push(t);
                added += 1;
            }
        }
    }
    if let Some(n) = notes {
        profile.notes = n;
    }

    let root = redesign_dir(ws);
    redesign_store::ensure(ws)?;
    let path = redesign_store::safe_child(&root, IDENTITY_FILE)?;
    redesign_store::write_record(&path, &profile)?;
    let env = redesign_store::bump(ws)?;

    let known: HashSet<String> = redesign_store::list_evidence(ws)?.into_iter().map(|e| e.id).collect();
    let mut unknown_evidence: Vec<&str> =
        profile.traits.iter().flat_map(|t| t.evidence_ids.iter()).filter(|id| !known.contains(*id)).map(String::as_str).collect();
    unknown_evidence.sort_unstable();
    unknown_evidence.dedup();
    let without_evidence: Vec<&str> = profile.traits.iter().filter(|t| t.evidence_ids.is_empty()).map(|t| t.id.as_str()).collect();

    Ok(json!({
        "path": format!("{}/{IDENTITY_FILE}", redesign_store::DIR),
        "revision": env.revision,
        "replace_all": replace_all,
        "added": added,
        "updated": updated,
        "total": profile.traits.len(),
        "counts": counts(&profile),
        "traits_without_evidence": without_evidence,
        "unknown_evidence_ids": unknown_evidence,
        "missing_evolution": missing_evolution(&profile),
        "constraints": identity_constraints(&profile),
    }))
}

fn counts(profile: &IdentityProfile) -> Value {
    let intent: serde_json::Map<String, Value> =
        TraitIntent::ALL.iter().map(|i| (i.as_str().to_string(), json!(profile.traits.iter().filter(|t| t.intent == *i).count()))).collect();
    let decision: serde_json::Map<String, Value> =
        TraitDecision::ALL.iter().map(|d| (d.as_str().to_string(), json!(profile.traits.iter().filter(|t| t.decision == *d).count()))).collect();
    let effective_keep = profile.traits.iter().filter(|t| t.effective_decision() == TraitDecision::Keep).count();
    json!({ "intent": intent, "decision": decision, "effective_keep": effective_keep })
}

/// Profile, counts and builder constraints; a missing file is an empty profile, not an error.
pub fn show(ws: &Workspace) -> Result<Value, ToolError> {
    let (profile, recorded) = match load(ws)? {
        Some(p) => (p, true),
        None => {
            let mut p = IdentityProfile::default();
            p.notes.push("No identity traits were recorded for this project; record them with redesign_identity_record.".into());
            (p, false)
        }
    };
    let missing = missing_evolution(&profile);
    let mut summary_notes = Vec::new();
    if !missing.is_empty() {
        summary_notes.push(MISSING_EVOLUTION_NOTE);
    }
    Ok(json!({
        "path": format!("{}/{IDENTITY_FILE}", redesign_store::DIR),
        "recorded": recorded,
        "missing_evolution": missing,
        "summary_notes": summary_notes,
        "counts": counts(&profile),
        "constraints": identity_constraints(&profile),
        "profile": profile,
    }))
}

fn checks_hint(c: &TraitChecks) -> Option<String> {
    let mut parts = Vec::new();
    if !c.font_families_any.is_empty() {
        parts.push(format!("use one of the fonts {}", c.font_families_any.join(", ")));
    }
    if !c.forbidden_font_families.is_empty() {
        parts.push(format!("do not use the fonts {}", c.forbidden_font_families.join(", ")));
    }
    if let Some(r) = c.max_radius_px {
        parts.push(format!("corner radius at most {r}px"));
    }
    if let Some(s) = c.max_shadows {
        parts.push(format!("at most {s} box-shadow declarations"));
    }
    if !c.required_colors.is_empty() {
        parts.push(format!("keep the colours {}", c.required_colors.join(", ")));
    }
    (!parts.is_empty()).then(|| format!(" (checked: {})", parts.join("; ")))
}

/// Plain-English lines for a builder: the style-not-structure rule, then one line per trait.
pub fn identity_constraints(profile: &IdentityProfile) -> Vec<String> {
    if profile.traits.is_empty() {
        return Vec::new();
    }
    let lines = profile.traits.iter().map(|t| {
        let why = t.rationale_text();
        match t.effective_decision() {
            TraitDecision::Keep => {
                let head = if t.intent == TraitIntent::Intentional { "Keep (intentional)" } else { "Keep" };
                let evolve = t.evolve_text().map(|e| format!("; evolve within: {e}")).unwrap_or_default();
                format!("{head}: {} — {}{}{evolve}", t.label, t.observation, checks_hint(&t.checks).unwrap_or_default())
            }
            TraitDecision::Evolve => {
                let how = [why, t.evolve_text()].into_iter().flatten().map(|s| format!("; {s}")).collect::<String>();
                format!("Evolve: {} — {}{how}", t.label, t.observation)
            }
            TraitDecision::Replace => {
                let head = match t.approval_id() {
                    Some(a) if t.intent == TraitIntent::Intentional => format!("Replace (operator approved, {a})"),
                    _ => "Replace".to_string(),
                };
                format!("{head}: {} — {}", t.label, why.unwrap_or(&t.observation))
            }
            TraitDecision::Undecided => format!("Undecided: {} — {}", t.label, t.observation),
        }
    });
    std::iter::once(STYLE_NOT_STRUCTURE.to_string()).chain(lines).collect()
}

/// App text `identity_drift` scans: the quality runners' sources plus top-level stylesheets.
fn identity_sources(ws: &Workspace) -> String {
    let mut out = super::redesign_quality::read_app_blob(ws);
    let src = ws.app_dir().join("src");
    for dir in [src.clone(), src.join("styles")] {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        let mut paths: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().and_then(|e| e.to_str()) == Some("css")).collect();
        paths.sort();
        for p in paths {
            if let Ok(s) = fs::read_to_string(&p) {
                out.push_str(&s);
                out.push('\n');
            }
        }
    }
    out
}

/// `(property, value)` for every `prop: value` whose property `wanted` accepts.
/// Values end at `;`, `}` or a newline. Works on CSS and on JS style objects.
fn property_values(lower: &str, wanted: impl Fn(&str) -> bool) -> Vec<(&str, &str)> {
    const MAX_VALUE: usize = 240;
    let bytes = lower.as_bytes();
    let mut out = Vec::new();
    for (i, _) in lower.match_indices(':') {
        let mut end = i;
        while end > 0 && matches!(bytes[end - 1], b' ' | b'\t' | b'"' | b'\'') {
            end -= 1;
        }
        let mut start = end;
        while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || matches!(bytes[start - 1], b'-' | b'_')) {
            start -= 1;
        }
        if start == end || !wanted(&lower[start..end]) {
            continue;
        }
        let rest = &lower[i + 1..];
        let mut cut = rest.find([';', '}', '\n']).unwrap_or(rest.len()).min(MAX_VALUE);
        while !rest.is_char_boundary(cut) {
            cut -= 1;
        }
        out.push((&lower[start..end], &rest[..cut]));
    }
    out
}

/// Font declarations and font imports, lowercased.
fn font_mentions(lower: &str) -> Vec<&str> {
    let mut out: Vec<&str> = property_values(lower, |p| p == "font" || p == "fontfamily" || p.contains("font-family") || p.starts_with("--font"))
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    out.extend(lower.lines().filter(|l| ["fonts.googleapis.com", "fonts.bunny.net", "@fontsource", "use.typekit.net"].iter().any(|m| l.contains(m))));
    out
}

fn contains_word(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    hay.match_indices(needle).any(|(i, _)| {
        let before = hay[..i].chars().next_back();
        let after = hay[i + needle.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric()) && !after.is_some_and(|c| c.is_ascii_alphanumeric())
    })
}

fn mentions_family(mentions: &[&str], family: &str) -> bool {
    let f = family.trim().to_ascii_lowercase();
    let variants = [f.clone(), f.replace(' ', "+"), f.replace(' ', "-")];
    mentions.iter().any(|m| variants.iter().any(|v| contains_word(m, v)))
}

/// Radius values in px. `rem`/`em` count as 16px; a bare number counts as px in JS style objects.
fn radius_values_px(lower: &str) -> Vec<f64> {
    let mut out = Vec::new();
    for (prop, value) in property_values(lower, |p| p.contains("radius") && (p.starts_with("border") || p.starts_with("--"))) {
        let js_object = !prop.contains('-');
        let value = if js_object { value.split(',').next().unwrap_or("") } else { value };
        let b = value.as_bytes();
        let mut i = 0;
        while i < b.len() {
            let digit_here = b[i].is_ascii_digit() || (b[i] == b'.' && b.get(i + 1).is_some_and(u8::is_ascii_digit));
            let starts_number = digit_here && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || matches!(b[i - 1], b'-' | b'#' | b'.')));
            if !starts_number {
                i += 1;
                continue;
            }
            let start = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
                i += 1;
            }
            let Ok(n) = value[start..i].parse::<f64>() else { continue };
            let unit_end = (i..b.len()).find(|&j| !b[j].is_ascii_alphabetic()).unwrap_or(b.len());
            match &value[i..unit_end] {
                "px" => out.push(n),
                "rem" | "em" => out.push(n * 16.0),
                "" if js_object && !value[i..].starts_with('%') => out.push(n),
                _ => {}
            }
            i = unit_end;
        }
    }
    out
}

fn shadow_count(lower: &str) -> usize {
    property_values(lower, |p| p == "box-shadow" || p == "boxshadow")
        .into_iter()
        .filter(|(_, v)| {
            let v = v.replace("!important", "");
            let v = v.trim().trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c.is_whitespace());
            !v.is_empty() && v != "none"
        })
        .count()
}

fn contains_hex(lower: &str, color: &str) -> bool {
    let c = color.to_ascii_lowercase();
    let mut variants = vec![c.clone()];
    let h: Vec<char> = c.trim_start_matches('#').chars().collect();
    if h.len() == 3 {
        variants.push(format!("#{0}{0}{1}{1}{2}{2}", h[0], h[1], h[2]));
    } else if h.len() == 6 && h[0] == h[1] && h[2] == h[3] && h[4] == h[5] {
        variants.push(format!("#{}{}{}", h[0], h[2], h[4]));
    }
    variants.iter().any(|v| lower.match_indices(v.as_str()).any(|(i, _)| !lower[i + v.len()..].chars().next().is_some_and(|ch| ch.is_ascii_hexdigit())))
}

/// Which checks of one trait drift from `lower` (lowercased app text).
fn trait_drifts(t: &IdentityTrait, lower: &str) -> Vec<Value> {
    let c = &t.checks;
    let mut drifts = Vec::new();
    let fonts = font_mentions(lower);
    if !c.font_families_any.is_empty() && !c.font_families_any.iter().any(|f| mentions_family(&fonts, f)) {
        drifts.push(json!({"check": "font_families_any", "detail": format!("none of {:?} is declared or imported", c.font_families_any)}));
    }
    let forbidden: Vec<&String> = c.forbidden_font_families.iter().filter(|f| mentions_family(&fonts, f)).collect();
    if !forbidden.is_empty() {
        drifts.push(json!({"check": "forbidden_font_families", "detail": format!("forbidden fonts in use: {forbidden:?}")}));
    }
    if let Some(max) = c.max_radius_px {
        let over: Vec<f64> = radius_values_px(lower).into_iter().filter(|r| *r > f64::from(max)).collect();
        if let Some(largest) = over.iter().copied().reduce(f64::max) {
            drifts.push(json!({"check": "max_radius_px", "detail": format!("{} radius value(s) above {max}px, largest {largest}px", over.len())}));
        }
    }
    if let Some(max) = c.max_shadows {
        let n = shadow_count(lower);
        if n > max as usize {
            drifts.push(json!({"check": "max_shadows", "detail": format!("{n} box-shadow declarations, at most {max} allowed")}));
        }
    }
    let missing: Vec<&String> = c.required_colors.iter().filter(|col| !contains_hex(lower, col)).collect();
    if !missing.is_empty() {
        drifts.push(json!({"check": "required_colors", "detail": format!("missing colours: {missing:?}")}));
    }
    drifts
}

/// Evaluate kept traits with checks against app text. Never passes when nothing was checked.
/// Drift on an intentional trait fails; on any other kept trait it warns.
pub fn evaluate_drift(profile: Option<&IdentityProfile>, app_text: &str) -> CheckResult {
    const ID: &str = "identity_drift";
    let limitations = json!([
        "Static scan of app sources; utility classes (e.g. Tailwind) and computed styles are not evaluated.",
        "Drift on an intentional trait blocks Prototype Ready; drift on another kept trait is a warning."
    ]);
    let not_evaluated = |summary: &str| CheckResult {
        id: ID.into(),
        status: "not_evaluated".into(),
        summary: summary.into(),
        details: json!({"advisory": true, "limitations": limitations}),
    };
    let Some(profile) = profile else {
        return not_evaluated("no identity profile recorded");
    };
    let checkable: Vec<&IdentityTrait> = profile.traits.iter().filter(|t| t.effective_decision() == TraitDecision::Keep && !t.checks.is_empty()).collect();
    if checkable.is_empty() {
        return not_evaluated("no kept identity trait has checks");
    }
    if app_text.trim().is_empty() {
        return not_evaluated("no app sources to check");
    }
    let lower = app_text.to_ascii_lowercase();
    let mut drifted = Vec::new();
    let mut essence_drifted = false;
    for t in &checkable {
        for d in trait_drifts(t, &lower) {
            essence_drifted |= t.intent == TraitIntent::Intentional;
            drifted.push(json!({"trait_id": t.id, "label": t.label, "intent": t.intent.as_str(), "check": d["check"], "detail": d["detail"]}));
        }
    }
    let checked: Vec<&str> = checkable.iter().map(|t| t.id.as_str()).collect();
    let (status, summary) = if drifted.is_empty() {
        ("pass", format!("{} kept identity trait(s) hold", checked.len()))
    } else {
        let mut ids: Vec<&str> = drifted.iter().filter_map(|d| d["trait_id"].as_str()).collect();
        ids.dedup();
        (if essence_drifted { "fail" } else { "warn" }, format!("identity drift in {}", ids.join(", ")))
    };
    CheckResult {
        id: ID.into(),
        status: status.into(),
        summary,
        details: json!({"advisory": !essence_drifted, "checked_traits": checked, "drifted": drifted, "limitations": limitations}),
    }
}

/// `identity_essence`: is the target's essence still in the plan? Fails when an intentional trait
/// is marked for replacement without the operator's approval, or the profile can't be read.
pub fn essence_check(ws: &Workspace) -> CheckResult {
    const ID: &str = "identity_essence";
    let row = |status: &str, summary: String, details: Value| CheckResult { id: ID.into(), status: status.into(), summary, details };
    let profile = match load(ws) {
        Ok(p) => p,
        Err(e) => return row("fail", format!("identity profile could not be read, so the essence is unknown: {}", e.message), json!({"unreadable": true})),
    };
    let intentional: Vec<&IdentityTrait> = profile.iter().flat_map(|p| &p.traits).filter(|t| t.intent == TraitIntent::Intentional).collect();
    if intentional.is_empty() {
        return row("not_evaluated", "no intentional identity trait recorded, so there is no essence to check".into(), json!({}));
    }
    let (mut approved, mut unapproved) = (Vec::new(), Vec::new());
    for t in intentional.iter().filter(|t| t.replaces_essence()) {
        match approved_by_operator(ws, t.approval_id()) {
            Ok(true) => approved.push(t.id.as_str()),
            Ok(false) => unapproved.push(t.id.as_str()),
            Err(e) => return row("fail", format!("approval for {} could not be read: {}", t.id, e.message), json!({"unreadable": true})),
        }
    }
    let details = json!({
        "intentional_traits": intentional.len(),
        "replaced_with_approval": approved,
        "replaced_without_approval": unapproved,
        "note": "An approval is an evidence record with source \"user\": an audit trail, not proof of who wrote it.",
    });
    if unapproved.is_empty() {
        row(
            "pass",
            format!("{} intentional trait(s) kept or evolved; {} replaced with the operator's approval", intentional.len() - approved.len(), approved.len()),
            details,
        )
    } else {
        row("fail", format!("the target's essence would be lost: {} marked for replacement without the operator's approval", unapproved.join(", ")), details)
    }
}

/// `identity_drift` for a project. A profile that can't be read is reported, not passed.
pub fn drift_check(ws: &Workspace) -> CheckResult {
    match load(ws) {
        Ok(profile) => evaluate_drift(profile.as_ref(), &identity_sources(ws)),
        Err(e) => CheckResult {
            id: "identity_drift".into(),
            status: "not_evaluated".into(),
            summary: format!("identity profile could not be read: {}", e.message),
            details: json!({"advisory": true}),
        },
    }
}

/// Whether keeping the character still leaves room for evolution. Advisory, like `identity_drift`.
pub fn evaluate_evolution(profile: Option<&IdentityProfile>) -> CheckResult {
    const ID: &str = "identity_evolution";
    let advisory_note = "Advisory: identity evolution never changes concept/prototype/engineering Ready.";
    let not_evaluated = |summary: &str| CheckResult {
        id: ID.into(),
        status: "not_evaluated".into(),
        summary: summary.into(),
        details: json!({"advisory": true, "note": advisory_note}),
    };
    let Some(profile) = profile else {
        return not_evaluated("no identity profile recorded");
    };
    let changing: Vec<&str> =
        profile.traits.iter().filter(|t| matches!(t.effective_decision(), TraitDecision::Evolve | TraitDecision::Replace)).map(|t| t.id.as_str()).collect();
    let kept: Vec<&IdentityTrait> = profile.traits.iter().filter(|t| t.is_kept()).collect();
    if kept.is_empty() && changing.is_empty() {
        return not_evaluated("no identity trait is kept, evolved or replaced yet");
    }
    let kept_with_evolution: Vec<&str> = kept.iter().filter(|t| t.evolve_text().is_some()).map(|t| t.id.as_str()).collect();
    let frozen = !kept.is_empty() && changing.is_empty() && kept_with_evolution.is_empty();
    let (status, summary) = if frozen {
        ("warn", "character preserved but no evolution proposed".to_string())
    } else {
        ("pass", format!("{} trait(s) evolve or are replaced; {} kept trait(s) say how they evolve", changing.len(), kept_with_evolution.len()))
    };
    CheckResult {
        id: ID.into(),
        status: status.into(),
        summary,
        details: json!({
            "advisory": true,
            "note": advisory_note,
            "evolve_or_replace": changing,
            "kept_with_evolve_within": kept_with_evolution,
            "missing_evolution": missing_evolution(profile),
        }),
    }
}

/// `identity_evolution` for a project. A profile that can't be read is reported, not passed.
pub fn evolution_check(ws: &Workspace) -> CheckResult {
    match load(ws) {
        Ok(profile) => evaluate_evolution(profile.as_ref()),
        Err(e) => CheckResult {
            id: "identity_evolution".into(),
            status: "not_evaluated".into(),
            summary: format!("identity profile could not be read: {}", e.message),
            details: json!({"advisory": true}),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::TargetRef;

    fn ws() -> (tempfile::TempDir, Workspace) {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "Archive", "kind=redesign", TargetRef::None).unwrap();
        (d, ws)
    }

    fn archival_chrome(intent: TraitIntent, decision: TraitDecision) -> IdentityTrait {
        IdentityTrait {
            id: "trait_archival_chrome".into(),
            label: "Archival library chrome".into(),
            category: TraitCategory::Chrome,
            observation: "Square corners, serif headings and a dark header bar read as an old catalogue.".into(),
            intent,
            decision,
            rationale: None,
            approval_evidence_id: None,
            evolve_within: None,
            evidence_ids: vec![],
            checks: TraitChecks { font_families_any: vec!["Georgia".into()], max_radius_px: Some(2), ..Default::default() },
        }
    }

    fn write_app(ws: &Workspace, css: &str) {
        let src = ws.app_dir().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("App.tsx"), "import './index.css';\nexport function App(){ return <button>Search</button> }\n").unwrap();
        fs::write(src.join("index.css"), css).unwrap();
    }

    fn evidence(ws: &Workspace, id: &str, source: &str) {
        redesign_store::put_evidence(
            ws,
            redesign_store::EvidenceRecord {
                id: id.into(),
                source: source.into(),
                source_operation: "answer".into(),
                subject: "identity".into(),
                kind: "approval".into(),
                status: "declared".into(),
                scope: Value::Null,
                locator: None,
                digest: None,
                observed_at: "0Z".into(),
                limitations: vec![],
                sensitivity: "public".into(),
                summary: "Approved a rebrand of the catalogue chrome.".into(),
            },
        )
        .unwrap();
    }

    fn rebrand(approval: Option<&str>) -> IdentityTrait {
        let mut t = archival_chrome(TraitIntent::Intentional, TraitDecision::Replace);
        t.rationale = Some("New catalogue brand.".into());
        t.approval_evidence_id = approval.map(Into::into);
        t
    }

    #[test]
    fn replacing_an_intentional_trait_needs_a_rationale_and_the_operators_recorded_approval() {
        let (_d, ws) = ws();
        let no_reason = record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Replace)], false, None).unwrap_err();
        assert_eq!(no_reason.code, ErrorCode::BadInput, "{no_reason:?}");

        evidence(&ws, "ev_tool_note", "craft");
        for refused in [None, Some("ev_missing"), Some("ev_tool_note")] {
            let err = record(&ws, vec![rebrand(refused)], false, None).unwrap_err();
            assert_eq!(err.details.as_ref().map(|d| &d["code"]), Some(&json!("approval_required")), "{refused:?}: {err:?}");
        }
        assert!(load(&ws).unwrap().is_none(), "a refused record writes nothing");

        evidence(&ws, "ev_operator_ok", "user");
        record(&ws, vec![rebrand(Some("ev_operator_ok"))], false, None).unwrap();
        let saved = load(&ws).unwrap().unwrap();
        assert!(identity_constraints(&saved).iter().any(|c| c.starts_with("Replace (operator approved")), "{saved:?}");
        assert_eq!(essence_check(&ws).status, "pass");
    }

    #[test]
    fn relabelling_or_dropping_an_intentional_trait_cannot_bypass_the_approval() {
        let (_d, ws) = ws();
        record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Keep)], false, None).unwrap();

        let mut relabelled = archival_chrome(TraitIntent::Incidental, TraitDecision::Replace);
        relabelled.rationale = Some("It was never deliberate.".into());
        let err = record(&ws, vec![relabelled.clone()], false, None).unwrap_err();
        assert_eq!(err.details.as_ref().map(|d| &d["code"]), Some(&json!("approval_required")), "{err:?}");
        let dropped = record(&ws, vec![], true, None).unwrap_err();
        assert_eq!(dropped.details.as_ref().map(|d| &d["code"]), Some(&json!("approval_required")), "{dropped:?}");
        assert!(load(&ws).unwrap().unwrap().traits.iter().any(|t| t.intent == TraitIntent::Intentional), "still the essence");

        evidence(&ws, "ev_operator_ok", "user");
        relabelled.approval_evidence_id = Some("ev_operator_ok".into());
        record(&ws, vec![relabelled], false, None).unwrap();
    }

    #[test]
    fn a_replacement_written_without_approval_still_counts_as_kept_and_fails_the_essence_check() {
        let (_d, ws) = ws();
        // Written past `record`, as an older version or a hand edit would.
        let profile = IdentityProfile { traits: vec![rebrand(None)], ..Default::default() };
        redesign_store::ensure(&ws).unwrap();
        redesign_store::write_record(&redesign_dir(&ws).join(IDENTITY_FILE), &profile).unwrap();

        assert_eq!(profile.traits[0].effective_decision(), TraitDecision::Keep);
        assert!(identity_constraints(&profile).iter().any(|c| c.starts_with("Keep (intentional)")), "the builder is told to keep it");
        let essence = essence_check(&ws);
        assert_eq!(essence.status, "fail", "{essence:?}");
    }

    #[test]
    fn drift_from_an_intentional_trait_fails_a_match_passes_and_nothing_checked_never_passes() {
        let (_d, ws) = ws();
        assert_eq!(drift_check(&ws).status, "not_evaluated", "no profile");

        // Intentional + undecided counts as keep.
        record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Undecided)], false, None).unwrap();
        write_app(&ws, ".card { border-radius: 12px; font-family: Inter, sans-serif; }\n");
        let drift = drift_check(&ws);
        assert_eq!(drift.status, "fail", "drift from the essence: {drift:?}");
        let drifted = drift.details["drifted"].as_array().unwrap();
        for check in ["max_radius_px", "font_families_any"] {
            assert!(drifted.iter().any(|d| d["trait_id"] == "trait_archival_chrome" && d["check"] == check), "{check}: {drift:?}");
        }

        write_app(&ws, ".card { border-radius: 2px; font-family: Georgia, serif; }\n");
        assert_eq!(drift_check(&ws).status, "pass");
    }

    #[test]
    fn drift_on_a_kept_trait_that_is_not_intentional_only_warns_and_a_replaced_one_is_not_checked() {
        let (_d, ws) = ws();
        write_app(&ws, ".card { border-radius: 12px; font-family: Inter, sans-serif; }\n");
        record(&ws, vec![archival_chrome(TraitIntent::Incidental, TraitDecision::Keep)], false, None).unwrap();
        assert_eq!(drift_check(&ws).status, "warn");

        let mut replaced = archival_chrome(TraitIntent::Incidental, TraitDecision::Replace);
        replaced.rationale = Some("New brand".into());
        record(&ws, vec![replaced], true, None).unwrap();
        assert_eq!(drift_check(&ws).status, "not_evaluated", "nothing is claimed about a replaced trait");
    }

    #[test]
    fn keeping_the_look_without_a_way_forward_is_flagged_until_evolution_is_proposed() {
        let (_d, ws) = ws();
        assert_eq!(evolution_check(&ws).status, "not_evaluated", "no profile");

        // Intentional + undecided is kept; with no evolve_within the profile is frozen.
        record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Undecided)], false, None).unwrap();
        let shown = show(&ws).unwrap();
        assert!(shown["missing_evolution"].as_array().unwrap().iter().any(|id| id == "trait_archival_chrome"), "{shown}");
        assert!(!shown["summary_notes"].as_array().unwrap().is_empty(), "{shown}");
        assert_eq!(evolution_check(&ws).status, "warn");

        let mut with_way_forward = archival_chrome(TraitIntent::Intentional, TraitDecision::Keep);
        with_way_forward.evolve_within = Some("Keep the dark bar; media nav maps straight to Wayback and Library with a visible active state.".into());
        record(&ws, vec![with_way_forward], false, None).unwrap();
        let shown = show(&ws).unwrap();
        assert!(!shown["missing_evolution"].as_array().unwrap().iter().any(|id| id == "trait_archival_chrome"), "{shown}");
        let evolution = evolution_check(&ws);
        assert_eq!(evolution.status, "pass", "{evolution:?}");
        assert!(evolution.details["kept_with_evolve_within"].as_array().unwrap().iter().any(|id| id == "trait_archival_chrome"));
        let lines = identity_constraints(&load(&ws).unwrap().unwrap());
        assert!(lines.first().is_some_and(|l| l.contains("not structure")), "{lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("Keep (intentional)") && l.contains("evolve within:")), "{lines:?}");

        // Frozen again, then one evolving trait is enough.
        let mut evolving = archival_chrome(TraitIntent::Incidental, TraitDecision::Evolve);
        evolving.id = "trait_result_density".into();
        record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Keep)], true, None).unwrap();
        assert_eq!(evolution_check(&ws).status, "warn");
        record(&ws, vec![evolving], false, None).unwrap();
        assert_eq!(evolution_check(&ws).status, "pass");
    }

    #[test]
    fn losing_the_essence_blocks_ready_and_a_frozen_character_does_not() {
        let (_d, mut ws) = ws();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let ds = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/design-system").canonicalize().unwrap();
        super::super::redesign_ds::ingest_package(&ws, &ds, "advisory", false).unwrap();
        super::super::redesign_build::scaffold_app(&ws, None).unwrap();
        let css = ws.app_dir().join("src/index.css");
        fs::write(&css, ".card { border-radius: 2px; font-family: Georgia, serif; }\n").unwrap();
        let run = || super::super::redesign_checks::run_native_checks(&ws).unwrap();
        let before = run();
        assert!(before.concept_ready, "the scaffold is a concept: {:?}", before.checks);

        // Kept and matching, but with no way forward: a warning, and the gates stay where they were.
        record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Keep)], false, None).unwrap();
        let frozen = run();
        assert!(frozen.checks.iter().any(|c| c.id == "identity_evolution" && c.status == "warn"), "{:?}", frozen.checks);
        assert_eq!((before.concept_ready, before.prototype_ready), (frozen.concept_ready, frozen.prototype_ready));

        // A replacement nobody approved loses the essence in the plan: no gate holds.
        redesign_store::write_record(&redesign_dir(&ws).join(IDENTITY_FILE), &IdentityProfile { traits: vec![rebrand(None)], ..Default::default() }).unwrap();
        let lost = run();
        assert!(!lost.concept_ready && !lost.prototype_ready && !lost.engineering_ready, "{:?}", lost.checks);
    }

    #[test]
    fn an_app_that_drifts_from_the_essence_is_a_concept_but_never_a_ready_prototype() {
        let inputs = |essence_drift| super::super::redesign_checks::CheckInputs {
            preview_available: true,
            security_ok: true,
            build_status: super::super::redesign_checks::CheckExec::Pass,
            ds_status: super::super::redesign_checks::CheckExec::Pass,
            functional: super::super::redesign_checks::FunctionalSummary { critical_scenarios_total: 1, critical_scenarios_passed: 1, button_works: true },
            essence_drift,
            ..Default::default()
        };
        use super::super::redesign_checks::{concept_ready, prototype_ready};
        assert!(prototype_ready(&inputs(false), 0), "the same prototype without drift is ready");
        assert!(concept_ready(&inputs(true)) && !prototype_ready(&inputs(true), 0));
    }
}
