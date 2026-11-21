//! `web_site_journey_run`: do what a visitor would on the open website, one step at a time
//! (click a control by its name, type into a field by its label, press a key, open an address on
//! the site, expect a text to show), and say which step failed. A click is a real pointer click at
//! the control's centre, so a control that is covered or disabled does nothing, as it would for a
//! person.
//!
//! Data minimisation: the text a step types is the caller's own; nothing a field already held is
//! read back. A `value_of` check compares a field with the caller's text inside the page and
//! returns only whether they match.

use super::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Step {
    /// Click the control with this visible or accessible name (a button, link, tab, option…).
    Click { click: String },
    /// Type this text into the field with this label, placeholder or name, replacing what it held.
    /// For a drop-down, choose the option with this text; a date, time, colour or range field is
    /// set to it (a date written 2026-11-20).
    Type {
        #[serde(rename = "type")]
        text: String,
        into: String,
    },
    /// Press a key: Enter, Escape, Tab, Space, ArrowUp, ArrowDown, ArrowLeft or ArrowRight.
    Press { press: String },
    /// Load this address on the open site again from scratch, e.g. "/#/library".
    Open { open: String },
    /// This text must show on the page (case and spacing ignored); waits a moment for it.
    Expect { expect: String },
    /// The control or field with this name is in this state; waits a moment for it.
    Check { check: String, is: ControlState },
    /// The field with this name holds this text (case and spacing ignored); waits a moment for it.
    Value { value_of: String, equals: String },
    /// An alert or live region (role alert, alertdialog or status, aria-live) shows this text;
    /// an empty text means any text. Waits a moment for it.
    Alert { alert: String },
}

/// The state a `check` step expects a control or field to be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlState {
    /// It shows and can be used.
    Enabled,
    /// It shows but can't be used (disabled, aria-disabled or inert).
    Disabled,
    /// It shows.
    Present,
    /// Nothing with that name shows.
    Absent,
    /// It can't be used: disabled, or not shown at all.
    Unusable,
}

impl ControlState {
    fn word(self) -> &'static str {
        match self {
            ControlState::Enabled => "enabled",
            ControlState::Disabled => "disabled",
            ControlState::Present => "present",
            ControlState::Absent => "absent",
            ControlState::Unusable => "unusable",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JourneyParams {
    /// The steps, in order. Each is one of {"click": name}, {"type": text, "into": field},
    /// {"press": key}, {"open": path}, {"expect": text}, {"check": name, "is": "enabled" |
    /// "disabled" | "present" | "absent" | "unusable"}, {"value_of": field, "equals": text} and
    /// {"alert": text}. The run stops at the first that fails.
    pub steps: Vec<Step>,
    /// Page width in CSS pixels (320..=3840, default 1440).
    #[serde(default = "d_state_width")]
    pub width: u32,
    /// Load the address the site was opened at before the first step, so each journey starts
    /// from the same place (default true).
    #[serde(default = "yes")]
    pub restart: bool,
    /// How long an "expect" waits for its text, in milliseconds (100..=30000, default 3000).
    #[serde(default = "d_wait")]
    pub wait_ms: u64,
    /// Absolute path of a folder that does not exist yet: a picture of the page as the journey
    /// left it is written there as end.png. Unset: no picture.
    #[serde(default)]
    pub output_directory: Option<String>,
    #[serde(default = "d_timeout")]
    pub timeout_secs: u64,
}

fn d_wait() -> u64 {
    3000
}

pub(super) fn tool() -> ToolSpec {
    spec(
        "web_site_journey_run",
        "Run a visitor's journey",
        "On the open website, do a visitor's steps in order and say which one failed: {\"click\": name} clicks the control with that visible or accessible name, {\"type\": text, \"into\": field} types into the field with that label, placeholder or name (or chooses that option of a drop-down, or sets a date, time, colour or range field to it, a date written 2026-11-20), {\"press\": key} presses Enter, Escape, Tab, Space or an arrow key, {\"open\": path} loads that address on the same site from scratch, and {\"expect\": text} waits up to wait_ms for that text to show, {\"check\": name, \"is\": state} for that control or field to be enabled, disabled, present, absent or unusable, {\"value_of\": field, \"equals\": text} for that field to hold the text (compared in the page; its content is never read back), and {\"alert\": text} for an alert or live region to announce it (empty: anything). Controls inside open shadow roots are found; ones in frames are not. A click is a real pointer click at the control's centre, so a covered control fails the step. The page is loaded again first unless restart is false. With output_directory, a picture of the page at the end is saved as end.png. Clicking and typing run the site's own code, so it needs your approval.",
        "Click and type through the site the way a visitor would, and check what shows.",
        Toolset::Browser,
        Effects::EVIDENCE.union(Effects::WRITES_FILESYSTEM).union(Effects::EXECUTES_FOREIGN_CODE).union(Effects::ACCESSES_NETWORK),
        Requires::Engine("chromium"),
        schema_of::<JourneyParams>,
        web_site_journey_run,
    )
}

/// Shared by the scripts below: every element, open shadow roots included; a name made
/// comparable; whether an element shows.
pub(super) const DEEP_JS: &str = r#"
  const all = (root, out) => { for (const el of root.querySelectorAll('*')) { out.push(el); if (el.shadowRoot) all(el.shadowRoot, out); } return out; };
  const norm = s => String(s || '').replace(/\s+/g, ' ').trim().toLowerCase();
  const shown = el => { const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; const s = getComputedStyle(el); return s.visibility !== 'hidden' && s.display !== 'none' && (!el.checkVisibility || el.checkVisibility()); };
  const labelled = el => { const root = el.getRootNode(); return (el.getAttribute('aria-labelledby') || '').split(/\s+/).filter(Boolean).map(id => root.getElementById ? root.getElementById(id) : null).filter(Boolean).map(e => e.innerText || e.textContent).join(' '); };
  const up = n => n.parentNode || n.host || null;
  const inside = (inner, outer) => { for (let n = inner; n; n = up(n)) if (n === outer) return true; return false; };
"#;

/// Find the control named `__NAME__`, bring it into view and say where its centre is.
const FIND_CONTROL_JS: &str = r#"(() => {
  __DEEP__
  const want = norm(__NAME__);
  const names = el => [el.getAttribute('aria-label'), labelled(el), el.innerText, el.value && /^(submit|button|reset)$/.test(el.type || '') ? el.value : '', el.getAttribute('title'), el.querySelector('img[alt]') ? el.querySelector('img[alt]').alt : ''].map(norm).filter(Boolean);
  const controls = 'a[href], button, summary, label, select, [role="button"], [role="link"], [role="tab"], [role="menuitem"], [role="option"], [role="checkbox"], [role="radio"], [role="switch"], input[type="submit"], input[type="button"], input[type="reset"], input[type="checkbox"], input[type="radio"], [onclick], [tabindex]';
  const els = all(document, []).filter(shown);
  const named = els.filter(el => el.matches(controls));
  let hits = named.filter(el => names(el).includes(want));
  let how = 'named';
  // Something a script made clickable has no mark of it: take the innermost element with exactly that text.
  if (!hits.length) { hits = els.filter(el => norm(el.innerText) === want).filter((el, _, a) => !a.some(o => o !== el && inside(o, el))); how = 'text'; }
  if (!hits.length) { hits = named.filter(el => names(el).some(n => n.includes(want))); how = 'part of the name'; }
  if (!hits.length) return {found: false};
  const el = hits[0];
  el.scrollIntoView({block: 'center', inline: 'center', behavior: 'instant'});
  const r = el.getBoundingClientRect();
  const x = Math.round(r.left + r.width / 2), y = Math.round(r.top + r.height / 2);
  let top = document.elementFromPoint(x, y);
  while (top && top.shadowRoot) { const deeper = top.shadowRoot.elementFromPoint(x, y); if (!deeper || deeper === top) break; top = deeper; }
  const reached = !!top && (inside(top, el) || inside(el, top));
  return {found: true, x, y, how, same_name: hits.length, kind: el.getAttribute('role') || el.tagName.toLowerCase(), disabled: !!el.disabled || el.getAttribute('aria-disabled') === 'true', reached, covered_by: reached || !top ? null : top.tagName.toLowerCase()};
})()"#;

/// Find the field labelled `__NAME__`, focus it with its content selected, or choose the option
/// `__TEXT__` when it is a drop-down.
const FIND_FIELD_JS: &str = r#"(() => {
  __DEEP__
  const want = norm(__NAME__);
  const names = el => { const root = el.getRootNode(); const forLabel = el.id && root.querySelector ? root.querySelector('label[for="' + CSS.escape(el.id) + '"]') : null; const wrap = el.closest('label');
    return [el.getAttribute('aria-label'), labelled(el), forLabel && forLabel.innerText, wrap && wrap.innerText, el.getAttribute('placeholder'), el.getAttribute('title'), el.getAttribute('name')].map(norm).filter(Boolean); };
  const fields = all(document, []).filter(el => el.matches('input:not([type="hidden"]):not([type="submit"]):not([type="button"]):not([type="reset"]):not([type="checkbox"]):not([type="radio"]):not([type="file"]):not([type="image"]), textarea, select, [contenteditable=""], [contenteditable="true"]')).filter(shown);
  let hits = fields.filter(el => names(el).includes(want));
  if (!hits.length) hits = fields.filter(el => names(el).some(n => n.includes(want)));
  if (!hits.length) return {found: false};
  const el = hits[0];
  if (el.disabled || el.readOnly) return {found: true, locked: true};
  el.scrollIntoView({block: 'center', inline: 'center', behavior: 'instant'});
  if (el.tagName === 'SELECT') {
    const text = norm(__TEXT__);
    const opt = Array.from(el.options).find(o => norm(o.text) === text || norm(o.value) === text);
    if (!opt) return {found: true, kind: 'select', chosen: false};
    el.value = opt.value;
    el.dispatchEvent(new Event('input', {bubbles: true}));
    el.dispatchEvent(new Event('change', {bubbles: true}));
    return {found: true, kind: 'select', chosen: true};
  }
  // A date, time, colour or range field takes no typed characters: set its value the way its
  // picker would, and say whether the field kept it.
  if (el.tagName === 'INPUT' && ['date', 'time', 'datetime-local', 'month', 'week', 'color', 'range'].includes(el.type)) {
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    el.focus();
    set.call(el, __TEXT__);
    el.dispatchEvent(new Event('input', {bubbles: true}));
    el.dispatchEvent(new Event('change', {bubbles: true}));
    return {found: true, kind: 'set', field: el.type, kept: el.value === __TEXT__};
  }
  el.focus();
  if (typeof el.select === 'function') el.select(); else getSelection().selectAllChildren(el);
  return {found: true, kind: 'text', same_name: hits.length};
})()"#;

/// The control or field named `__NAME__` (as a click or a type step would find it, without
/// touching it): whether it shows, whether it can be used, and, when `__COMPARE__` is true,
/// whether its value (or its text) equals `__TEXT__`. Only the comparison leaves the page.
const STATE_OF_JS: &str = r#"(() => {
  __DEEP__
  const want = norm(__NAME__);
  const names = el => { const root = el.getRootNode(); const forLabel = el.id && root.querySelector ? root.querySelector('label[for="' + CSS.escape(el.id) + '"]') : null; const wrap = el.closest('label');
    const field = ['INPUT', 'SELECT', 'TEXTAREA'].includes(el.tagName);
    return [el.getAttribute('aria-label'), labelled(el), forLabel && forLabel.innerText, wrap && wrap.innerText, el.getAttribute('placeholder'), el.getAttribute('title'), el.getAttribute('name'),
      field ? '' : el.innerText, el.value && /^(submit|button|reset)$/.test(el.type || '') ? el.value : '', el.querySelector && el.querySelector('img[alt]') ? el.querySelector('img[alt]').alt : ''].map(norm).filter(Boolean); };
  const SEL = 'a[href], button, summary, select, textarea, input:not([type="hidden"]), [role="button"], [role="link"], [role="tab"], [role="menuitem"], [role="option"], [role="checkbox"], [role="radio"], [role="switch"], [contenteditable=""], [contenteditable="true"]';
  const els = all(document, []).filter(el => el.matches(SEL) && shown(el));
  let hits = els.filter(el => names(el).includes(want));
  if (!hits.length) hits = els.filter(el => names(el).some(n => n.includes(want)));
  if (!hits.length) return {found: false};
  const el = hits[0];
  const out = {found: true, kind: el.getAttribute('role') || el.tagName.toLowerCase(), same_name: hits.length,
    disabled: !!el.disabled || el.getAttribute('aria-disabled') === 'true' || !!el.closest('[inert]')};
  if (__COMPARE__) out.equals = norm('value' in el && el.tagName !== 'BUTTON' ? el.value : el.innerText) === norm(__TEXT__);
  return out;
})()"#;

/// Whether an alert or live region shows `__TEXT__` (any text when it is empty).
const ALERT_JS: &str = r#"(() => {
  __DEEP__
  const want = norm(__TEXT__);
  return all(document, []).some(el => el.matches('[role="alert"], [role="alertdialog"], [role="status"], [aria-live="assertive"], [aria-live="polite"], output')
    && shown(el) && norm(el.innerText) !== '' && norm(el.innerText).includes(want));
})()"#;

/// Whether `__TEXT__` shows on the page, open shadow roots included.
const SHOWS_JS: &str = r#"(() => {
  __DEEP__
  const want = norm(__TEXT__);
  if (!document.body) return false;
  if (norm(document.body.innerText).includes(want)) return true;
  return all(document, []).some(el => el.shadowRoot && Array.from(el.shadowRoot.children).some(c => norm(c.innerText).includes(want)));
})()"#;

fn script(js: &str, name: &str, text: &str) -> String {
    let quoted = |s: &str| serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into());
    js.replace("__DEEP__", DEEP_JS).replace("__NAME__", &quoted(name)).replace("__TEXT__", &quoted(text))
}

/// The keys a step may press: (name, key, code, virtual key code, text it types).
const KEYS: [(&str, &str, &str, u32, &str); 8] = [
    ("enter", "Enter", "Enter", 13, "\r"),
    ("escape", "Escape", "Escape", 27, ""),
    ("tab", "Tab", "Tab", 9, ""),
    ("space", " ", "Space", 32, " "),
    ("arrowup", "ArrowUp", "ArrowUp", 38, ""),
    ("arrowdown", "ArrowDown", "ArrowDown", 40, ""),
    ("arrowleft", "ArrowLeft", "ArrowLeft", 37, ""),
    ("arrowright", "ArrowRight", "ArrowRight", 39, ""),
];

fn key_named(name: &str) -> Option<&'static (&'static str, &'static str, &'static str, u32, &'static str)> {
    let wanted = name.trim().to_ascii_lowercase();
    KEYS.iter().find(|k| k.0 == wanted)
}

/// Refuse a journey that could not run before the browser is touched.
fn check_steps(steps: &[Step]) -> Result<(), ToolError> {
    if steps.is_empty() || steps.len() > 60 {
        return Err(ToolError::invalid("steps must hold 1..=60 steps"));
    }
    for (n, step) in steps.iter().enumerate() {
        let (what, empty) = match step {
            Step::Click { click } => ("click needs the control's name", click.trim().is_empty()),
            Step::Type { text, into } => ("type needs a text and the field's name", text.is_empty() || into.trim().is_empty()),
            Step::Press { press } => ("press takes Enter, Escape, Tab, Space, ArrowUp, ArrowDown, ArrowLeft or ArrowRight", key_named(press).is_none()),
            Step::Open { open } => ("open needs an address on the site, like \"/#/library\"", open.trim().is_empty()),
            Step::Expect { expect } => ("expect needs the text that must show", expect.trim().is_empty()),
            Step::Check { check, .. } => ("check needs the control's or field's name", check.trim().is_empty()),
            Step::Value { value_of, .. } => ("value_of needs the field's name", value_of.trim().is_empty()),
            Step::Alert { .. } => ("", false),
        };
        if empty {
            return Err(ToolError::invalid(format!("step {}: {what}", n + 1)));
        }
    }
    Ok(())
}

/// `path` as an address on the site that was opened at `start`; another site's address is refused.
fn on_the_site(start: &str, path: &str) -> Result<String, String> {
    let scheme_end = start.find("://").map(|i| i + 3).ok_or("the open site has no address to open a path on")?;
    let origin_end = start[scheme_end..].find(['/', '?', '#']).map_or(start.len(), |i| scheme_end + i);
    let origin = &start[..origin_end];
    let path = path.trim();
    if path.starts_with(origin) && path[origin.len()..].chars().next().is_none_or(|c| matches!(c, '/' | '?' | '#')) {
        return Ok(path.to_string());
    }
    if path.contains("://") || path.starts_with("//") {
        return Err(format!("open takes an address on the open site ({origin}), not another site's"));
    }
    Ok(if path.starts_with('/') { format!("{origin}{path}") } else { format!("{origin}/{path}") })
}

/// Load `url` from scratch. Going through a blank page first makes an address that differs from
/// the current one only after its `#` load again too, instead of only moving within the page.
fn load(client: &mut CdpClient, url: &str, width: u32, dl: &Deadline) -> Result<(), ToolError> {
    navigate(client, "about:blank", width, VIEW_HEIGHT, 0, dl)?;
    navigate(client, url, width, VIEW_HEIGHT, 800, dl).map(|_| ())
}

fn click_at(client: &mut CdpClient, x: f64, y: f64, dl: &Deadline) -> Result<(), ToolError> {
    client.call("Input.dispatchMouseEvent", json!({"type": "mouseMoved", "x": x, "y": y}), None, dl)?;
    for kind in ["mousePressed", "mouseReleased"] {
        client.call("Input.dispatchMouseEvent", json!({"type": kind, "x": x, "y": y, "button": "left", "clickCount": 1}), None, dl)?;
    }
    Ok(())
}

/// Do one step: `Ok(note)` says what was done, `Err(note)` why the step failed.
fn run_step(client: &mut CdpClient, step: &Step, start: &str, p: &JourneyParams, dl: &Deadline) -> Result<Result<String, String>, ToolError> {
    Ok(match step {
        Step::Click { click } => {
            let found = eval(client, &script(FIND_CONTROL_JS, click, ""), dl)?;
            if found["found"] != true {
                return Ok(Err(format!("no control named {click:?} shows on the page")));
            }
            let kind = found["kind"].as_str().unwrap_or("control");
            if found["disabled"] == true {
                return Ok(Err(format!("the {kind} {click:?} is disabled")));
            }
            if found["reached"] != true {
                let by = found["covered_by"].as_str().unwrap_or("something else");
                return Ok(Err(format!("the {kind} {click:?} is covered by a <{by}> at its centre, so a click would not reach it")));
            }
            click_at(client, found["x"].as_f64().unwrap_or(0.0), found["y"].as_f64().unwrap_or(0.0), dl)?;
            settle(client, 400)?;
            let others = found["same_name"].as_u64().unwrap_or(1).saturating_sub(1);
            let found_by = found["how"].as_str().unwrap_or("named");
            Ok(format!(
                "clicked the {kind} {click:?} (found by: {found_by}{})",
                if others > 0 { format!("; the first of {} with that name", others + 1) } else { String::new() }
            ))
        }
        Step::Type { text, into } => {
            let found = eval(client, &script(FIND_FIELD_JS, into, text), dl)?;
            if found["found"] != true {
                return Ok(Err(format!("no field named {into:?} shows on the page")));
            }
            if found["locked"] == true {
                return Ok(Err(format!("the field {into:?} is disabled or read-only")));
            }
            if found["kind"] == "select" {
                if found["chosen"] != true {
                    return Ok(Err(format!("the drop-down {into:?} has no option {text:?}")));
                }
                settle(client, 300)?;
                return Ok(Ok(format!("chose {text:?} in the drop-down {into:?}")));
            }
            if found["kind"] == "set" {
                let field = found["field"].as_str().unwrap_or("date");
                if found["kept"] != true {
                    return Ok(Err(format!("the {field} field {into:?} does not take {text:?} (a date is written 2026-11-20)")));
                }
                settle(client, 300)?;
                return Ok(Ok(format!("set the {field} field {into:?} to {text:?}")));
            }
            client.call("Input.insertText", json!({"text": text}), None, dl)?;
            settle(client, 300)?;
            Ok(format!("typed into the field {into:?}"))
        }
        Step::Press { press } => {
            let Some((_, key, code, vk, text)) = key_named(press) else { return Ok(Err(format!("{press:?} is not a key this tool presses"))) };
            let mut down = json!({"type": "keyDown", "key": key, "code": code, "windowsVirtualKeyCode": vk, "nativeVirtualKeyCode": vk});
            if !text.is_empty() {
                down["text"] = json!(text);
            }
            client.call("Input.dispatchKeyEvent", down, None, dl)?;
            client.call(
                "Input.dispatchKeyEvent",
                json!({"type": "keyUp", "key": key, "code": code, "windowsVirtualKeyCode": vk, "nativeVirtualKeyCode": vk}),
                None,
                dl,
            )?;
            settle(client, 400)?;
            Ok(format!("pressed {key}", key = if *key == " " { "Space" } else { key }))
        }
        Step::Open { open } => match on_the_site(start, open) {
            Err(why) => Err(why),
            Ok(url) => {
                load(client, &url, p.width, dl)?;
                Ok(format!("opened {}", shown_url(&url)))
            }
        },
        Step::Expect { expect } => match wait_for(client, p, dl, |c| Ok(eval(c, &script(SHOWS_JS, "", expect), dl).is_ok_and(|shows| shows == true)))? {
            true => Ok(format!("{expect:?} shows")),
            false => Err(format!("{expect:?} did not show within {} ms", p.wait_ms)),
        },
        Step::Check { check, is } => {
            let holds = |state: &Value| match is {
                ControlState::Absent => state["found"] != true,
                ControlState::Present => state["found"] == true,
                ControlState::Enabled => state["found"] == true && state["disabled"] != true,
                ControlState::Disabled => state["found"] == true && state["disabled"] == true,
                ControlState::Unusable => state["found"] != true || state["disabled"] == true,
            };
            let js = script(STATE_OF_JS, check, "").replace("__COMPARE__", "false");
            let mut last = Value::Null;
            let held = wait_for(client, p, dl, |c| {
                last = eval(c, &js, dl).unwrap_or(Value::Null);
                Ok(holds(&last))
            })?;
            let now = match (last["found"] == true, last["disabled"] == true) {
                (false, _) => "no control or field by that name shows",
                (true, true) => "it shows but can't be used",
                (true, false) => "it shows and can be used",
            };
            if held { Ok(format!("{check:?} is {} ({now})", is.word())) } else { Err(format!("{check:?} is not {} within {} ms: {now}", is.word(), p.wait_ms)) }
        }
        Step::Value { value_of, equals } => {
            let js = script(STATE_OF_JS, value_of, equals).replace("__COMPARE__", "true");
            let mut found = false;
            let held = wait_for(client, p, dl, |c| {
                let v = eval(c, &js, dl).unwrap_or(Value::Null);
                found = v["found"] == true;
                Ok(v["equals"] == true)
            })?;
            match (held, found) {
                (true, _) => Ok(format!("{value_of:?} holds {equals:?}")),
                (false, true) => Err(format!("{value_of:?} does not hold {equals:?} within {} ms", p.wait_ms)),
                (false, false) => Err(format!("no field named {value_of:?} shows on the page")),
            }
        }
        Step::Alert { alert } => match wait_for(client, p, dl, |c| Ok(eval(c, &script(ALERT_JS, "", alert), dl).is_ok_and(|shows| shows == true)))? {
            true if alert.trim().is_empty() => Ok("an alert is announced".to_string()),
            true => Ok(format!("an alert announces {alert:?}")),
            false => Err(format!("no alert{} within {} ms", if alert.trim().is_empty() { String::new() } else { format!(" announcing {alert:?}") }, p.wait_ms)),
        },
    })
}

/// Ask `holds` until it is true or `p.wait_ms` passes. A page that is still loading has no
/// script context for a moment, so a failed question is asked again.
fn wait_for(
    client: &mut CdpClient,
    p: &JourneyParams,
    dl: &Deadline,
    mut holds: impl FnMut(&mut CdpClient) -> Result<bool, ToolError>,
) -> Result<bool, ToolError> {
    let wait = Deadline::new(Duration::from_millis(p.wait_ms), crate::tool::CancelToken::default());
    loop {
        if holds(client)? {
            return Ok(true);
        }
        if wait.expired() || dl.expired() {
            return Ok(false);
        }
        settle(client, 100)?;
    }
}

/// What a run left: each step that ran, the first that failed (from 0), where the page ended, and
/// its picture when one was asked for.
struct Ran {
    rows: Vec<Value>,
    failed_at: Option<usize>,
    url: String,
    picture: Option<Vec<u8>>,
}

fn web_site_journey_run(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: JourneyParams = parse_params(params)?;
    check_steps(&p.steps)?;
    if !(320..=3840).contains(&p.width) {
        return Err(ToolError::invalid("width must be 320..=3840"));
    }
    if !(100..=30_000).contains(&p.wait_ms) {
        return Err(ToolError::invalid("wait_ms must be 100..=30000"));
    }
    let dl = deadline(ctx, p.timeout_secs.max(30))?;
    let site = open_site(ctx, &dl)?;
    let out = p.output_directory.as_deref().map(create_absent_dir).transpose()?;
    let mut client = page_client(&site, &dl)?;
    // Where the site was opened; a browser whose record of it is lost starts from where it is now.
    let opened = site.url.lock().map(|u| u.clone()).unwrap_or_default();
    let start =
        if opened.is_empty() { eval(&mut client, "location.href", &dl).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default() } else { opened };
    let r = (|| -> Result<Ran, ToolError> {
        client.call("Page.enable", json!({}), None, &dl)?;
        if p.restart {
            load(&mut client, &start, p.width, &dl)?;
        } else {
            set_viewport(&mut client, p.width, VIEW_HEIGHT, &dl)?;
            settle(&mut client, 300)?;
        }
        let (mut rows, mut failed_at) = (Vec::new(), None);
        for (n, step) in p.steps.iter().enumerate() {
            if dl.expired() {
                rows.push(json!({"step": step, "ok": false, "note": "the time for this run ended before this step"}));
                failed_at = Some(n);
                break;
            }
            ctx.report(Some(n as f32 / p.steps.len() as f32), format!("Step {} of {}…", n + 1, p.steps.len()));
            let done = run_step(&mut client, step, &start, &p, &dl)?;
            rows.push(json!({"step": step, "ok": done.is_ok(), "note": match &done { Ok(note) | Err(note) => note }}));
            if done.is_err() {
                failed_at = Some(n);
                break;
            }
        }
        let url = eval(&mut client, "location.href", &dl).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
        let picture = match &out {
            Some(_) => {
                let shot = client.call("Page.captureScreenshot", json!({"format": "png"}), None, &dl)?;
                shot.get("data").and_then(Value::as_str).map(web_common::b64_decode).transpose()?
            }
            None => None,
        };
        Ok(Ran { rows, failed_at, url, picture })
    })();
    cleanup(&mut client, &dl);
    let Ran { rows, failed_at, url, picture } = r?;
    let mut result = json!({"passed": failed_at.is_none(), "steps": rows, "steps_given": p.steps.len(), "failed_at": failed_at.map(|n| n + 1), "url": url, "width": p.width});
    if let (Some(out), Some(bytes)) = (&out, &picture) {
        write_new_file(out, &["end.png".into()], bytes).map_err(ToolError::invalid)?;
        result["picture"] = json!(out.join("end.png").display().to_string());
        result["picture_sha256"] = json!(sha256_hex(bytes));
    }
    let shown = shown_url(&start);
    let summary = match failed_at {
        None => format!("All {} step(s) passed on {shown}.", p.steps.len()),
        Some(n) => format!("Step {} of {} failed on {shown}: {}", n + 1, p.steps.len(), rows[n]["note"].as_str().unwrap_or("it failed")),
    };
    let browser = site.handle();
    let ev =
        EvidenceBuilder::new("web_site_journey_run", Subject { kind: "website".into(), sha256: None, name: Some(shown) }, browser_env(ctx.session.clean_room(), &browser))
            .params(json!({"steps": p.steps.len(), "width": p.width, "restart": p.restart, "wait_ms": p.wait_ms}))
            .result(result)
            .summary(summary)
            .confidence(Confidence::Observed)
            .authority(Authority::OurRun)
            .limitation(Limitation::new(
                "controls_by_name",
                "Controls and fields are found by their visible or accessible name, and the first one that shows wins when several share it. Controls in frames are not found, and an expected text is looked for anywhere on the page.",
            ))
            .coverage(Coverage::complete(rows.len() as u64))
            .location(Location::new("url", url))
            .build()?;
    Ok(ToolOutput::evidence(ev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_opened_on_the_same_site_and_another_site_is_refused() {
        let start = "http://127.0.0.1:5181/index.html#/home";
        assert_eq!(on_the_site(start, "/#/library").unwrap(), "http://127.0.0.1:5181/#/library");
        assert_eq!(on_the_site(start, "about").unwrap(), "http://127.0.0.1:5181/about");
        assert_eq!(on_the_site(start, "http://127.0.0.1:5181/x?y=1").unwrap(), "http://127.0.0.1:5181/x?y=1");
        assert!(on_the_site(start, "https://example.com/").is_err());
        assert!(on_the_site(start, "//example.com/").is_err());
        assert!(on_the_site(start, "http://127.0.0.1:51819/").is_err(), "a longer port is another site");
    }

    #[test]
    fn a_journey_that_could_not_run_is_refused_before_the_browser_is_touched() {
        let steps = |v: Value| serde_json::from_value::<Vec<Step>>(v);
        assert!(steps(json!([{"click": "Go"}, {"type": "cats", "into": "Search"}, {"press": "Enter"}, {"open": "/"}, {"expect": "Results"}])).is_ok());
        assert!(steps(json!([{"click": "Go", "expect": "Results"}])).is_err(), "one thing per step");
        assert!(steps(json!([{"hover": "Go"}])).is_err());
        assert!(check_steps(&[]).is_err());
        assert!(check_steps(&[Step::Press { press: "F13".into() }]).is_err());
        assert!(check_steps(&[Step::Click { click: " ".into() }]).is_err());
        assert!(check_steps(&[Step::Press { press: "enter".into() }, Step::Expect { expect: "Done".into() }]).is_ok());
        assert!(check_steps(&[Step::Check { check: " ".into(), is: ControlState::Disabled }]).is_err());
        let parsed: Vec<Step> =
            serde_json::from_value(json!([{"check": "Submit", "is": "unusable"}, {"value_of": "Name", "equals": "Ada"}, {"alert": ""}])).unwrap();
        assert_eq!(parsed[0], Step::Check { check: "Submit".into(), is: ControlState::Unusable }, "each new step reads as itself, not as another");
        assert!(matches!(parsed[1], Step::Value { .. }) && matches!(parsed[2], Step::Alert { .. }));
    }
}
