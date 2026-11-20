//! Visual identity traits: which look-and-feel traits of the original are intentional, and
//! whether the redesign keeps, evolves or replaces them. Stored at `redesign/identity.json`.
//!
//! Traits constrain style by context, not structure: a kept trait still evolves (`evolve_within`).
//! An intentional trait left `undecided` counts as `keep` everywhere. Replacing an intentional
//! trait needs a rationale. `identity_drift` and `identity_evolution` are advisory: they never
//! feed the Ready gates.

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
    /// The decision the redesign acts on: an intentional, undecided trait is kept.
    pub fn effective_decision(&self) -> TraitDecision {
        match (self.intent, self.decision) {
            (TraitIntent::Intentional, TraitDecision::Undecided) => TraitDecision::Keep,
            (_, d) => d,
        }
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
    if t.intent == TraitIntent::Intentional && t.decision == TraitDecision::Replace && t.rationale_text().is_none() {
        return Err(ToolError::new(ErrorCode::BadInput, format!("trait {}: replacing an intentional trait needs a rationale", t.id))
            .with_details(json!({"code": "rationale_required", "trait_id": t.id})));
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
    for n in notes.iter().flatten() {
        redesign_store::refuse_secretish("identity note", n)?;
    }

    let mut profile = if replace_all { IdentityProfile::default() } else { load(ws)?.unwrap_or_default() };
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
            TraitDecision::Replace => format!("Replace: {} — {}", t.label, why.unwrap_or(&t.observation)),
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
pub fn evaluate_drift(profile: Option<&IdentityProfile>, app_text: &str) -> CheckResult {
    const ID: &str = "identity_drift";
    let limitations = json!([
        "Static scan of app sources; utility classes (e.g. Tailwind) and computed styles are not evaluated.",
        "Advisory: identity drift never changes concept/prototype/engineering Ready."
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
    for t in &checkable {
        for d in trait_drifts(t, &lower) {
            drifted.push(json!({"trait_id": t.id, "label": t.label, "check": d["check"], "detail": d["detail"]}));
        }
    }
    let checked: Vec<&str> = checkable.iter().map(|t| t.id.as_str()).collect();
    let (status, summary) = if drifted.is_empty() {
        ("pass", format!("{} kept identity trait(s) hold", checked.len()))
    } else {
        let mut ids: Vec<&str> = drifted.iter().filter_map(|d| d["trait_id"].as_str()).collect();
        ids.dedup();
        ("warn", format!("identity drift in {}", ids.join(", ")))
    };
    CheckResult {
        id: ID.into(),
        status: status.into(),
        summary,
        details: json!({"advisory": true, "checked_traits": checked, "drifted": drifted, "limitations": limitations}),
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

    #[test]
    fn replacing_an_intentional_trait_needs_a_rationale() {
        let (_d, ws) = ws();
        let err = record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Replace)], false, None).unwrap_err();
        assert_eq!(err.code, ErrorCode::BadInput, "{err:?}");
        assert!(load(&ws).unwrap().is_none(), "a refused record writes nothing");

        let mut with_reason = archival_chrome(TraitIntent::Intentional, TraitDecision::Replace);
        with_reason.rationale = Some("Operator approved a rebrand for the new catalogue.".into());
        record(&ws, vec![with_reason], false, None).unwrap();
        let saved = load(&ws).unwrap().unwrap();
        assert!(saved.traits.iter().any(|t| t.id == "trait_archival_chrome" && t.decision == TraitDecision::Replace));
        assert!(identity_constraints(&saved).iter().any(|c| c.starts_with("Replace:") && c.contains("rebrand")));
    }

    #[test]
    fn identity_drift_warns_on_drift_passes_on_match_and_never_passes_unchecked() {
        let (_d, ws) = ws();
        assert_eq!(drift_check(&ws).status, "not_evaluated", "no profile");

        // Intentional + undecided counts as keep.
        record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Undecided)], false, None).unwrap();
        write_app(&ws, ".card { border-radius: 12px; font-family: Inter, sans-serif; }\n");
        let drift = drift_check(&ws);
        assert_eq!(drift.status, "warn", "{drift:?}");
        let drifted = drift.details["drifted"].as_array().unwrap();
        for check in ["max_radius_px", "font_families_any"] {
            assert!(drifted.iter().any(|d| d["trait_id"] == "trait_archival_chrome" && d["check"] == check), "{check}: {drift:?}");
        }

        write_app(&ws, ".card { border-radius: 2px; font-family: Georgia, serif; }\n");
        assert_eq!(drift_check(&ws).status, "pass");

        // A trait marked for replacement is not checked, so nothing is claimed.
        let mut replaced = archival_chrome(TraitIntent::Incidental, TraitDecision::Replace);
        replaced.rationale = Some("New brand".into());
        record(&ws, vec![replaced], true, None).unwrap();
        assert_eq!(drift_check(&ws).status, "not_evaluated");
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
    fn identity_warnings_do_not_change_ready_gates() {
        let (_d, mut ws) = ws();
        ws.manifest.kind = Some("redesign".into());
        ws.save().unwrap();
        let ds = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../REDESIGN_SPEC/fixtures/design-system").canonicalize().unwrap();
        super::super::redesign_ds::ingest_package(&ws, &ds, "advisory", false).unwrap();
        super::super::redesign_build::scaffold_app(&ws, None).unwrap();
        fs::write(ws.app_dir().join("src/index.css"), ".card { border-radius: 24px; font-family: Inter; }\n").unwrap();
        let before = super::super::redesign_checks::run_native_checks(&ws).unwrap();

        record(&ws, vec![archival_chrome(TraitIntent::Intentional, TraitDecision::Keep)], false, None).unwrap();
        let after = super::super::redesign_checks::run_native_checks(&ws).unwrap();
        assert!(after.checks.iter().any(|c| c.id == "identity_drift" && c.status == "warn"), "{:?}", after.checks);
        assert!(after.checks.iter().any(|c| c.id == "identity_evolution" && c.status == "warn"), "frozen profile: {:?}", after.checks);
        assert_eq!(
            (before.concept_ready, before.prototype_ready, before.engineering_ready),
            (after.concept_ready, after.prototype_ready, after.engineering_ready)
        );
    }
}
