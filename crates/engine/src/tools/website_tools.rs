//! Websites as targets: `web_site_open` starts a private browser owned by the session and loads a
//! page; every other browser tool then looks at that page by default (see
//! [`super::web_common::Browser::resolve`]). `web_site_capture` records what a rebuild needs —
//! full-page screenshots at several widths, the rendered HTML, design tokens, a layout outline,
//! assets, links and form outlines — into a folder. `web_site_close` ends the browser.
//!
//! Data minimisation: form field values, cookies and storage are never read; the capture holds
//! what any visitor sees on the page.

use super::artifact_tools::{create_absent_dir, write_new_file};
use super::browser_tools::{browser_env, cleanup, deadline};
use super::native_common::spec;
use super::site_sign_in;
use super::web_common::{self, Browser, LaunchOptions, ProfileChoice};
use crate::tool::{Caller, Ctx, Effects, Requires, ToolOutput, ToolSpec, Toolset, parse_params, schema_of};
use craft_anything_engines::cdp::{self, CdpClient, CdpEvent, Deadline, Endpoint, PAGE_MAX_MESSAGE};
use craft_anything_evidence::{Authority, Confidence, Coverage, ErrorCode, EvidenceBuilder, Limitation, Location, Subject, ToolError, sha256_hex};
use craft_anything_platform::sync::Mutex;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

mod journey;

pub fn specs() -> Vec<ToolSpec> {
    vec![
        spec(
            "web_site_open",
            "Open a website",
            "Load a web page (http/https) in a private browser owned by this session (fresh profile, no history or logins) at a chosen window size, wait for it to finish loading, and report its final address, title and size. When web_site_sign_in saved a sign-in for that exact site, the private profile starts with a copy of it (signed_in: false opens signed out; true requires one). Every browser tool then inspects this page unless given another endpoint. Runs the site's scripts, so it needs your approval.",
            "Open a website in a private, hidden browser so it can be studied.",
            Toolset::Browser,
            Effects::EVIDENCE
                .union(Effects::EXECUTES_FOREIGN_CODE)
                .union(Effects::ACCESSES_NETWORK)
                .union(Effects::LAUNCHES_PROCESS)
                .union(Effects::WRITES_FILESYSTEM),
            Requires::Engine("chromium"),
            schema_of::<OpenParams>,
            web_site_open,
        ),
        spec(
            "web_site_capture",
            "Capture a website for rebuilding",
            "Record the open website into a new folder: full-page PNG screenshots at each requested width, the rendered HTML, design tokens (fonts, sizes, weights, text and background colours as hex, radii, shadows, spacing, content widths, CSS custom properties), the widths its CSS switches layout at, the layout at each width (page width, heading and text sizes, each section's padding, gap, content column and items per row), a layout outline of visible landmarks, headings, controls and images with positions, asset lists (images, stylesheets, scripts, font files, the requests it made), links and form outlines (never values). Then presses Tab through the page from its top (up to 80 times) and records how many visible controls the focus reached and which showed no focus. Writes capture.json and a README.md summary.",
            "Save pictures and a blueprint of a web page so it can be rebuilt.",
            Toolset::Browser,
            Effects::EVIDENCE.union(Effects::WRITES_FILESYSTEM).union(Effects::ACCESSES_NETWORK),
            Requires::Engine("chromium"),
            schema_of::<CaptureParams>,
            web_site_capture,
        ),
        spec(
            "web_page_states_capture",
            "Capture menus, dialogs and tabs",
            "On the open website, open up to max_states of its visible menus, pop-ups, tabs and collapsed sections (aria-expanded, aria-haspopup, unselected tabs, <summary>; never form controls or submit buttons) one at a time: click it, take a screenshot, record what changed (expanded, a dialog appeared, the tab or the address changed), then put the page back. Then, when the page has a search field, type a word into it without submitting (search_text, or by default the word its headings and links use most) and picture what showed, such as suggestions; the field is emptied after. Writes state-<n>.png and states.json into a new folder. Clicking runs the site's own code, so it needs your approval.",
            "Open the site's menus, pop-ups and tabs and photograph each, so a rebuild can match them too.",
            Toolset::Browser,
            Effects::EVIDENCE.union(Effects::WRITES_FILESYSTEM).union(Effects::EXECUTES_FOREIGN_CODE).union(Effects::ACCESSES_NETWORK),
            Requires::Engine("chromium"),
            schema_of::<StatesParams>,
            web_page_states_capture,
        ),
        journey::tool(),
        spec(
            "web_site_close",
            "Close the website",
            "Close the private browser started by web_site_open and delete its temporary profile.",
            "Close the hidden browser.",
            Toolset::Browser,
            Effects::MUTATES_SESSION.union(Effects::IDEMPOTENT),
            Requires::Engine("chromium"),
            schema_of::<CloseParams>,
            web_site_close,
        ),
    ]
}

/// The session's website browser. The process lives as long as this slot (it is killed and its
/// profile removed when the last handle drops).
pub struct SiteBrowser {
    pub endpoint: Endpoint,
    pub product: String,
    pub version: String,
    pub url: Mutex<String>,
    /// What went wrong while the last page opened ([`page_health`]); web_site_capture records it.
    pub health: Mutex<Value>,
    /// The site key whose saved sign-in this browser started with; `None`: a fresh private profile.
    pub signed_in: Option<String>,
    /// Revision of the saved profile copied at launch; `None` for a fresh profile.
    pub sign_in_revision: Option<String>,
    _owner: Mutex<Browser>,
}

impl SiteBrowser {
    /// A non-owning handle for the shared browser helpers.
    fn handle(&self) -> Browser {
        Browser::attached(&self.endpoint, &self.product, &self.version)
    }
}

// ---------------------------------------------------------------------------------------------
// web_site_open

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenParams {
    /// The page to open (http or https).
    pub url: String,
    /// Window width in CSS pixels (320..=3840, default 1440).
    #[serde(default = "d_width")]
    pub width: u32,
    /// Window height in CSS pixels (320..=4000, default 900).
    #[serde(default = "d_height")]
    pub height: u32,
    /// Extra time to let late content settle after the load event, in milliseconds (default 2000).
    #[serde(default = "d_settle")]
    pub settle_ms: u64,
    /// Run the browser without a window (default true).
    #[serde(default = "yes")]
    pub headless: bool,
    #[serde(default = "d_timeout")]
    pub timeout_secs: u64,
    /// Use the sign-in saved for this site by web_site_sign_in. Unset: use it when one is saved;
    /// false: a fresh private profile; true: it must be saved (an error otherwise).
    #[serde(default)]
    pub signed_in: Option<bool>,
}

fn d_width() -> u32 {
    1440
}
fn d_height() -> u32 {
    900
}
fn d_settle() -> u64 {
    2000
}
fn d_timeout() -> u64 {
    90
}
fn yes() -> bool {
    true
}

/// Only http(s) pages; never local files or browser-internal schemes.
pub fn check_url(url: &str) -> Result<&str, ToolError> {
    let u = url.trim();
    let rest = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://"));
    match rest {
        Some(r) if !r.is_empty() && !r.starts_with('/') && !u.chars().any(char::is_whitespace) => Ok(u),
        _ => Err(ToolError::invalid(format!("{url:?} is not a web address")).with_remediation("Give a full address such as https://example.com.")),
    }
}

/// A URL as shown in evidence and summaries: no credentials, query or fragment.
pub(crate) fn shown_url(url: &str) -> String {
    web_common::sanitize_url(url).split(['?', '#']).next().unwrap_or_default().to_string()
}

/// Open a page, reusing a live browser only when its requested profile still matches.
fn web_site_open(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: OpenParams = parse_params(params)?;
    let url = check_url(&p.url)?.to_string();
    if !(320..=3840).contains(&p.width) || !(320..=4000).contains(&p.height) {
        return Err(ToolError::invalid("width must be 320..=3840 and height 320..=4000"));
    }
    let dl = deadline(ctx, p.timeout_secs)?;
    let saved = chosen_sign_in(ctx.session, &url, p.signed_in)?;
    let reservation = saved.as_ref().map(|s| site_sign_in::lock_site(ctx.session, &s.key)).transpose()?;
    // Re-read under the reservation: a different engine may have replaced or forgotten it.
    let saved = if reservation.is_some() { chosen_sign_in(ctx.session, &url, p.signed_in)? } else { saved };
    let want = saved.as_ref().map(|s| s.key.clone());
    // A browser started for another sign-in (or none) isn't reused: each site sees only its own.
    // The new one replaces it once it has started, so a failed start leaves the old one open.
    let revision = saved.as_ref().map(|s| s.revision.clone());
    let reuse = live_site(ctx, &dl).filter(|s| reuse_profile(p.signed_in, s.signed_in.as_deref(), s.sign_in_revision.as_deref(), saved.as_ref()));
    let site = match reuse {
        Some(s) => s,
        None => {
            ctx.report(None, if want.is_some() { "Starting a private browser with your saved sign-in…" } else { "Starting a private browser…" });
            let profile = saved.as_ref().map_or(ProfileChoice::Fresh, |s| ProfileChoice::SeededFrom(s.profile.clone()));
            let opts = LaunchOptions { headless: p.headless, profile, ..LaunchOptions::default() };
            let b = Browser::launch(ctx, &opts, &dl)?;
            let site = Arc::new(SiteBrowser {
                endpoint: b.endpoint.clone(),
                product: b.product.clone(),
                version: b.version.clone(),
                url: Mutex::new(String::new()),
                health: Mutex::new(Value::Null),
                signed_in: want.clone(),
                sign_in_revision: revision,
                _owner: Mutex::new(b),
            });
            ctx.session.ext.insert(site.clone());
            site
        }
    };
    drop(reservation);
    ctx.report(None, format!("Loading {url}…"));
    let mut client = page_client(&site, &dl)?;
    let r = navigate(&mut client, &url, p.width, p.height, p.settle_ms, &dl).and_then(|health| Ok((health, page_facts(&mut client, &dl)?)));
    cleanup(&mut client, &dl);
    let (health, facts) = r?;
    let final_url = facts.get("url").and_then(Value::as_str).unwrap_or(&url).to_string();
    if let Ok(mut u) = site.url.lock() {
        u.clone_from(&final_url);
    }
    if let Ok(mut h) = site.health.lock() {
        h.clone_from(&health);
    }
    // Clean room keeps what a visitor sees: not the page's load errors, failed requests or size in
    // elements. A dialog the page opened is something a visitor sees, so it's said either way.
    let dialogs = health.get("dialogs").cloned().unwrap_or_else(|| json!([]));
    let (visitor, mut facts) = (ctx.session.clean_room(), facts);
    let health = if visitor {
        if let Some(f) = facts.as_object_mut() {
            f.remove("elements");
        }
        Value::Null
    } else {
        health
    };
    let title = facts.get("title").and_then(Value::as_str).unwrap_or_default().to_string();
    let shown = shown_url(&final_url);
    let how = if want.is_some() { " signed in" } else { "" };
    let opened = if title.is_empty() { format!("Opened {shown}{how}.") } else { format!("Opened {shown}{how}: \"{title}\".") };
    let count = |k: &str| health.get(k).and_then(Value::as_array).map_or(0, Vec::len);
    let summary = match (count("errors"), count("failed_requests")) {
        (0, 0) => opened,
        (e, f) => format!("{opened} While loading: {e} error(s), {f} failed request(s)."),
    };
    let summary = format!("{summary}{}", dialogs_said(&dialogs));
    // Pages opened signed in often land on addresses carrying codes or tokens: keep only the path.
    let (final_url, facts) = if want.is_some() { signed_in_urls(final_url, facts) } else { (final_url, facts) };
    let browser = site.handle();
    let ev = EvidenceBuilder::new(
        "web_site_open",
        Subject { kind: "website".into(), sha256: None, name: Some(shown.clone()) },
        browser_env(ctx.session.clean_room(), &browser),
    )
    .params(json!({"url": shown, "width": p.width, "height": p.height}))
    .result(json!({"url": final_url, "title": title, "facts": facts, "health": health, "dialogs": dialogs, "browser": site.product, "signed_in": want.is_some(), "signed_in_site": want}))
    .summary(summary)
    .confidence(Confidence::Observed)
    .authority(Authority::OurRun)
    .limitation(profile_limitation(want.as_deref()))
    .coverage(Coverage::complete(1))
    .location(Location::new("url", shown))
    .build()?;
    Ok(ToolOutput::evidence(ev))
}

/// The summary's sentence on dialogs the page opened by itself while it loaded (empty when none).
fn dialogs_said(dialogs: &Value) -> String {
    web_common::dialogs_said_in(dialogs)
}

/// Explicit signed-out opens must discard even a matching browser's accumulated state.
fn reuse_profile(requested: Option<bool>, key: Option<&str>, revision: Option<&str>, saved: Option<&site_sign_in::SavedSignIn>) -> bool {
    requested != Some(false) && key == saved.map(|s| s.key.as_str()) && revision == saved.map(|s| s.revision.as_str())
}

/// The saved sign-in `web_site_open` starts with for `url`: the one saved for its exact site,
/// unless `signed_in` is false. `true` with none saved is an error naming the tool that saves one.
fn chosen_sign_in(session: &crate::session::Session, url: &str, signed_in: Option<bool>) -> Result<Option<site_sign_in::SavedSignIn>, ToolError> {
    if signed_in == Some(false) {
        return Ok(None);
    }
    let key = site_sign_in::site_key(url);
    let saved = key.as_ref().ok().and_then(|k| site_sign_in::saved(session, k));
    if signed_in == Some(true) && saved.is_none() {
        let site = key.unwrap_or_else(|_| shown_url(url));
        return Err(ToolError::new(ErrorCode::NotFound, format!("No sign-in is saved for {site}.")).with_remediation(format!(
            "Sign in once with {} (it opens a window where you sign in yourself), then open the site again.",
            site_sign_in::SIGN_IN_TOOL
        )));
    }
    Ok(saved)
}

/// A signed-in page's address and the facts' copy of it without query or fragment.
fn signed_in_urls(url: String, mut facts: Value) -> (String, Value) {
    if let Some(f) = facts.as_object_mut()
        && let Some(u) = f.get("url").and_then(Value::as_str).map(shown_url)
    {
        f.insert("url".into(), json!(u));
    }
    (shown_url(&url), facts)
}

/// What the opened pages' browser profile means for what they show.
fn profile_limitation(signed_in: Option<&str>) -> Limitation {
    match signed_in {
        Some(site) => Limitation::new(
            "signed_in",
            format!(
                "Opened signed in with your saved sign-in for {site}: screenshots and page text may show your own account's data. If the site has ended that sign-in since, pages show their signed-out state."
            ),
        ),
        None => Limitation::new("private_profile", "A fresh private browser: pages behind a login show their signed-out state."),
    }
}

/// The session's website browser when it is still answering; a dead one is dropped.
fn live_site(ctx: &mut Ctx<'_>, dl: &Deadline) -> Option<Arc<SiteBrowser>> {
    let site = ctx.session.ext.get::<SiteBrowser>()?;
    if cdp::json_version(&site.endpoint, dl).is_ok() {
        return Some(site);
    }
    ctx.session.ext.remove::<SiteBrowser>();
    None
}

/// The site `web_site_open` opened in this session. Each CLI command is a session of its own, and
/// its browser closes when the command ends, so from the CLI the steps belong in one session (#153).
fn open_site(ctx: &mut Ctx<'_>, dl: &Deadline) -> Result<Arc<SiteBrowser>, ToolError> {
    live_site(ctx, dl).ok_or_else(|| {
        let fix = match ctx.caller {
            Caller::Cli => {
                "Each CLI command starts its own browser and closes it when it ends, so a site opened by an earlier command isn't open here. Run web_site_open and the tools that use the page in one session: an MCP client (`craft-anything-cli mcp`), the app, or `craft-anything-cli rebuild`."
            }
            _ => "Open one first with web_site_open.",
        };
        ToolError::new(ErrorCode::NotFound, "no website is open").with_remediation(fix)
    })
}

fn page_client(site: &SiteBrowser, dl: &Deadline) -> Result<CdpClient, ToolError> {
    let b = site.handle();
    let targets = cdp::json_list(&site.endpoint, dl)?;
    let target = web_common::pick_target(&targets, None, Some("page"))?;
    // Nobody sits at this private browser, so a page's own alert would hold it until the deadline.
    web_common::connect_target(&b, target, PAGE_MAX_MESSAGE, dl).map(CdpClient::answering_dialogs)
}

/// Size the page and pin light mode, so captures don't depend on this computer's theme.
fn set_viewport(client: &mut CdpClient, width: u32, height: u32, dl: &Deadline) -> Result<(), ToolError> {
    client.call("Emulation.setDeviceMetricsOverride", json!({"width": width, "height": height, "deviceScaleFactor": 1, "mobile": width < 600}), None, dl)?;
    let _ = client.call("Emulation.setEmulatedMedia", json!({"features": [{"name": "prefers-color-scheme", "value": "light"}]}), None, dl);
    Ok(())
}

/// Open `url` and wait for it to settle; returns what went wrong while it loaded ([`page_health`]).
pub(crate) fn navigate(client: &mut CdpClient, url: &str, width: u32, height: u32, settle_ms: u64, dl: &Deadline) -> Result<Value, ToolError> {
    client.call("Page.enable", json!({}), None, dl)?;
    // Errors and failed requests are read from the browser's own events; a browser that refuses
    // one of these still loads the page.
    for domain in ["Runtime.enable", "Log.enable", "Network.enable"] {
        let _ = client.call(domain, json!({}), None, dl);
    }
    set_viewport(client, width, height, dl)?;
    let r = client.call("Page.navigate", json!({"url": url}), None, dl)?;
    if let Some(err) = r.get("errorText").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        return Err(ToolError::new(ErrorCode::ProgramFailed, format!("the page didn't load: {err}"))
            .with_remediation("Check the address and your internet connection, then try again."));
    }
    // Wait for the load event (bounded by the overall deadline), then let late content settle.
    let loaded = Deadline::new(Duration::from_secs(45), crate::tool::CancelToken::default());
    let mut done = false;
    let mut seen: Vec<CdpEvent> = Vec::new();
    let mut keep = |events: Vec<CdpEvent>| {
        let wanted = events.into_iter().filter(|e| {
            e.method.starts_with("Runtime.") || e.method.starts_with("Log.") || e.method.starts_with("Network.") || e.method == cdp::DIALOG_OPENING
        });
        seen.extend(wanted.take(MAX_HEALTH_EVENTS.saturating_sub(seen.len())));
    };
    while !done && !loaded.expired() && !dl.expired() {
        let slice = Deadline::new(Duration::from_millis(250), crate::tool::CancelToken::default());
        client.pump_until(&slice)?;
        let events = client.take_events();
        done = events.iter().any(|e| e.method == "Page.loadEventFired");
        keep(events);
    }
    let settle = Deadline::new(Duration::from_millis(settle_ms.min(30_000)), crate::tool::CancelToken::default());
    client.pump_until(&settle)?;
    keep(client.take_events());
    for domain in ["Network.disable", "Log.disable"] {
        let _ = client.call(domain, json!({}), None, dl);
    }
    Ok(page_health(&seen))
}

/// Most browser events read for a page's health (a busy page sends thousands of network events).
const MAX_HEALTH_EVENTS: usize = 20_000;

/// What went wrong while a page loaded, from the browser's events: `errors` (uncaught script
/// errors, console errors and error log entries, each text once) and `failed_requests` (answers
/// of 400 or more, and loads that failed). Texts are cut to one line of 300 characters and
/// addresses lose their query strings, so ids in them aren't kept. A page that opened dialogs by
/// itself also gets `dialogs`: each one's kind, first line and how it was answered.
pub fn page_health(events: &[CdpEvent]) -> Value {
    let line = |s: &str| s.lines().next().unwrap_or_default().chars().take(300).collect::<String>();
    let str_at = |v: &Value, p: &str| v.pointer(p).and_then(Value::as_str).unwrap_or_default().to_string();
    let mut urls: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut errors: Vec<Value> = Vec::new();
    let mut failed: Vec<Value> = Vec::new();
    let mut add_error = |kind: &str, text: String, url: String| {
        if !text.is_empty() && errors.len() < 30 && !errors.iter().any(|e| e["text"] == text.as_str()) {
            errors.push(json!({"kind": kind, "text": text, "url": shown_url(&url)}));
        }
    };
    for e in events {
        let p = &e.params;
        match e.method.as_str() {
            "Network.requestWillBeSent" => {
                urls.insert(str_at(p, "/requestId"), str_at(p, "/request/url"));
            }
            "Runtime.exceptionThrown" => {
                let d = &p["exceptionDetails"];
                let text = Some(str_at(d, "/exception/description")).filter(|t| !t.is_empty()).unwrap_or_else(|| str_at(d, "/text"));
                add_error("script", line(&text), str_at(d, "/url"));
            }
            "Runtime.consoleAPICalled" if matches!(p["type"].as_str(), Some("error" | "assert")) => {
                let parts: Vec<String> = p["args"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|a| a.get("value").and_then(Value::as_str).or_else(|| a.get("description").and_then(Value::as_str)).map(String::from))
                    .collect();
                add_error("console", line(&parts.join(" ")), String::new());
            }
            // A failed load is also logged by the browser; the network events say it better.
            "Log.entryAdded" if p.pointer("/entry/level") == Some(&json!("error")) && p.pointer("/entry/source") != Some(&json!("network")) => {
                add_error("log", line(&str_at(p, "/entry/text")), str_at(p, "/entry/url"));
            }
            "Network.responseReceived" => {
                let status = p.pointer("/response/status").and_then(Value::as_u64).unwrap_or(0);
                if status >= 400 && failed.len() < 50 {
                    failed.push(json!({"url": shown_url(&str_at(p, "/response/url")), "status": status, "type": p["type"]}));
                }
            }
            "Network.loadingFailed" if p["canceled"] != true && failed.len() < 50 => {
                let url = urls.get(&str_at(p, "/requestId")).cloned().unwrap_or_default();
                failed.push(json!({"url": shown_url(&url), "error": p["errorText"], "type": p["type"]}));
            }
            _ => {}
        }
    }
    let dialogs = web_common::dialogs_from(events);
    let mut health = json!({"errors": errors, "failed_requests": failed});
    if !dialogs.is_empty()
        && let Some(o) = health.as_object_mut()
    {
        o.insert("dialogs".into(), json!(dialogs));
    }
    health
}

/// Evaluate one of our own read-only scripts in the page and return its JSON value.
pub(crate) fn eval(client: &mut CdpClient, script: &str, dl: &Deadline) -> Result<Value, ToolError> {
    let r = client.call("Runtime.evaluate", json!({"expression": script, "returnByValue": true, "awaitPromise": true}), None, dl)?;
    if let Some(ex) = r.get("exceptionDetails") {
        let m = ex.pointer("/exception/description").or_else(|| ex.get("text")).and_then(Value::as_str).unwrap_or("script error");
        return Err(ToolError::new(ErrorCode::ProgramFailed, format!("reading the page failed: {}", m.lines().next().unwrap_or(m))));
    }
    Ok(r.pointer("/result/value").cloned().unwrap_or(Value::Null))
}

fn page_facts(client: &mut CdpClient, dl: &Deadline) -> Result<Value, ToolError> {
    eval(
        client,
        "({url: location.href, title: document.title, ready: document.readyState, lang: document.documentElement.lang || '', \
          width: document.documentElement.scrollWidth, height: document.documentElement.scrollHeight, \
          elements: document.getElementsByTagName('*').length, images: document.images.length, links: document.links.length, \
          forms: document.forms.length})",
        dl,
    )
}

// ---------------------------------------------------------------------------------------------
// web_page_states_capture

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StatesParams {
    /// Absolute path of a folder that does not exist yet; the states are written there.
    pub output_directory: String,
    /// Most controls to open (1..=8, default 4).
    #[serde(default = "d_states")]
    pub max_states: usize,
    /// Page width in CSS pixels (320..=3840, default 1440).
    #[serde(default = "d_state_width")]
    pub width: u32,
    /// What to type into the page's search field (at most 40 characters; never submitted). Default:
    /// the word of five letters or more its headings and links use most. Empty: type nothing.
    #[serde(default)]
    pub search_text: Option<String>,
    #[serde(default = "d_timeout")]
    pub timeout_secs: u64,
}

fn d_states() -> usize {
    4
}
fn d_state_width() -> u32 {
    1440
}

/// Marks the visible toggles (`data-craft-state="<n>"`) and lists them, after dropping the marks
/// an earlier call left (they'd name a control hidden at this width, #339). `__MAX__` is replaced.
const FIND_STATES_JS: &str = r#"(() => {
  for (const old of document.querySelectorAll('[data-craft-state]')) old.removeAttribute('data-craft-state');
  const cap = (s, n) => String(s || '').replace(/\s+/g, ' ').trim().slice(0, n);
  const shown = el => { const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; const s = getComputedStyle(el); return s.visibility !== 'hidden' && s.display !== 'none' && (!el.checkVisibility || el.checkVisibility()); };
  const sel = '[aria-expanded="false"], [aria-haspopup]:not([aria-haspopup="false"]), [role="tab"][aria-selected="false"], details:not([open]) > summary';
  const out = [];
  for (const el of document.querySelectorAll(sel)) {
    if (out.length >= __MAX__) break;
    if (!shown(el) || el.closest('form') || el.matches('[type="submit"]')) continue;
    const pop = el.getAttribute('aria-haspopup');
    const kind = el.matches('summary') ? 'section' : el.getAttribute('role') === 'tab' ? 'tab' : pop === 'dialog' ? 'dialog' : 'menu';
    el.setAttribute('data-craft-state', String(out.length));
    out.push({index: out.length, kind, label: cap(el.innerText || el.getAttribute('aria-label') || el.getAttribute('title'), 80)});
  }
  return out;
})()"#;

/// Find the page's search field, in open shadow roots too (a search input, a combobox or a field
/// in a search landmark, then one whose name, id or form says search), mark it, focus it with its
/// content selected, and say its label and the word of five letters or more the page's headings
/// and links use most. `{found: false}` when there is none.
const FOCUS_SEARCH_JS: &str = r#"(() => {
  __DEEP__
  const cap = (s, n) => String(s || '').replace(/\s+/g, ' ').trim().slice(0, n);
  const els = all(document, []);
  for (const old of els) if (old.hasAttribute('data-craft-search')) old.removeAttribute('data-craft-search');
  const inSearch = el => { for (let n = el; n; n = up(n)) if (n.getAttribute && n.getAttribute('role') === 'search') return true; return false; };
  const score = el => {
    if (el.tagName !== 'INPUT' || !['search', 'text', ''].includes((el.getAttribute('type') || '').toLowerCase())) return 0;
    if (!shown(el) || el.disabled || el.readOnly) return 0;
    if (el.type === 'search') return 3;
    if (el.getAttribute('role') === 'combobox' || inSearch(el)) return 2;
    return /search|query|^q$/i.test([el.name, el.id, el.form ? el.form.getAttribute('action') : ''].join(' ')) ? 1 : 0;
  };
  let field = null, best = 0;
  for (const el of els) { const n = score(el); if (n > best) { best = n; field = el; } }
  if (!field) return {found: false};
  field.setAttribute('data-craft-search', '1');
  const root = field.getRootNode();
  const forLabel = field.id && root.querySelector ? root.querySelector('label[for="' + CSS.escape(field.id) + '"]') : null;
  const label = cap(field.getAttribute('aria-label') || labelled(field) || (forLabel && forLabel.innerText) || field.placeholder || field.getAttribute('title') || field.name, 80);
  const counts = {};
  for (const el of els) if (el.matches('h1, h2, h3, a') && shown(el)) for (const w of (el.innerText || '').toLowerCase().match(/\p{L}{5,}/gu) || []) counts[w] = (counts[w] || 0) + 1;
  const word = Object.entries(counts).sort((a, b) => b[1] - a[1])[0];
  field.scrollIntoView({block: 'center', behavior: 'instant'});
  field.focus();
  if (typeof field.select === 'function') field.select();
  return {found: true, label, word: word ? word[0] : ''};
})()"#;

/// What shows around the marked search field now: whether it says its list is open, how many
/// options show, how many elements show at all, and the address.
const SEARCH_STATE_JS: &str = r#"(() => {
  __DEEP__
  const els = all(document, []);
  const field = els.find(el => el.hasAttribute('data-craft-search'));
  const seen = els.filter(shown);
  return {present: !!field, expanded: field ? field.getAttribute('aria-expanded') : null, options: seen.filter(el => el.matches('[role="option"]')).length, showing: seen.length, url: location.href};
})()"#;

/// Empty the marked search field the way typing would, and take the focus from it.
const CLEAR_SEARCH_JS: &str = r#"(() => {
  __DEEP__
  const field = all(document, []).find(el => el.hasAttribute('data-craft-search'));
  if (!field) return false;
  const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
  set.call(field, '');
  field.dispatchEvent(new Event('input', {bubbles: true}));
  field.blur();
  return true;
})()"#;

/// What the page looks like around control `__I__` now.
const STATE_JS: &str = r#"(() => {
  const el = document.querySelector('[data-craft-state="__I__"]');
  const seen = d => { const r = d.getBoundingClientRect(); return r.width > 0 && r.height > 0; };
  const dialogs = Array.from(document.querySelectorAll('[role="dialog"], [role="alertdialog"], dialog[open], [aria-modal="true"]')).filter(seen).length;
  const details = el && el.parentElement && el.parentElement.tagName === 'DETAILS' ? el.parentElement.open : null;
  return {present: !!el, expanded: el ? el.getAttribute('aria-expanded') : null, selected: el ? el.getAttribute('aria-selected') : null, open: details, dialogs, url: location.href};
})()"#;

const CLICK_STATE_JS: &str = r#"(() => { const el = document.querySelector('[data-craft-state="__I__"]'); if (!el) return false; el.scrollIntoView({block: 'center'}); el.click(); return true; })()"#;

/// Before tab `__I__` is clicked: mark the tab its list has selected now, to select it again after (#335).
const REMEMBER_TAB_JS: &str = r#"(() => {
  const el = document.querySelector('[data-craft-state="__I__"]');
  if (!el || el.getAttribute('role') !== 'tab') return false;
  const list = el.closest('[role="tablist"]') || el.parentElement;
  const was = list && Array.from(list.querySelectorAll('[role="tab"][aria-selected="true"]')).find(t => t !== el);
  if (!was) return false;
  was.setAttribute('data-craft-tab-was', '__I__');
  return true;
})()"#;

const RESELECT_TAB_JS: &str = r#"(() => { const el = document.querySelector('[data-craft-tab-was="__I__"]'); if (!el) return false; el.removeAttribute('data-craft-tab-was'); el.click(); return true; })()"#;

fn press_escape(client: &mut CdpClient, dl: &Deadline) {
    for kind in ["keyDown", "keyUp"] {
        let _ = client.call("Input.dispatchKeyEvent", json!({"type": kind, "key": "Escape", "code": "Escape", "windowsVirtualKeyCode": 27}), None, dl);
    }
}

fn settle(client: &mut CdpClient, ms: u64) -> Result<(), ToolError> {
    let d = Deadline::new(Duration::from_millis(ms), crate::tool::CancelToken::default());
    client.pump_until(&d)?;
    let _ = client.take_events();
    Ok(())
}

/// What typing into the search field changed, in plain words, from the state before and after.
fn search_change(before: &Value, after: &Value) -> Vec<&'static str> {
    let n = |v: &Value, k: &str| v[k].as_u64().unwrap_or(0);
    let mut out = Vec::new();
    if n(after, "options") > n(before, "options") || (after["expanded"] == "true" && before["expanded"] != "true") {
        out.push("suggestions appeared");
    } else if n(after, "showing") >= n(before, "showing") + 3 {
        out.push("more showed");
    }
    if after["url"] != before["url"] {
        out.push("address changed");
    }
    out
}

/// What a click changed, in plain words, from the state before and after.
fn state_change(before: &Value, after: &Value) -> Vec<&'static str> {
    let mut out = Vec::new();
    if after["expanded"] == "true" && before["expanded"] != "true" {
        out.push("expanded");
    }
    if after["selected"] == "true" && before["selected"] != "true" {
        out.push("tab selected");
    }
    if after["open"] == true && before["open"] != true {
        out.push("section opened");
    }
    if after["dialogs"].as_u64().unwrap_or(0) > before["dialogs"].as_u64().unwrap_or(0) {
        out.push("dialog appeared");
    }
    if after["url"] != before["url"] {
        out.push("address changed");
    }
    out
}

/// Type into the page's search field (never submitted), picture what showed as
/// `state-<n + 1>.png`, then empty the field and put the page back. None when the page has no
/// search field, there is nothing to type, or the typing couldn't be done.
fn search_state(
    ctx: &mut Ctx<'_>,
    client: &mut CdpClient,
    out: &Path,
    start: &str,
    p: &StatesParams,
    n: usize,
    dl: &Deadline,
) -> Result<Option<Value>, ToolError> {
    let deep = |js: &str| js.replace("__DEEP__", journey::DEEP_JS);
    let field = eval(client, &deep(FOCUS_SEARCH_JS), dl)?;
    if field["found"] != true {
        return Ok(None);
    }
    let text = p.search_text.clone().unwrap_or_else(|| field["word"].as_str().unwrap_or_default().to_string());
    if text.trim().is_empty() {
        return Ok(None);
    }
    ctx.report(None, format!("Typing {text:?} into the search field…"));
    let before = eval(client, &deep(SEARCH_STATE_JS), dl)?;
    if client.call("Input.insertText", json!({"text": text}), None, dl).is_err() {
        return Ok(None);
    }
    settle(client, 1200)?;
    let after = eval(client, &deep(SEARCH_STATE_JS), dl).unwrap_or(Value::Null);
    let changed = search_change(&before, &after);
    let mut row = json!({"index": n, "kind": "search", "label": field["label"], "typed": text, "changed": changed});
    if !changed.contains(&"address changed") {
        let shot = client.call("Page.captureScreenshot", json!({"format": "png"}), None, dl)?;
        if let Some(data) = shot.get("data").and_then(Value::as_str) {
            let bytes = web_common::b64_decode(data)?;
            let name = format!("state-{}.png", n + 1);
            write_new_file(out, std::slice::from_ref(&name), &bytes).map_err(ToolError::invalid)?;
            row["file"] = json!(name);
            row["sha256"] = json!(sha256_hex(&bytes));
        }
    }
    let _ = eval(client, &deep(CLEAR_SEARCH_JS), dl);
    press_escape(client, dl);
    if after["url"].as_str().is_some_and(|u| u != start) {
        let _ = navigate(client, start, p.width, 900, 800, dl)?;
    }
    settle(client, 300)?;
    Ok(Some(row))
}

fn web_page_states_capture(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: StatesParams = parse_params(params)?;
    if !(1..=8).contains(&p.max_states) {
        return Err(ToolError::invalid("max_states must be 1..=8"));
    }
    if !(320..=3840).contains(&p.width) {
        return Err(ToolError::invalid("width must be 320..=3840"));
    }
    if p.search_text.as_ref().is_some_and(|t| t.chars().count() > 40) {
        return Err(ToolError::invalid("search_text is at most 40 characters"));
    }
    let dl = deadline(ctx, p.timeout_secs.max(30))?;
    let site = open_site(ctx, &dl)?;
    let out = create_absent_dir(&p.output_directory)?;
    let mut client = page_client(&site, &dl)?;
    let r = (|| -> Result<(String, Vec<Value>, usize), ToolError> {
        client.call("Page.enable", json!({}), None, &dl)?;
        set_viewport(&mut client, p.width, 900, &dl)?;
        settle(&mut client, 500)?;
        let start = eval(&mut client, "location.href", &dl)?.as_str().unwrap_or_default().to_string();
        let found = eval(&mut client, &FIND_STATES_JS.replace("__MAX__", &p.max_states.to_string()), &dl)?;
        let found = found.as_array().cloned().unwrap_or_default();
        let mut states = Vec::new();
        for (n, c) in found.iter().enumerate() {
            if dl.expired() {
                break;
            }
            let i = c.get("index").and_then(Value::as_u64).unwrap_or(n as u64).to_string();
            ctx.report(Some(n as f32 / found.len().max(1) as f32), format!("Opening {}…", c.get("label").and_then(Value::as_str).unwrap_or("a control")));
            let before = eval(&mut client, &STATE_JS.replace("__I__", &i), &dl)?;
            let _ = eval(&mut client, &REMEMBER_TAB_JS.replace("__I__", &i), &dl);
            if eval(&mut client, &CLICK_STATE_JS.replace("__I__", &i), &dl)? != true {
                continue;
            }
            settle(&mut client, 700)?;
            let after = eval(&mut client, &STATE_JS.replace("__I__", &i), &dl).unwrap_or(Value::Null);
            let changed = state_change(&before, &after);
            let mut row = json!({"index": n, "kind": c.get("kind"), "label": c.get("label"), "changed": changed});
            if !changed.contains(&"address changed") {
                let shot = client.call("Page.captureScreenshot", json!({"format": "png"}), None, &dl)?;
                if let Some(data) = shot.get("data").and_then(Value::as_str) {
                    let bytes = web_common::b64_decode(data)?;
                    let name = format!("state-{}.png", n + 1);
                    write_new_file(&out, std::slice::from_ref(&name), &bytes).map_err(ToolError::invalid)?;
                    row["file"] = json!(name);
                    row["sha256"] = json!(sha256_hex(&bytes));
                }
            }
            // Put the page back for the next one: close what opened, or go back to where it was.
            press_escape(&mut client, &dl);
            if after["url"].as_str().is_some_and(|u| u != start) {
                let _ = navigate(&mut client, &start, p.width, 900, 800, &dl)?;
                // The page reloaded: its marks are gone, so mark the toggles again.
                let _ = eval(&mut client, &FIND_STATES_JS.replace("__MAX__", &p.max_states.to_string()), &dl);
            } else if after["expanded"] == "true" || after["open"] == true {
                let _ = eval(&mut client, &CLICK_STATE_JS.replace("__I__", &i), &dl);
            } else if changed.contains(&"tab selected") {
                let _ = eval(&mut client, &RESELECT_TAB_JS.replace("__I__", &i), &dl);
            }
            settle(&mut client, 300)?;
            states.push(row);
        }
        if !dl.expired()
            && let Some(row) = search_state(ctx, &mut client, &out, &start, &p, states.len(), &dl)?
        {
            states.push(row);
        }
        Ok((start, states, found.len()))
    })();
    cleanup(&mut client, &dl);
    let (start, states, found) = r?;
    let json_bytes = serde_json::to_vec_pretty(&json!({"url": start, "width": p.width, "states": states})).map_err(|e| ToolError::internal(e.to_string()))?;
    write_new_file(&out, &["states.json".into()], &json_bytes).map_err(ToolError::invalid)?;
    let shown = shown_url(&start);
    let pictured = states.iter().filter(|s| s.get("file").is_some()).count();
    let searched = states.iter().find(|s| s["kind"] == "search").and_then(|s| s["typed"].as_str()).map(|t| format!(" Typed {t:?} into its search field."));
    let summary = if found == 0 {
        format!("{shown} has no menus, pop-ups, tabs or collapsed sections to open.{}", searched.unwrap_or_default())
    } else {
        format!(
            "Opened {} of {found} control(s) on {shown}; {pictured} pictured in {}.{}",
            states.iter().filter(|s| s["kind"] != "search").count(),
            out.display(),
            searched.unwrap_or_default()
        )
    };
    let browser = site.handle();
    let ev = EvidenceBuilder::new(
        "web_page_states_capture",
        Subject { kind: "website".into(), sha256: None, name: Some(shown.clone()) },
        browser_env(ctx.session.clean_room(), &browser),
    )
    .params(json!({"max_states": p.max_states, "width": p.width, "search_text": p.search_text}))
    .result(json!({"url": start, "found": found, "states": states, "folder": out.display().to_string()}))
    .summary(summary)
    .confidence(Confidence::Observed)
    .authority(Authority::OurRun)
    .limitation(Limitation::new("first_controls", "Only the first controls found are opened, one at a time; menus inside menus and hover-only menus are not."))
    .coverage(Coverage::complete(states.len() as u64))
    .location(Location::new("url", start))
    .build()?;
    Ok(ToolOutput::evidence(ev))
}

// ---------------------------------------------------------------------------------------------
// web_site_capture

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptureParams {
    /// Absolute path of a folder that does not exist yet; the capture is written there.
    pub output_directory: String,
    /// Page widths to screenshot, in CSS pixels (1..=4 widths, each 320..=3840). Default: desktop and phone.
    #[serde(default = "d_widths")]
    pub widths: Vec<u32>,
    /// Tallest screenshot in CSS pixels; longer pages are cut here (600..=20000, default 12000).
    #[serde(default = "d_max_height")]
    pub max_height: u32,
    /// Keep the rendered HTML (default true).
    #[serde(default = "yes")]
    pub include_html: bool,
    /// Photograph each width a second time a moment later and save the parts that changed by
    /// themselves (carousels, videos, animations) as noise-<width>.png, white where it changed,
    /// so comparisons can leave them out (default false).
    #[serde(default)]
    pub noise_pass: bool,
    #[serde(default = "d_timeout")]
    pub timeout_secs: u64,
}

fn d_widths() -> Vec<u32> {
    vec![1440, 390]
}
fn d_max_height() -> u32 {
    12_000
}

const MAX_HTML_BYTES: usize = 4 << 20;

/// Our read-only blueprint script. Counts styles, outlines visible structure and lists assets;
/// form fields contribute only their type, name, label and placeholder.
const BLUEPRINT_JS: &str = r#"(() => {
  const cap = (s, n) => String(s || '').replace(/\s+/g, ' ').trim().slice(0, n);
  const box = el => { const r = el.getBoundingClientRect(); return {x: Math.round(r.x + scrollX), y: Math.round(r.y + scrollY), w: Math.round(r.width), h: Math.round(r.height)}; };
  const shown = el => { const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; const s = getComputedStyle(el); return s.visibility !== 'hidden' && s.display !== 'none' && s.opacity !== '0' && (!el.checkVisibility || el.checkVisibility()); };
  const tally = (m, k) => { if (k) m[k] = (m[k] || 0) + 1; };
  const top = (m, n) => Object.entries(m).sort((a, b) => b[1] - a[1]).slice(0, n).map(([value, count]) => ({value, count}));
  const fonts = {}, sizes = {}, weights = {}, colors = {}, backgrounds = {}, radii = {}, shadows = {}, bgImages = {}, spacing = {}, maxWidths = {};
  // Colours as #rrggbb(aa), whatever notation the CSS used. Computed rgb()/rgba() is read as numbers: a canvas read
  // can be off by one per channel (Brave adds noise to it, #154). Anything else (oklch, color()…) is painted on a 1×1
  // canvas and read back.
  const cv = document.createElement('canvas'); cv.width = cv.height = 1; const g = cv.getContext('2d', {willReadFrequently: true}); const seenHex = {};
  const hex = c => { if (!c) return c; if (c in seenHex) return seenHex[c]; const h2 = x => x.toString(16).padStart(2, '0'); const m = /^rgba?\((\d+), (\d+), (\d+)(?:, ([\d.]+))?\)$/.exec(c); if (m) { const a = m[4] === undefined ? 1 : +m[4]; return seenHex[c] = '#' + h2(+m[1]) + h2(+m[2]) + h2(+m[3]) + (a < 1 ? h2(Math.round(a * 255)) : ''); } if (!g) return c; g.clearRect(0, 0, 1, 1); g.fillStyle = '#000'; g.fillStyle = c; g.fillRect(0, 0, 1, 1); const d = g.getImageData(0, 0, 1, 1).data; return seenHex[c] = '#' + h2(d[0]) + h2(d[1]) + h2(d[2]) + (d[3] < 255 ? h2(d[3]) : ''); };
  // Elements in page order, each open shadow root's content right after its host: components (Lit and
  // the like) keep their content there. Closed roots can't be read; hosts that look like one are counted.
  const openRoots = [];
  const composed = (root, limit) => {
    const out = [];
    const go = r => { for (const el of r.querySelectorAll('*')) { if (out.length >= limit) return; out.push(el); if (el.shadowRoot) { if (root === document) openRoots.push(el.shadowRoot); go(el.shadowRoot); } } };
    if (root) go(root);
    return out;
  };
  const all = composed(document, 200000);
  const deepAll = (sel, n) => { const out = []; for (const el of all) { if (out.length >= n) break; if (el.matches(sel)) out.push(el); } return out; };
  const up = n => n.parentElement || (n.parentNode && n.parentNode.host) || null;
  // The visible text: innerText stops at a shadow root, so each open root's own text follows the
  // page's, in the order of their hosts (a nested root's text comes after its parent's).
  const pageText = () => {
    const parts = [document.body ? document.body.innerText : ''];
    let size = parts[0] ? parts[0].length : 0;
    for (const r of openRoots) {
      if (size >= 30000) break;
      for (const c of r.children) { if (['STYLE', 'SCRIPT', 'TEMPLATE'].includes(c.tagName)) continue; const t = c.innerText; if (t && t.trim()) { parts.push(t); size += t.length; } }
    }
    return parts.filter(t => t && t.trim()).join('\n');
  };
  let closedHosts = 0;
  for (const el of all) { if (closedHosts >= 1000) break; if (el.tagName.includes('-') && !el.shadowRoot && !el.childElementCount && !el.textContent.trim() && shown(el)) closedHosts++; }
  const els = [document.documentElement, document.body, ...composed(document.body, 12000)].filter(Boolean);
  for (const el of els) {
    if (!shown(el)) continue;
    const s = getComputedStyle(el);
    const text = Array.from(el.childNodes).some(n => n.nodeType === 3 && n.textContent.trim());
    if (text) { tally(fonts, s.fontFamily); tally(sizes, s.fontSize); tally(weights, s.fontWeight); tally(colors, hex(s.color)); }
    if (s.backgroundColor && s.backgroundColor !== 'rgba(0, 0, 0, 0)' && s.backgroundColor !== 'transparent') tally(backgrounds, hex(s.backgroundColor));
    for (const k of ['paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft', 'rowGap', 'columnGap', 'marginTop', 'marginBottom']) { const v = s[k]; if (v && v.endsWith('px') && v !== '0px') tally(spacing, v); }
    if (s.maxWidth.endsWith('px')) tally(maxWidths, s.maxWidth);
    // A pill made with calc(infinity * 1px) comes back clamped as 3.35544e+07px: record it as 9999px (#155).
    if (s.borderRadius && s.borderRadius !== '0px') tally(radii, s.borderRadius.replace(/[\d.]+(?:e\+?\d+)?px/g, n => parseFloat(n) > 9999 ? '9999px' : n));
    if (s.boxShadow && s.boxShadow !== 'none') tally(shadows, s.boxShadow);
    const bi = s.backgroundImage; if (bi && bi !== 'none') for (const m of bi.matchAll(/url\(["']?([^"')]+)["']?\)/g)) tally(bgImages, m[1]);
  }
  const vars = {}, bps = {}; let ruleCount = 0, unreadable = 0;
  const bp = (side, n, unit) => tally(bps, side + '-width ' + Math.round(parseFloat(n) * (unit === 'px' ? 1 : 16)) + 'px');
  // A component's sheet: its :host variables don't replace the page's :root ones.
  const walk = (rules, depth, component) => {
    if (!rules || depth > 6) return;
    for (const r of Array.from(rules)) {
      if (++ruleCount > 40000) return;
      if (r.selectorText && /:root|:host/.test(r.selectorText) && r.style) for (const name of Array.from(r.style)) if (name.startsWith('--') && Object.keys(vars).length < 300 && !(component && name in vars)) vars[name] = cap(r.style.getPropertyValue(name), 120);
      const media = r.media && r.media.mediaText ? r.media.mediaText : (r.conditionText || '');
      for (const m of media.matchAll(/(min|max)-width\s*:\s*([\d.]+)(px|r?em)/g)) bp(m[1], m[2], m[3]);
      for (const m of media.matchAll(/width\s*(>=?|<=?)\s*([\d.]+)(px|r?em)/g)) bp(m[1][0] === '>' ? 'min' : 'max', m[2], m[3]);
      if (r.cssRules) walk(r.cssRules, depth + 1, component);
    }
  };
  // Components share their constructed sheets, so each sheet is read once.
  const seenSheets = new Set();
  const readSheets = (sheets, component) => { for (const sheet of Array.from(sheets || [])) { if (seenSheets.has(sheet)) continue; seenSheets.add(sheet); let rules; try { rules = sheet.cssRules; } catch (e) { unreadable++; continue; } walk(rules, 0, component); } };
  readSheets(document.styleSheets, false); readSheets(document.adoptedStyleSheets, false);
  for (const r of openRoots) { if (ruleCount > 40000) break; readSheets(r.styleSheets, true); readSheets(r.adoptedStyleSheets, true); }
  const breakpoints = Object.entries(bps).map(([query, count]) => ({query, px: parseInt(query.split(' ')[1], 10), count})).sort((a, b) => a.px - b.px).slice(0, 24);
  const PICK = 'header,nav,main,footer,section,article,aside,form,h1,h2,h3,h4,p,ul,ol,button,a,img,picture,video,svg,input,select,textarea,[role]';
  const outline = [];
  for (const el of all) {
    if (outline.length >= 900) break;
    if (!el.matches(PICK) || !shown(el)) continue;
    let depth = 0; for (let p = up(el); p; p = up(p)) depth++;
    const tag = el.tagName.toLowerCase();
    const item = {tag, depth, box: box(el)};
    const role = el.getAttribute('role'); if (role) item.role = role;
    if (el.id) item.id = cap(el.id, 60);
    const cls = typeof el.className === 'string' ? cap(el.className, 120) : ''; if (cls) item.class = cls;
    if (!['main', 'section', 'article', 'header', 'footer', 'nav', 'aside', 'form', 'ul', 'ol', 'svg', 'picture', 'video'].includes(tag)) { const t = cap(el.innerText, 200); if (t) item.text = t; }
    if (tag === 'a') item.href = el.href || '';
    if (tag === 'img') { item.src = el.currentSrc || el.src || ''; item.alt = cap(el.alt, 120); }
    if (['input', 'select', 'textarea'].includes(tag)) { item.type = el.type || tag; item.name = cap(el.name, 60); item.placeholder = cap(el.placeholder, 80); }
    const s = getComputedStyle(el);
    if (['header', 'nav', 'main', 'footer', 'section', 'article', 'aside', 'form', 'ul', 'ol'].includes(tag)) { item.display = s.display; if (s.display.includes('flex')) item.flex = s.flexDirection; if (s.display.includes('grid')) item.grid = cap(s.gridTemplateColumns, 120); }
    item.font = s.fontSize + ' ' + s.fontWeight; item.color = hex(s.color); if (s.backgroundColor !== 'rgba(0, 0, 0, 0)') item.background = hex(s.backgroundColor);
    outline.push(item);
  }
  const labelFor = el => { if (el.labels && el.labels[0]) return cap(el.labels[0].innerText, 80); return cap(el.getAttribute('aria-label'), 80); };
  const forms = deepAll('form', 30).map(f => ({action: f.getAttribute('action') || '', method: (f.method || 'get').toLowerCase(),
    fields: Array.from(f.elements).slice(0, 60).map(e => ({tag: e.tagName.toLowerCase(), type: e.type || '', name: cap(e.name, 60), label: labelFor(e), placeholder: cap(e.placeholder, 80), required: !!e.required}))}));
  const links = deepAll('a[href],area[href]', 2000).filter(a => typeof a.href === 'string').slice(0, 400).map(a => ({text: cap(a.innerText || a.getAttribute('aria-label'), 100), href: a.href}));
  const meta = n => { const m = document.querySelector(`meta[name="${n}"],meta[property="${n}"]`); return m ? cap(m.content, 300) : ''; };
  const icons = Array.from(document.querySelectorAll('link[rel~="icon"],link[rel="apple-touch-icon"]')).map(l => l.href).slice(0, 10);
  // What the page loaded, without query strings (they can carry ids).
  const resources = (performance.getEntriesByType ? performance.getEntriesByType('resource') : []).map(e => ({url: String(e.name).split(/[?#]/)[0], type: e.initiatorType, bytes: e.transferSize || 0}));
  const images = deepAll('img', 3000);
  // Accessibility at a glance: controls a screen reader can't name, pictures without alt text,
  // the landmarks and the page language.
  const CONTROLS = 'a[href],button,input:not([type="hidden"]),select,textarea,[role="button"],[role="link"],[role="tab"],[role="menuitem"]';
  const named = el => !!(cap(el.innerText, 80) || el.getAttribute('aria-label') || el.getAttribute('aria-labelledby') || el.getAttribute('title')
    || (el.labels && el.labels.length) || el.getAttribute('placeholder') || (['submit', 'button', 'reset'].includes(el.type) && el.value)
    || el.querySelector('img[alt]:not([alt=""]),svg title,[aria-label]'));
  const unnamed = deepAll(CONTROLS, 3000).filter(el => shown(el) && !named(el));
  const landmarks = {};
  for (const el of deepAll('header,nav,main,footer,aside,form[aria-label],[role="banner"],[role="navigation"],[role="main"],[role="contentinfo"],[role="complementary"],[role="search"]', 2000)) {
    if (!shown(el)) continue;
    const role = el.getAttribute('role') || {header: 'banner', nav: 'navigation', main: 'main', footer: 'contentinfo', aside: 'complementary', form: 'form'}[el.tagName.toLowerCase()];
    tally(landmarks, role);
  }
  // Text contrast (the WCAG ratio) against the colour behind the text: the element's own background
  // and its ancestors', blended down to the first opaque one over a white canvas. Text over a
  // picture or a gradient, or in a colour this can't read as rgb, is counted apart, not judged.
  const rgba = c => { const m = /^rgba?\((\d+), (\d+), (\d+)(?:, ([\d.]+))?\)$/.exec(c || ''); return m ? [+m[1], +m[2], +m[3], m[4] === undefined ? 1 : +m[4]] : null; };
  const over = (t, u) => [0, 1, 2].map(i => t[i] * t[3] + u[i] * (1 - t[3])).concat(1);
  const lum = c => { const f = v => { v /= 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }; return 0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2]); };
  const rgbHex = c => '#' + c.slice(0, 3).map(v => Math.round(v).toString(16).padStart(2, '0')).join('');
  const seenBehind = new Map();
  const behind = el => {
    if (!el) return [255, 255, 255, 1];
    if (seenBehind.has(el)) return seenBehind.get(el);
    const s = getComputedStyle(el), b = rgba(s.backgroundColor);
    let c;
    if ((s.backgroundImage && s.backgroundImage !== 'none') || (!b && s.backgroundColor && s.backgroundColor !== 'transparent')) c = null;
    else if (b && b[3] >= 1) c = b;
    else { const u = behind(up(el)); c = u && b && b[3] > 0 ? over(b, u) : u; }
    seenBehind.set(el, c); return c;
  };
  const ownText = el => { for (const n of el.childNodes) if (n.nodeType === 3 && n.textContent.trim()) return true; return false; };
  let textChecked = 0, contrastUnknown = 0; const lowContrast = [];
  for (const el of all) {
    if (textChecked + contrastUnknown >= 4000) break;
    if (el.namespaceURI !== 'http://www.w3.org/1999/xhtml' || !ownText(el) || !shown(el) || el.closest('[disabled],[aria-disabled="true"]')) continue;
    const s = getComputedStyle(el), fg = rgba(s.color), bg = behind(el);
    if (!fg || !bg) { contrastUnknown++; continue; }
    if (fg[3] === 0) continue;
    const text = over(fg, bg), [hi, lo] = [lum(text), lum(bg)].sort((a, b) => b - a), ratio = (hi + 0.05) / (lo + 0.05);
    const px = parseFloat(s.fontSize), large = px >= 24 || (px >= 18.66 && +s.fontWeight >= 700);
    textChecked++;
    if (ratio < (large ? 3 : 4.5)) lowContrast.push({element: el.tagName.toLowerCase() + (typeof el.className === 'string' && el.className.trim() ? '.' + el.className.trim().split(/\s+/)[0] : ''),
      ratio: Math.round(ratio * 100) / 100, needs: large ? 3 : 4.5, color: rgbHex(text), background: rgbHex(bg)});
  }
  // One example per pair of colours, the faintest first, with how many texts share it.
  const pairs = new Map();
  for (const l of lowContrast) { const k = l.color + l.background + l.needs; const p = pairs.get(k); if (p) p.count++; else pairs.set(k, Object.assign({count: 1}, l)); }
  const lowExamples = Array.from(pairs.values()).sort((a, b) => a.ratio - b.ratio);
  const a11y = {lang: document.documentElement.lang || '', landmarks, unnamed_controls: unnamed.length,
    unnamed_examples: unnamed.slice(0, 5).map(el => el.tagName.toLowerCase() + (el.className && typeof el.className === 'string' ? '.' + cap(el.className, 40).split(' ')[0] : '')),
    images_without_alt: images.filter(i => shown(i) && !i.hasAttribute('alt')).length,
    text_checked: textChecked, low_contrast: lowContrast.length, low_contrast_examples: lowExamples.slice(0, 5), contrast_unknown: contrastUnknown};
  const fontFaces = []; try { document.fonts.forEach(f => { if (fontFaces.length < 80) fontFaces.push({family: f.family, weight: f.weight, style: f.style, status: f.status}); }); } catch (e) {}
  return {
    url: location.href, title: document.title, lang: document.documentElement.lang || '',
    viewport: {w: innerWidth, h: innerHeight}, page: {w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight},
    meta: {description: meta('description'), theme_color: meta('theme-color'), og_title: meta('og:title'), og_image: meta('og:image'), icons},
    design: {fonts: top(fonts, 12), font_sizes: top(sizes, 16), font_weights: top(weights, 8), text_colors: top(colors, 16), backgrounds: top(backgrounds, 16), radii: top(radii, 10), shadows: top(shadows, 8), spacing: top(spacing, 24), max_widths: top(maxWidths, 8), css_variables: vars, font_faces: fontFaces},
    breakpoints, unreadable_stylesheets: unreadable, a11y,
    outline, forms, links, shadow: {open_roots: openRoots.length, closed_hosts: closedHosts},
    assets: {
      images: images.slice(0, 300).map(i => ({src: i.currentSrc || i.src, w: i.naturalWidth, h: i.naturalHeight, alt: cap(i.alt, 120)})),
      background_images: top(bgImages, 60).map(x => x.value),
      stylesheets: Array.from(document.styleSheets).map(s => s.href).filter(Boolean).slice(0, 80),
      scripts: Array.from(document.scripts).map(s => s.src).filter(Boolean).slice(0, 120),
      inline_scripts: Array.from(document.scripts).filter(s => !s.src).length,
      fonts: resources.filter(r => /\.(woff2?|ttf|otf|eot)$/i.test(r.url) || r.url.includes('fonts.googleapis.com')).map(r => r.url).slice(0, 40),
      requests: resources.slice(0, 300)
    },
    headings: deepAll('h1,h2,h3', 120).map(h => ({level: Number(h.tagName[1]), text: cap(h.innerText, 160)})),
    text: cap(pageText(), 30000)
  };
})()"#;

/// Scroll through the page as a visitor would, so sections that appear as they come into view and
/// lazy images are there for the full-page picture (#160), then back to the top. At most a few
/// seconds; returns how far it scrolled.
const REVEAL_JS: &str = r#"(async () => {
  const pause = ms => new Promise(r => setTimeout(r, ms));
  const end = Date.now() + 5000;
  const step = Math.max(200, Math.floor(innerHeight * 0.8));
  const bottom = () => Math.max(document.documentElement.scrollHeight, document.body ? document.body.scrollHeight : 0) - innerHeight;
  let y = 0;
  while (y < bottom() && Date.now() < end) {
    y = Math.min(y + step, bottom());
    scrollTo(0, y);
    await pause(120);
  }
  for (const img of document.images) if (img.loading === 'lazy') img.loading = 'eager';
  const loading = [...document.images].filter(i => !i.complete).map(i => new Promise(r => { i.addEventListener('load', r, {once: true}); i.addEventListener('error', r, {once: true}); }));
  await Promise.race([Promise.all(loading), pause(Math.max(300, end - Date.now()))]);
  scrollTo(0, 0);
  await pause(200);
  return y;
})()"#;

/// The window's height while a page is photographed.
const VIEW_HEIGHT: u32 = 900;
/// A full-page picture with more than this share of its height in flat bands is taken again in bands.
const FLAT_SHARE_SUSPECT: f64 = 0.35;

/// Before photographing in bands, at the top of the page: remember what doesn't scroll with it
/// (fixed or sticky), with where it is now and its own inline `visibility`.
const PIN_JS: &str = r#"(() => {
  const pinned = []; let seen = 0;
  const go = r => { for (const el of r.querySelectorAll('*')) { if (++seen > 20000) return; const p = getComputedStyle(el).position; if (p === 'fixed' || p === 'sticky') pinned.push({el, y0: el.getBoundingClientRect().top + scrollY, vis: el.style.getPropertyValue('visibility'), pri: el.style.getPropertyPriority('visibility')}); if (el.shadowRoot) go(el.shadowRoot); } };
  go(document);
  window.__craftAnythingPinned = pinned;
  return pinned.length;
})()"#;

/// Scroll to `__Y__` as a visitor would, let the page draw and its pictures in view load (at most
/// 1.5 s). What has left the place it had at the
/// top of the page (it is fixed, or stuck) is hidden for this band, so it is in the picture once.
/// Returns where the page really scrolled to.
const BAND_JS: &str = r#"(async () => {
  const pause = ms => new Promise(r => setTimeout(r, ms));
  const frame = () => new Promise(r => requestAnimationFrame(() => r()));
  scrollTo(0, __Y__); await frame(); await pause(150);
  // Pictures that load as they come into view, in shadow roots too: wait for the ones in this window.
  const waiting = []; let seen = 0;
  const go = r => { for (const el of r.querySelectorAll('*')) { if (++seen > 20000) return; if (el.tagName === 'IMG' && !el.complete) { const b = el.getBoundingClientRect(); if (b.bottom > 0 && b.top < innerHeight) waiting.push(new Promise(d => { el.addEventListener('load', d, {once: true}); el.addEventListener('error', d, {once: true}); })); } if (el.shadowRoot) go(el.shadowRoot); } };
  go(document);
  if (waiting.length) await Promise.race([Promise.all(waiting), pause(1500)]);
  for (const p of (window.__craftAnythingPinned || [])) {
    const moved = scrollY > 0 && Math.abs(p.el.getBoundingClientRect().top + scrollY - p.y0) > 2;
    if (moved) p.el.style.setProperty('visibility', 'hidden', 'important');
    else if (p.vis) p.el.style.setProperty('visibility', p.vis, p.pri); else p.el.style.removeProperty('visibility');
  }
  await frame();
  return scrollY;
})()"#;

/// Put back what [`BAND_JS`] hid, and the page at its top.
const UNPIN_JS: &str = r#"(async () => {
  for (const p of (window.__craftAnythingPinned || [])) { if (p.vis) p.el.style.setProperty('visibility', p.vis, p.pri); else p.el.style.removeProperty('visibility'); }
  delete window.__craftAnythingPinned;
  scrollTo(0, 0);
  await new Promise(r => setTimeout(r, 150));
  return true;
})()"#;

/// Our read-only layout script, run at every width: the page's width (wider than the window means
/// it scrolls sideways), heading and text sizes, and each landmark and section's padding, gap,
/// background, content column and how many items sit side by side.
const LAYOUT_JS: &str = r#"(() => {
  const cap = (s, n) => String(s || '').replace(/\s+/g, ' ').trim().slice(0, n);
  const px = v => Math.round(parseFloat(v) || 0);
  const shown = el => { const r = el.getBoundingClientRect(); if (r.width < 1 || r.height < 1) return false; const s = getComputedStyle(el); return s.visibility !== 'hidden' && s.display !== 'none' && s.opacity !== '0' && (!el.checkVisibility || el.checkVisibility()); };
  const size = sel => { const el = Array.from(document.querySelectorAll(sel)).slice(0, 50).find(shown); return el ? px(getComputedStyle(el).fontSize) : 0; };
  const cv = document.createElement('canvas'); cv.width = cv.height = 1; const g = cv.getContext('2d', {willReadFrequently: true});
  // Computed rgb()/rgba() is read as numbers; a canvas read can be off by one (#154).
  const hex = c => { const h2 = x => x.toString(16).padStart(2, '0'); const m = /^rgba?\((\d+), (\d+), (\d+)(?:, ([\d.]+))?\)$/.exec(c); if (m) { const a = m[4] === undefined ? 1 : +m[4]; return '#' + h2(+m[1]) + h2(+m[2]) + h2(+m[3]) + (a < 1 ? h2(Math.round(a * 255)) : ''); } if (!g) return c; g.clearRect(0, 0, 1, 1); g.fillStyle = '#000'; g.fillStyle = c; g.fillRect(0, 0, 1, 1); const d = g.getImageData(0, 0, 1, 1).data; return '#' + h2(d[0]) + h2(d[1]) + h2(d[2]) + (d[3] < 255 ? h2(d[3]) : ''); };
  // Items side by side: the most children of one box sharing a top edge, counting only children at
  // least 15% of the block's width (cards and columns, not inline links or logo strips).
  const perRow = (root, width) => {
    let best = 1, seen = 0;
    for (const c of [root, ...root.querySelectorAll('*')]) {
      if (++seen > 400) break;
      if (c.children.length < 2) continue;
      const kids = Array.from(c.children).filter(k => shown(k) && k.getBoundingClientRect().width >= Math.max(40, width * 0.15));
      if (kids.length < 2) continue;
      const top = kids[0].getBoundingClientRect().top;
      best = Math.max(best, kids.filter(k => Math.abs(k.getBoundingClientRect().top - top) <= 2).length);
    }
    return best;
  };
  const blocks = [];
  for (const el of document.querySelectorAll('header,nav,main,footer,section,article,aside')) {
    if (blocks.length >= 60) break;
    if (!shown(el)) continue;
    const r = el.getBoundingClientRect(); if (r.height < 24) continue;
    const s = getComputedStyle(el);
    const b = {tag: el.tagName.toLowerCase(), y: Math.round(r.top + scrollY), w: Math.round(r.width), h: Math.round(r.height),
      pad: [px(s.paddingTop), px(s.paddingRight), px(s.paddingBottom), px(s.paddingLeft)], per_row: perRow(el, r.width)};
    const head = Array.from(el.querySelectorAll('h1,h2,h3')).slice(0, 20).find(shown); if (head) b.heading = cap(head.innerText, 80);
    const gap = px(s.columnGap) || px(s.rowGap); if (gap) b.gap = gap;
    if (s.backgroundColor !== 'rgba(0, 0, 0, 0)') b.background = hex(s.backgroundColor);
    // The content column: the first box (itself or inside) held to a maximum width in px.
    let n = 0;
    for (const c of [el, ...el.querySelectorAll('*')]) {
      if (++n > 80) break;
      const cs = getComputedStyle(c);
      if (cs.maxWidth.endsWith('px') && !['IMG', 'SVG', 'VIDEO', 'PICTURE'].includes(c.tagName.toUpperCase()) && shown(c)) {
        const cr = c.getBoundingClientRect();
        b.column = {max_w: px(cs.maxWidth), x: Math.round(cr.left), w: Math.round(cr.width), pad: [px(cs.paddingLeft), px(cs.paddingRight)]};
        break;
      }
    }
    blocks.push(b);
  }
  return {page_w: document.documentElement.scrollWidth, page_h: document.documentElement.scrollHeight,
    type: {h1: size('h1'), h2: size('h2'), h3: size('h3'), body: size('p')}, blocks};
})()"#;

fn web_site_capture(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let p: CaptureParams = parse_params(params)?;
    if p.widths.is_empty() || p.widths.len() > 4 || p.widths.iter().any(|w| !(320..=3840).contains(w)) {
        return Err(ToolError::invalid("widths must hold 1..=4 values, each 320..=3840"));
    }
    if !(600..=20_000).contains(&p.max_height) {
        return Err(ToolError::invalid("max_height must be 600..=20000"));
    }
    let mut p = p;
    // Clean room keeps what a visitor sees: never the page's own code.
    let visitor = ctx.session.clean_room();
    p.include_html &= !visitor;
    let dl = deadline(ctx, p.timeout_secs.max(30))?;
    let site = open_site(ctx, &dl)?;
    let mut client = page_client(&site, &dl)?;
    // A site that didn't open leaves the browser on its own error page (or a blank one), which
    // would be captured as if it were the site (#158).
    let here = eval(&mut client, "location.href", &dl).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
    if here.is_empty() || here.starts_with("chrome-error:") || here == "about:blank" {
        cleanup(&mut client, &dl);
        return Err(ToolError::new(ErrorCode::NotFound, "The website didn't open, so there's nothing to capture: the browser is showing its own error page.")
            .with_remediation("Open the site again with web_site_open and make sure it loads."));
    }
    let out = match create_absent_dir(&p.output_directory) {
        Ok(out) => out,
        Err(e) => {
            cleanup(&mut client, &dl);
            return Err(e);
        }
    };
    let r = capture_into(ctx, &mut client, &out, &p, &dl);
    // Leave the page at the first (usually desktop) width for the tools that follow.
    if let Some(w) = p.widths.first() {
        let _ = set_viewport(&mut client, *w, 900, &dl);
    }
    // At that width, walk the page with Tab, as a keyboard user would.
    let keyboard = if r.is_ok() { keyboard_walk(&mut client, &dl).ok() } else { None };
    cleanup(&mut client, &dl);
    let (mut blueprint, shots, html) = r?;
    if let (Some(k), Some(Value::Object(a))) = (keyboard, blueprint.get_mut("a11y")) {
        a.insert("keyboard".into(), k);
    }
    let html_bytes = html.0;
    if visitor {
        visitor_view(&mut blueprint);
    } else if let (Value::Object(b), Ok(h)) = (&mut blueprint, site.health.lock()) {
        b.insert("health".into(), h.clone());
    }
    let signed_in = site.signed_in.is_some();
    let sign_in_page = has_password_field(&blueprint);
    if let Value::Object(b) = &mut blueprint {
        if signed_in && let Some(u) = b.get("url").and_then(Value::as_str).map(shown_url) {
            b.insert("url".into(), json!(u));
        }
        b.insert("signed_in".into(), json!(signed_in));
        b.insert("signed_in_site".into(), json!(site.signed_in));
        b.insert("sign_in_page".into(), json!(sign_in_page));
    }

    let url = blueprint.get("url").and_then(Value::as_str).unwrap_or_default().to_string();
    let shown = shown_url(&url);
    let n = |k: &str| blueprint.get(k).and_then(Value::as_array).map_or(0, Vec::len);
    let images = blueprint.pointer("/assets/images").and_then(Value::as_array).map_or(0, Vec::len);
    let readme = readme(&blueprint, &shots, html, visitor);
    write_new_file(&out, &["README.md".into()], readme.as_bytes()).map_err(ToolError::invalid)?;
    let capture_json = serde_json::to_vec_pretty(&blueprint).map_err(|e| ToolError::internal(e.to_string()))?;
    write_new_file(&out, &["capture.json".into()], &capture_json).map_err(ToolError::invalid)?;

    let summary = format!(
        "Captured {shown}: {} screenshot(s), {} outline items, {} headings, {images} images, {} links, {} forms → {}.",
        shots.len(),
        n("outline"),
        n("headings"),
        n("links"),
        n("forms"),
        out.display()
    );
    let browser = site.handle();
    let ev = EvidenceBuilder::new("web_site_capture", Subject { kind: "website".into(), sha256: None, name: Some(shown.clone()) }, browser_env(ctx.session.clean_room(), &browser))
        .params(json!({"widths": p.widths, "max_height": p.max_height, "include_html": p.include_html, "noise_pass": p.noise_pass, "visitor_view": visitor}))
        .result(json!({
            "output_directory": out.display().to_string(),
            "url": url,
            "title": blueprint.get("title"),
            "screenshots": shots,
            "html_bytes": html_bytes,
            "counts": {"outline": n("outline"), "headings": n("headings"), "links": n("links"), "forms": n("forms"), "images": images},
            "design": blueprint.get("design"),
            "shadow": blueprint.get("shadow"),
            "files": ["README.md", "capture.json"],
            "signed_in": signed_in,
            "signed_in_site": site.signed_in,
            "sign_in_page": sign_in_page,
        }))
        .summary(summary)
        .confidence(Confidence::Observed)
        .authority(Authority::OurRun)
        .limitation(Limitation::new("one_state", "One state of the page: menus, dialogs and content behind interactions are not opened."))
        .limitation(if visitor {
            Limitation::new("visitor_view", "Clean room: only what a visitor sees was kept (screenshots, visible text, measured layout and styles); not the page's HTML, CSS rules, scripts, requests or errors.")
        } else {
            Limitation::new("cross_origin_css", "CSS custom properties and breakpoints from cross-origin stylesheets can't be read.")
        })
        .limitations(html_cut(html))
        .limitations(closed_shadow_roots(&blueprint))
        .limitations(photographed_in_bands(&shots))
        .limitations(site.signed_in.as_deref().map(|s| profile_limitation(Some(s))))
        .limitations(sign_in_page.then(|| if signed_in { Limitation::new("sign_in_ended", SIGN_IN_ENDED) } else { Limitation::new("sign_in_page", SIGN_IN_HINT) }))
        .coverage(Coverage::complete(p.widths.len() as u64))
        .location(Location::new("url", shown))
        .location(Location::new("path", out.display().to_string()))
        .build()?;
    Ok(ToolOutput::evidence(ev))
}

/// Said when a page captured signed out asks for a password.
const SIGN_IN_HINT: &str =
    "This may be a sign-in page (it asks for a password). To study what's behind it, sign in once with web_site_sign_in and capture again.";
/// Said when a page captured with a saved sign-in still asks for a password.
const SIGN_IN_ENDED: &str = "This was opened with your saved sign-in but may still be a sign-in page (it asks for a password): the site may have ended that sign-in. Sign in again with web_site_sign_in and capture again.";

/// The page asks for a password: a field of type password in its forms or its outline.
fn has_password_field(blueprint: &Value) -> bool {
    let password = |f: &Value| f.get("type").and_then(Value::as_str) == Some("password");
    let in_forms = blueprint
        .get("forms")
        .and_then(Value::as_array)
        .is_some_and(|forms| forms.iter().any(|f| f.get("fields").and_then(Value::as_array).is_some_and(|fields| fields.iter().any(password))));
    in_forms || blueprint.get("outline").and_then(Value::as_array).is_some_and(|items| items.iter().any(password))
}

/// The blueprint, the screenshots, and the HTML's bytes as (saved, the page's).
type Captured = (Value, Vec<Value>, (usize, usize));

/// When page.html holds only the first part of the page's HTML (#336).
fn html_cut((kept, total): (usize, usize)) -> Option<Limitation> {
    (kept < total).then(|| {
        Limitation::new(
            "html_cut",
            format!("page.html holds the first {kept} of the page's {total} bytes of HTML: it was cut at the {} MiB limit.", MAX_HTML_BYTES >> 20),
        )
    })
}

/// Components that seem to keep their content in closed shadow roots, which our script in the
/// page can't read.
fn closed_shadow_roots(blueprint: &Value) -> Option<Limitation> {
    let n = blueprint.pointer("/shadow/closed_hosts").and_then(Value::as_u64).unwrap_or(0);
    (n > 0).then(|| {
        Limitation::new(
            "closed_shadow_roots",
            format!("{n} component(s) seem to keep their content in closed shadow roots, so their structure wasn't read; the screenshots still show them."),
        )
    })
}

fn window_picture(client: &mut CdpClient, dl: &Deadline) -> Result<web_common::Image, ToolError> {
    let shot = client.call("Page.captureScreenshot", json!({"format": "png"}), None, dl)?;
    let data = shot.get("data").and_then(Value::as_str).ok_or_else(|| ToolError::new(ErrorCode::GarbledOutput, "the browser returned no screenshot"))?;
    web_common::decode_png(&web_common::b64_decode(data)?)
}

/// Photograph the page as a visitor scrolling it sees it: one window at a time, the bands stitched
/// into one picture `height` tall. For a page that draws only what is in view (a virtualised list),
/// where one tall picture comes out blank. The page is put back as it was, whatever happens.
fn photograph_in_bands(client: &mut CdpClient, w: u32, height: u64, dl: &Deadline) -> Result<Vec<u8>, ToolError> {
    let (width, view) = (w as usize, u64::from(VIEW_HEIGHT));
    if u64::from(w).saturating_mul(height).saturating_mul(4) > web_common::MAX_PIXELS_WORKING_SET {
        return Err(ToolError::new(ErrorCode::OverLimit, "the page is too large to photograph in bands"));
    }
    let _ = eval(client, PIN_JS, dl);
    let stitch = |client: &mut CdpClient| -> Result<Vec<u8>, ToolError> {
        let mut rgba = vec![255u8; width * height as usize * 4];
        let mut top = 0u64;
        while top < height {
            let at = eval(client, &BAND_JS.replace("__Y__", &top.to_string()), dl)?.as_f64().unwrap_or(0.0).max(0.0).round() as u64;
            let img = window_picture(client, dl)?;
            if img.width != w {
                return Err(ToolError::new(ErrorCode::GarbledOutput, "the window's picture is not as wide as the window"));
            }
            // At the end of the page the window stops short of `top`: the band is lower in its picture.
            let skip = top.saturating_sub(at);
            let rows = (height - top).min(view).min(u64::from(img.height).saturating_sub(skip));
            for row in 0..rows as usize {
                let (from, to) = ((skip as usize + row) * width * 4, (top as usize + row) * width * 4);
                rgba[to..to + width * 4].copy_from_slice(&img.rgba[from..from + width * 4]);
            }
            top += view;
        }
        web_common::encode_png(&web_common::Image { width: w, height: height as u32, rgba })
    };
    let picture = stitch(client);
    let _ = eval(client, UNPIN_JS, dl);
    picture
}

fn flat_share(png: &[u8]) -> f64 {
    web_common::decode_png(png).map(|img| web_common::flat_band_share(&img, 64)).unwrap_or(0.0)
}

/// Said when a screenshot had to be taken one window at a time.
fn photographed_in_bands(shots: &[Value]) -> Option<Limitation> {
    shots.iter().any(|s| s["in_bands"] == true).then(|| {
        Limitation::new(
            "photographed_in_bands",
            "The page draws only what is in view, so a screenshot was taken one window at a time while scrolling and stitched; what doesn't scroll with the page is shown once, where it was at the top.",
        )
    })
}

/// Most Tab presses a capture makes: a page with more controls is walked through its first ones.
const KEYBOARD_TABS: usize = 80;

/// Before the Tab walk: take the focus off the page, go to its top, forget earlier marks, and
/// count the visible controls a keyboard should reach (in open shadow roots too).
const KEYBOARD_START_JS: &str = r#"(() => {
  __DEEP__
  const els = all(document, []);
  for (const el of els) if (el.hasAttribute('data-craft-tab')) el.removeAttribute('data-craft-tab');
  let a = document.activeElement; while (a && a.shadowRoot && a.shadowRoot.activeElement) a = a.shadowRoot.activeElement;
  if (a && a.blur) a.blur();
  scrollTo(0, 0);
  const CONTROLS = 'a[href], button, input:not([type="hidden"]), select, textarea, summary, [tabindex]:not([tabindex="-1"]), [contenteditable="true"]';
  const reachable = el => el.matches(CONTROLS) && !el.disabled && el.getAttribute('tabindex') !== '-1' && !el.closest('[inert]') && shown(el);
  return {controls: els.filter(reachable).length};
})()"#;

/// After a Tab press: the element that has the focus (through open shadow roots), whether it was
/// reached before (`again`), and whether its focus can be seen (an outline or a shadow on it or
/// on its parent). Marks it with its order.
const KEYBOARD_FOCUS_JS: &str = r#"(() => {
  let a = document.activeElement; while (a && a.shadowRoot && a.shadowRoot.activeElement) a = a.shadowRoot.activeElement;
  if (!a || a === document.body || a === document.documentElement) return {on_page: false};
  const again = a.hasAttribute('data-craft-tab');
  if (!again) a.setAttribute('data-craft-tab', '__N__');
  const ring = el => { if (!el || !el.getBoundingClientRect) return false; const s = getComputedStyle(el); return (s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) > 0) || (s.boxShadow && s.boxShadow !== 'none'); };
  const up = a.parentElement || (a.getRootNode() && a.getRootNode().host) || null;
  const cls = typeof a.className === 'string' && a.className.trim() ? '.' + a.className.trim().split(/\s+/)[0] : '';
  return {on_page: true, again, element: a.tagName.toLowerCase() + cls, visible: ring(a) || ring(up)};
})()"#;

/// Press Tab through the page from its top and say where the focus went: how many of the page's
/// visible controls it reached, which ones showed no focus, and why the walk stopped (`wrapped`
/// back to a control already reached, `left_page` to the browser, or the press `limit`).
fn keyboard_walk(client: &mut CdpClient, dl: &Deadline) -> Result<Value, ToolError> {
    let deep = |js: &str| js.replace("__DEEP__", journey::DEEP_JS);
    let start = eval(client, &deep(KEYBOARD_START_JS), dl)?;
    let (mut reached, mut not_visible, mut examples) = (0u64, 0u64, Vec::<String>::new());
    let mut stopped = "limit";
    for n in 0..KEYBOARD_TABS {
        if dl.expired() {
            stopped = "time";
            break;
        }
        for kind in ["keyDown", "keyUp"] {
            client.call(
                "Input.dispatchKeyEvent",
                json!({"type": kind, "key": "Tab", "code": "Tab", "windowsVirtualKeyCode": 9, "nativeVirtualKeyCode": 9}),
                None,
                dl,
            )?;
        }
        let f = eval(client, &KEYBOARD_FOCUS_JS.replace("__N__", &n.to_string()), dl)?;
        if f["on_page"] != true {
            // The first press can start outside the page; after that, leaving it ends the walk.
            if reached > 0 {
                stopped = "left_page";
                break;
            }
            continue;
        }
        if f["again"] == true {
            stopped = "wrapped";
            break;
        }
        reached += 1;
        if f["visible"] != true {
            not_visible += 1;
            if let Some(e) = f["element"].as_str()
                && examples.len() < 5
                && !examples.iter().any(|x| x == e)
            {
                examples.push(e.to_string());
            }
        }
    }
    let _ = eval(client, "(() => { let a = document.activeElement; if (a && a.blur) a.blur(); scrollTo(0, 0); })()", dl);
    Ok(
        json!({"controls": start["controls"], "reached": reached, "focus_not_visible": not_visible, "not_visible_examples": examples, "stopped": stopped, "max_tabs": KEYBOARD_TABS}),
    )
}

fn capture_into(ctx: &Ctx<'_>, client: &mut CdpClient, out: &Path, p: &CaptureParams, dl: &Deadline) -> Result<Captured, ToolError> {
    client.call("Page.enable", json!({}), None, dl)?;
    let mut shots = Vec::new();
    let mut blueprint = Value::Null;
    let mut layouts = Vec::new();
    for (i, &w) in p.widths.iter().enumerate() {
        ctx.report(Some(i as f32 / p.widths.len() as f32), format!("Photographing the page at {w} px…"));
        set_viewport(client, w, VIEW_HEIGHT, dl)?;
        settle(client, 900)?;
        // A page that can't be scrolled through is still photographed as it is.
        let _ = eval(client, REVEAL_JS, dl);
        if i == 0 {
            // The blueprint is read at the first width, where the layout is the reference one.
            blueprint = eval(client, BLUEPRINT_JS, dl)?;
        }
        // A page whose layout can't be read is still photographed.
        if let Ok(Value::Object(mut layout)) = eval(client, LAYOUT_JS, dl) {
            layout.insert("width".into(), json!(w));
            layouts.push(Value::Object(layout));
        }
        let height = eval(client, "Math.max(document.documentElement.scrollHeight, document.body ? document.body.scrollHeight : 0)", dl)?
            .as_u64()
            .unwrap_or(900)
            .clamp(320, u64::from(p.max_height));
        let clip = json!({"format": "png", "captureBeyondViewport": true, "clip": {"x": 0, "y": 0, "width": w, "height": height, "scale": 1}});
        let whole = |client: &mut CdpClient| -> Result<Vec<u8>, ToolError> {
            let shot = client.call("Page.captureScreenshot", clip.clone(), None, dl)?;
            let data =
                shot.get("data").and_then(Value::as_str).ok_or_else(|| ToolError::new(ErrorCode::GarbledOutput, "the browser returned no screenshot"))?;
            web_common::b64_decode(data)
        };
        let mut bytes = whole(client)?;
        // One tall picture of a page that draws only what is in view comes out blank where it reached
        // beyond the window (#8). When it looks like that, photograph it as it is scrolled instead,
        // and keep that picture if more of it is drawn.
        let mut in_bands = false;
        if height > u64::from(VIEW_HEIGHT) {
            let flat = flat_share(&bytes);
            if flat > FLAT_SHARE_SUSPECT
                && let Ok(stitched) = photograph_in_bands(client, w, height, dl)
                && flat_share(&stitched) < flat
            {
                bytes = stitched;
                in_bands = true;
            }
        }
        let photograph = |client: &mut CdpClient| if in_bands { photograph_in_bands(client, w, height, dl) } else { whole(client) };
        let (pw, ph) = web_common::png_dimensions(&bytes)?;
        let name = format!("screenshot-{w}.png");
        write_new_file(out, std::slice::from_ref(&name), &bytes).map_err(ToolError::invalid)?;
        let mut row = json!({"file": name, "width": pw, "height": ph, "css_width": w, "sha256": sha256_hex(&bytes), "cut": height >= u64::from(p.max_height)});
        if in_bands {
            row["in_bands"] = json!(true);
        }
        if p.noise_pass {
            // The same page twice more, at uneven gaps so something repeating every so often
            // can't look still in both: what changed by itself moves, not the build's fault.
            settle(client, 900)?;
            let second = photograph(client)?;
            settle(client, 1300)?;
            let third = photograph(client)?;
            if let (Ok(a), Ok(b), Ok(c)) = (web_common::decode_png(&bytes), web_common::decode_png(&second), web_common::decode_png(&third)) {
                let mask = web_common::NoiseMask::between(&a, &b, 24).union(&web_common::NoiseMask::between(&a, &c, 24));
                row["noise_tiles"] = json!(mask.count());
                if mask.count() > 0 {
                    let file = format!("noise-{w}.png");
                    write_new_file(out, std::slice::from_ref(&file), &mask.to_png()?).map_err(ToolError::invalid)?;
                    row["noise_file"] = json!(file);
                }
            }
        }
        shots.push(row);
    }
    if let Value::Object(b) = &mut blueprint {
        b.insert("layouts".into(), Value::Array(layouts));
    }
    let mut html_bytes = (0, 0);
    if p.include_html {
        let html = eval(client, "document.documentElement.outerHTML", dl)?;
        let html = html.as_str().unwrap_or_default();
        let kept = truncate_utf8(html, MAX_HTML_BYTES);
        html_bytes = (kept.len(), html.len());
        write_new_file(out, &["page.html".into()], kept.as_bytes()).map_err(ToolError::invalid)?;
    }
    Ok((blueprint, shots, html_bytes))
}

fn truncate_utf8(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s.get(..end).unwrap_or_default()
}

/// Drop everything a visitor can't see: the page's metadata, CSS variables and breakpoints, the
/// stylesheets, scripts, fonts and requests it loaded, where its pictures come from (their sizes
/// stay), its class names, and the query strings (deployment ids, tokens) on the addresses it
/// shows (#156). Screenshots, visible text, the measured layout and styles, links and forms are kept.
fn visitor_view(blueprint: &mut Value) {
    let Value::Object(b) = blueprint else { return };
    for k in ["meta", "breakpoints", "unreadable_stylesheets", "health"] {
        b.remove(k);
    }
    let bare = |v: &mut Value| {
        if let Some(s) = v.as_str() {
            *v = Value::String(s.split(['?', '#']).next().unwrap_or_default().to_string());
        }
    };
    if let Some(Value::Array(items)) = b.get_mut("outline") {
        for item in items.iter_mut().filter_map(Value::as_object_mut) {
            item.remove("class");
            item.get_mut("src").map(bare);
            item.get_mut("href").map(bare);
        }
    }
    if let Some(Value::Array(links)) = b.get_mut("links") {
        links.iter_mut().filter_map(|l| l.get_mut("href")).for_each(bare);
    }
    // "button.btn-primary" → "button": the element, not the page's class for it.
    if let Some(Value::Array(examples)) = b.get_mut("a11y").and_then(|a| a.get_mut("unnamed_examples")) {
        for e in examples.iter_mut() {
            if let Some(s) = e.as_str() {
                *e = Value::String(s.split('.').next().unwrap_or_default().to_string());
            }
        }
    }
    if let Some(Value::Array(examples)) = b.get_mut("a11y").and_then(|a| a.get_mut("keyboard")).and_then(|k| k.get_mut("not_visible_examples")) {
        let mut seen = Vec::new();
        examples.retain_mut(|e| {
            let tag = e.as_str().unwrap_or_default().split('.').next().unwrap_or_default().to_string();
            *e = Value::String(tag.clone());
            let new = !seen.contains(&tag);
            seen.push(tag);
            new
        });
    }
    if let Some(Value::Array(examples)) = b.get_mut("a11y").and_then(|a| a.get_mut("low_contrast_examples")) {
        for e in examples.iter_mut().filter_map(|e| e.get_mut("element")) {
            if let Some(s) = e.as_str() {
                *e = Value::String(s.split('.').next().unwrap_or_default().to_string());
            }
        }
    }
    if let Some(Value::Object(d)) = b.get_mut("design") {
        d.remove("css_variables");
        d.remove("font_faces");
    }
    let images: Vec<Value> = b
        .get("assets")
        .and_then(|a| a.get("images"))
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|i| json!({"w": i.get("w"), "h": i.get("h")})).collect())
        .unwrap_or_default();
    b.insert("assets".into(), json!({"images": images}));
}

/// A short human/agent-readable summary of the capture folder.
fn readme(b: &Value, shots: &[Value], (html_bytes, html_total): (usize, usize), visitor: bool) -> String {
    let s = |p: &str| b.pointer(p).and_then(Value::as_str).unwrap_or_default().to_string();
    let top = |p: &str, n: usize| {
        b.pointer(p)
            .and_then(Value::as_array)
            .map(|a| a.iter().take(n).filter_map(|x| x.get("value").and_then(Value::as_str)).map(|v| format!("`{v}`")).collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    };
    let mut md = format!("# Capture of {}\n\n", shown_url(&s("/url")));
    md.push_str(&format!("- Title: {}\n", s("/title")));
    if !s("/meta/description").is_empty() {
        md.push_str(&format!("- Description: {}\n", s("/meta/description")));
    }
    let shadow = |k: &str| b.pointer(&format!("/shadow/{k}")).and_then(Value::as_u64).unwrap_or(0);
    let (open, closed) = (shadow("open_roots"), shadow("closed_hosts"));
    if open + closed > 0 {
        let unread = if closed > 0 { format!("; {closed} seem to be closed and weren't read") } else { String::new() };
        md.push_str(&format!("- Web components: read inside {open} open shadow root(s){unread}\n"));
    }
    md.push_str(&format!(
        "- Page size at the first width: {}×{} CSS px\n\n",
        b.pointer("/page/w").and_then(Value::as_u64).unwrap_or(0),
        b.pointer("/page/h").and_then(Value::as_u64).unwrap_or(0)
    ));
    let yes = |k: &str| b.get(k) == Some(&Value::Bool(true));
    if yes("signed_in") && yes("sign_in_page") {
        md.push_str(&format!("{SIGN_IN_ENDED}\n\n"));
    } else if yes("signed_in") {
        md.push_str(&format!(
            "Captured with the person's saved sign-in for {}: if the site still accepted it, this is the site as they see it when signed in, and the pictures and text may show their own account's data.\n\n",
            s("/signed_in_site")
        ));
    } else if yes("sign_in_page") {
        md.push_str(&format!("{SIGN_IN_HINT}\n\n"));
    }
    md.push_str("## Files\n\n");
    for sh in shots {
        md.push_str(&format!(
            "- `{}` — {}×{} px screenshot{}\n",
            sh["file"].as_str().unwrap_or_default(),
            sh["width"],
            sh["height"],
            if sh["cut"].as_bool().unwrap_or(false) { " (cut at the height limit)" } else { "" }
        ));
        if let Some(noise) = sh["noise_file"].as_str() {
            md.push_str(&format!(
                "- `{noise}` — the parts that changed by themselves between two photos a moment apart (white, one pixel per 16 px square): carousels, videos, animations. They aren't compared.\n"
            ));
        }
    }
    if html_bytes > 0 {
        let cut = if html_bytes < html_total { format!(": the first part of {html_total}, cut at the size limit") } else { String::new() };
        md.push_str(&format!("- `page.html` — rendered HTML ({html_bytes} bytes{cut})\n"));
    }
    if visitor {
        md.push_str("- `capture.json` — design tokens, the layout at each width (`layouts`), layout outline (positions in CSS px), headings, links, forms, picture sizes and visible text\n\n");
        md.push_str("Clean room: only what a visitor sees was kept. The page's HTML, CSS rules, scripts, requests and errors weren't read.\n\n");
    } else {
        md.push_str("- `capture.json` — design tokens, breakpoints, the layout at each width (`layouts`), layout outline (positions in CSS px), headings, links, forms, assets and visible text\n\n");
    }
    md.push_str("## Design at a glance\n\n");
    md.push_str(&format!("- Fonts: {}\n", top("/design/fonts", 4)));
    md.push_str(&format!("- Text sizes: {}\n", top("/design/font_sizes", 8)));
    md.push_str(&format!("- Text colours: {}\n", top("/design/text_colors", 6)));
    md.push_str(&format!("- Backgrounds: {}\n", top("/design/backgrounds", 6)));
    md.push_str(&format!("- Corner radii: {}\n", top("/design/radii", 5)));
    md.push_str(&format!("- Spacing (padding, gaps, margins): {}\n", top("/design/spacing", 10)));
    md.push_str(&format!("- Content widths: {}\n", top("/design/max_widths", 4)));
    if visitor {
        md.push('\n');
    } else {
        md.push_str(&format!("- Font files: {}\n\n", b.pointer("/assets/fonts").and_then(Value::as_array).map_or(0, Vec::len)));
    }
    md.push_str(&layout_section(b, visitor));
    if !visitor {
        md.push_str(&health_section(b));
    }
    md.push_str("## Headings\n\n");
    for h in b.pointer("/headings").and_then(Value::as_array).into_iter().flatten().take(40) {
        let level = h.get("level").and_then(Value::as_u64).unwrap_or(1) as usize;
        md.push_str(&format!("{}- {}\n", "  ".repeat(level.saturating_sub(1)), h.get("text").and_then(Value::as_str).unwrap_or_default()));
    }
    md
}

/// What went wrong while the page loaded and how it reads to a screen reader, for the README.
fn health_section(b: &Value) -> String {
    let list = |p: &str| b.pointer(p).and_then(Value::as_array).cloned().unwrap_or_default();
    let (errors, failed) = (list("/health/errors"), list("/health/failed_requests"));
    let mut md = String::from("## Page health\n\n");
    if errors.is_empty() && failed.is_empty() {
        md.push_str("It loaded without script errors or failed requests.\n\n");
    } else {
        for e in errors.iter().take(10) {
            md.push_str(&format!("- {} error: {}\n", e["kind"].as_str().unwrap_or("script"), e["text"].as_str().unwrap_or_default()));
        }
        for f in failed.iter().take(10) {
            let why = f["status"].as_u64().map(|s| s.to_string()).or_else(|| f["error"].as_str().map(String::from)).unwrap_or_default();
            md.push_str(&format!("- failed request ({why}): {}\n", f["url"].as_str().unwrap_or_default()));
        }
        md.push('\n');
    }
    if let Some(a) = b.get("a11y") {
        let n = |k: &str| a.get(k).and_then(Value::as_u64).unwrap_or(0);
        let lang = a.get("lang").and_then(Value::as_str).filter(|l| !l.is_empty()).unwrap_or("not set");
        md.push_str(&format!(
            "## Accessibility\n\nLanguage: {lang}. Controls without a name: {}. Pictures without alt text: {}.\n\n",
            n("unnamed_controls"),
            n("images_without_alt")
        ));
        if let Some(k) = a.get("keyboard") {
            let kn = |key: &str| k.get(key).and_then(Value::as_u64).unwrap_or(0);
            md.push_str(&format!(
                "Keyboard: Tab reached {} of {} visible controls ({}); {} showed no focus.\n\n",
                kn("reached"),
                kn("controls"),
                k.get("stopped").and_then(Value::as_str).unwrap_or("?"),
                kn("focus_not_visible")
            ));
        }
        if a.get("text_checked").is_some() {
            md.push_str(&format!(
                "Text with too little contrast: {} of {} checked ({} over a picture, a gradient or a colour not read).\n\n",
                n("low_contrast"),
                n("text_checked"),
                n("contrast_unknown")
            ));
        }
    }
    md
}

/// The widths the page's CSS switches layout at, for the README.
fn breakpoints_section(b: &Value) -> String {
    let bps: Vec<String> = b
        .pointer("/breakpoints")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|q| q.get("query").and_then(Value::as_str))
        .map(String::from)
        .collect();
    let unreadable = b.get("unreadable_stylesheets").and_then(Value::as_u64).unwrap_or(0);
    let body = match (bps.is_empty(), unreadable) {
        (true, 0) => "Its CSS has no width breakpoints.\n\n".to_string(),
        (true, u) => format!("None readable: {u} stylesheet(s) from other sites can't be read; compare the screenshots at each width.\n\n"),
        (false, _) => format!("Its CSS switches layout at: {}\n\n", bps.join(", ")),
    };
    format!("## Breakpoints\n\n{body}")
}

/// Breakpoints (not in a visitor's view) and the layout at each width, for the README.
fn layout_section(b: &Value, visitor: bool) -> String {
    let n = |v: &Value, k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
    let mut md = String::new();
    if !visitor {
        md.push_str(&breakpoints_section(b));
    }
    md.push_str("## Layout at each width\n\n");
    for l in b.pointer("/layouts").and_then(Value::as_array).into_iter().flatten() {
        let (w, pw) = (n(l, "width"), n(l, "page_w"));
        let ty = |k: &str| l.pointer(&format!("/type/{k}")).and_then(Value::as_u64).unwrap_or(0);
        let scroll = if pw > w + 1 { format!("page {pw} px wide, so it scrolls sideways") } else { "no sideways scroll".into() };
        md.push_str(&format!("### {w} px ({scroll})\n\nHeadings {}/{}/{} px, text {} px.\n\n", ty("h1"), ty("h2"), ty("h3"), ty("body")));
        for blk in l.get("blocks").and_then(Value::as_array).into_iter().flatten().take(16) {
            md.push_str(&format!("- {}\n", block_line(blk)));
        }
        md.push('\n');
    }
    md
}

/// One section at one width: `section "Pricing": padding 96 32 96 32, gap 24, 3 per row, column 1280 px at x 80`.
fn block_line(blk: &Value) -> String {
    let n = |v: &Value, k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
    let mut s = blk.get("tag").and_then(Value::as_str).unwrap_or("block").to_string();
    if let Some(h) = blk.get("heading").and_then(Value::as_str) {
        s.push_str(&format!(" \"{h}\""));
    }
    let pad: Vec<String> = blk.get("pad").and_then(Value::as_array).into_iter().flatten().map(|p| p.as_u64().unwrap_or(0).to_string()).collect();
    s.push_str(&format!(": padding {}", pad.join(" ")));
    if n(blk, "gap") > 0 {
        s.push_str(&format!(", gap {}", n(blk, "gap")));
    }
    if n(blk, "per_row") > 1 {
        s.push_str(&format!(", {} per row", n(blk, "per_row")));
    }
    if let Some(c) = blk.get("column") {
        s.push_str(&format!(", column {} px at x {}", n(c, "max_w"), n(c, "x")));
    }
    if let Some(bg) = blk.get("background").and_then(Value::as_str) {
        s.push_str(&format!(", background {bg}"));
    }
    s
}

// ---------------------------------------------------------------------------------------------
// web_site_close

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloseParams {}

fn web_site_close(ctx: &mut Ctx<'_>, params: Value) -> Result<ToolOutput, ToolError> {
    let _: CloseParams = parse_params(params)?;
    let closed = ctx.session.ext.remove::<SiteBrowser>().is_some();
    Ok(ToolOutput::Value(json!({"closed": closed})))
}

#[cfg(test)]
mod tests {
    /// Reuse requires the same saved key and revision, and never an explicit fresh profile.
    #[test]
    fn profile_reuse_requires_current_revision_and_no_explicit_fresh_request() {
        let saved = super::site_sign_in::SavedSignIn {
            key: "site-example.com_443".into(),
            profile: std::path::PathBuf::from("profile"),
            saved_at: 1,
            revision: "second".into(),
        };
        for requested in [None, Some(true)] {
            assert!(super::reuse_profile(requested, Some(&saved.key), Some("second"), Some(&saved)));
            assert!(!super::reuse_profile(requested, Some(&saved.key), Some("first"), Some(&saved)));
            assert!(!super::reuse_profile(requested, Some("site-other.example_443"), Some("second"), Some(&saved)));
            assert!(!super::reuse_profile(requested, None, None, Some(&saved)));
            assert!(!super::reuse_profile(requested, Some(&saved.key), Some("second"), None));
        }
        assert!(super::reuse_profile(None, None, None, None));
        assert!(!super::reuse_profile(Some(false), None, None, None));
        assert!(!super::reuse_profile(Some(false), Some(&saved.key), Some("second"), Some(&saved)));
    }

    use super::*;

    #[test]
    fn a_click_is_described_by_what_changed() {
        let before = json!({"expanded": "false", "selected": null, "open": null, "dialogs": 0, "url": "https://a.test/"});
        let menu = json!({"expanded": "true", "selected": null, "open": null, "dialogs": 0, "url": "https://a.test/"});
        assert_eq!(state_change(&before, &menu), vec!["expanded"]);
        let dialog = json!({"expanded": "false", "selected": null, "open": null, "dialogs": 1, "url": "https://a.test/"});
        assert_eq!(state_change(&before, &dialog), vec!["dialog appeared"]);
        let away = json!({"expanded": "false", "selected": null, "open": null, "dialogs": 0, "url": "https://a.test/other"});
        assert_eq!(state_change(&before, &away), vec!["address changed"]);
        let tab = json!({"selected": "true", "url": "https://a.test/"});
        assert_eq!(state_change(&json!({"selected": "false", "url": "https://a.test/"}), &tab), vec!["tab selected"]);
        assert!(state_change(&before, &before).is_empty());
        // The page scripts carry their placeholders.
        assert!(FIND_STATES_JS.contains("__MAX__") && STATE_JS.contains("__I__") && CLICK_STATE_JS.contains("__I__"));
        assert!([FOCUS_SEARCH_JS, SEARCH_STATE_JS, CLEAR_SEARCH_JS].iter().all(|js| js.contains("__DEEP__")));
        assert!(REMEMBER_TAB_JS.contains("__I__") && RESELECT_TAB_JS.contains("__I__"));
    }

    #[test]
    fn urls_are_checked() {
        assert!(check_url("https://example.com").is_ok());
        assert!(check_url(" http://a.b/c?d ").is_ok());
        for bad in ["example.com", "file:///etc/passwd", "javascript:alert(1)", "https://", "https:///x", "https://a b.com", "chrome://settings"] {
            assert!(check_url(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn truncation_keeps_char_boundaries() {
        assert_eq!(truncate_utf8("héllo", 2), "h");
        assert_eq!(truncate_utf8("abc", 10), "abc");
    }

    #[test]
    fn readme_lists_files_and_tokens() {
        let b = json!({"url": "https://ex.com/a?secret=1", "title": "Ex", "page": {"w": 1440, "h": 3000},
            "design": {"fonts": [{"value": "Inter", "count": 9}], "font_sizes": [], "text_colors": [], "backgrounds": [], "radii": []},
            "headings": [{"level": 1, "text": "Hello"}, {"level": 2, "text": "World"}]});
        let md = readme(&b, &[json!({"file": "screenshot-1440.png", "width": 1440, "height": 3000, "cut": false})], (10, 10), false);
        assert!(md.contains("screenshot-1440.png") && md.contains("`Inter`") && md.contains("  - World") && md.contains("page.html"));
        assert!(!md.contains("secret=1"), "query strings are not echoed");
        assert!(md.contains("no width breakpoints"), "{md}");
        assert!(!md.contains("cut at the size limit") && html_cut((10, 10)).is_none());
    }

    /// #1178: a page captured signed out that asks for a password suggests signing in; a signed-in
    /// capture says it shows the person's own view, and never suggests it.
    #[test]
    fn a_sign_in_page_and_a_signed_in_capture_are_said() {
        let login = json!({"forms": [{"fields": [{"tag": "input", "type": "email"}, {"tag": "input", "type": "password"}]}]});
        assert!(has_password_field(&login));
        assert!(has_password_field(&json!({"outline": [{"tag": "input", "type": "password"}]})));
        assert!(!has_password_field(&json!({"forms": [{"fields": [{"type": "search"}]}], "outline": [{"tag": "a"}]})));
        assert!(!has_password_field(&json!({"forms": "garbage", "outline": 3})));
        let page = |extra: Value| {
            let mut b = json!({"url": "https://ex.com/", "title": "Ex", "page": {"w": 1440, "h": 900}});
            if let (Value::Object(b), Value::Object(e)) = (&mut b, extra) {
                b.extend(e);
            }
            readme(&b, &[], (0, 0), true)
        };
        assert!(page(json!({"signed_in": false, "sign_in_page": true})).contains(SIGN_IN_HINT));
        let signed = page(json!({"signed_in": true, "signed_in_site": "ex.com", "sign_in_page": false}));
        assert!(signed.contains("Captured with the person's saved sign-in") && signed.contains("ex.com") && !signed.contains(SIGN_IN_HINT), "{signed}");
        assert!(!page(json!({})).contains("signed in"));
        let ended = page(json!({"signed_in": true, "signed_in_site": "ex.com", "sign_in_page": true}));
        assert!(ended.contains(SIGN_IN_ENDED) && !ended.contains("Captured with the person's saved sign-in"), "{ended}");
        let (u, f) = signed_in_urls("https://ex.com/inbox?code=abc#t".into(), json!({"url": "https://ex.com/inbox?code=abc", "title": "x"}));
        assert_eq!((u.as_str(), f["url"].as_str()), ("https://ex.com/inbox", Some("https://ex.com/inbox")));
        assert!(profile_limitation(Some("ex.com")).message.contains("your own account"));
        assert_eq!(profile_limitation(None).code, "private_profile");
    }

    /// #336: HTML cut at the size limit is said in the evidence and the README.
    #[test]
    fn html_cut_at_the_limit_is_said() {
        let md = readme(&json!({}), &[], (MAX_HTML_BYTES, MAX_HTML_BYTES + 1174), false);
        assert!(md.contains(&format!("rendered HTML ({MAX_HTML_BYTES} bytes: the first part of {}, cut at the size limit)", MAX_HTML_BYTES + 1174)), "{md}");
        let l = html_cut((MAX_HTML_BYTES, MAX_HTML_BYTES + 1174)).unwrap();
        assert_eq!(l.code, "html_cut");
        assert!(l.message.contains("4 MiB"), "{}", l.message);
        assert!(html_cut((0, 0)).is_none());
    }

    #[test]
    fn components_with_closed_shadow_roots_are_said() {
        let b = json!({"shadow": {"open_roots": 4, "closed_hosts": 2}});
        let l = closed_shadow_roots(&b).unwrap();
        assert_eq!(l.code, "closed_shadow_roots");
        assert!(l.message.contains('2') && l.message.contains("screenshots"), "{}", l.message);
        let md = readme(&b, &[], (0, 0), false);
        assert!(md.contains("4 open shadow root(s)") && md.contains("2 seem to be closed"), "{md}");
        let open_only = json!({"shadow": {"open_roots": 4, "closed_hosts": 0}});
        assert!(closed_shadow_roots(&open_only).is_none() && closed_shadow_roots(&json!({})).is_none());
        assert!(!readme(&open_only, &[], (0, 0), false).contains("closed"));
        assert!(!readme(&json!({}), &[], (0, 0), false).contains("Web components"));
    }

    #[test]
    fn page_health_keeps_errors_and_failed_requests_without_query_strings() {
        let ev = |method: &str, params: Value| CdpEvent { method: method.into(), params, session_id: None };
        let events = vec![
            ev("Network.requestWillBeSent", json!({"requestId": "7", "request": {"url": "https://ex.com/font.woff2?token=abc"}})),
            ev(
                "Runtime.exceptionThrown",
                json!({"exceptionDetails": {"text": "Uncaught", "url": "https://ex.com/app.js?v=2", "exception": {"description": "TypeError: x is undefined\n    at app.js:1"}}}),
            ),
            ev(
                "Runtime.exceptionThrown",
                json!({"exceptionDetails": {"text": "Uncaught", "exception": {"description": "TypeError: x is undefined\n    at app.js:9"}}}),
            ),
            ev(
                "Runtime.consoleAPICalled",
                json!({"type": "error", "args": [{"type": "string", "value": "Hydration failed"}, {"type": "object", "description": "Error"}]}),
            ),
            ev("Runtime.consoleAPICalled", json!({"type": "log", "args": [{"type": "string", "value": "hello"}]})),
            ev("Log.entryAdded", json!({"entry": {"level": "error", "source": "network", "text": "Failed to load resource: 404"}})),
            ev("Log.entryAdded", json!({"entry": {"level": "error", "source": "security", "text": "Mixed content blocked", "url": "https://ex.com/"}})),
            ev("Network.responseReceived", json!({"type": "Image", "response": {"url": "https://ex.com/logo.png?sig=1", "status": 404}})),
            ev("Network.responseReceived", json!({"type": "Document", "response": {"url": "https://ex.com/", "status": 200}})),
            ev("Network.loadingFailed", json!({"requestId": "7", "type": "Font", "errorText": "net::ERR_NAME_NOT_RESOLVED"})),
            ev("Network.loadingFailed", json!({"requestId": "8", "type": "Media", "errorText": "net::ERR_ABORTED", "canceled": true})),
        ];
        let h = page_health(&events);
        let texts: Vec<&str> = h["errors"].as_array().unwrap().iter().map(|e| e["text"].as_str().unwrap()).collect();
        assert_eq!(texts, ["TypeError: x is undefined", "Hydration failed Error", "Mixed content blocked"], "one line each, once, no network log noise");
        assert_eq!(h["errors"][0]["url"], "https://ex.com/app.js");
        let failed = h["failed_requests"].as_array().unwrap();
        assert_eq!(failed.len(), 2, "a 404 and a failed font; the cancelled load and the 200 aren't failures");
        assert_eq!(failed[0], json!({"url": "https://ex.com/logo.png", "status": 404, "type": "Image"}));
        assert_eq!(failed[1]["url"], "https://ex.com/font.woff2");
        assert!(!h.to_string().contains("token") && !h.to_string().contains("sig="), "no query strings");
        assert_eq!(page_health(&[]), json!({"errors": [], "failed_requests": []}));
        let md = readme(&json!({"health": h, "a11y": {"lang": "en", "unnamed_controls": 2, "images_without_alt": 1}}), &[], (0, 0), false);
        assert!(md.contains("- script error: TypeError: x is undefined") && md.contains("- failed request (404): https://ex.com/logo.png"), "{md}");
        assert!(md.contains("Language: en. Controls without a name: 2. Pictures without alt text: 1."), "{md}");
        assert!(readme(&json!({"health": page_health(&[])}), &[], (0, 0), false).contains("loaded without script errors"));
    }

    #[test]
    fn dialogs_a_page_opened_are_kept_with_their_answer_and_said() {
        let ev = |params: Value| CdpEvent { method: cdp::DIALOG_OPENING.into(), params, session_id: None };
        let h = page_health(&[
            ev(json!({"type": "alert", "message": "Settings changed; resetting.\nline two", "url": "https://ex.com/?k=1"})),
            ev(json!({"type": "confirm", "message": "Delete everything?"})),
        ]);
        assert_eq!(
            h["dialogs"],
            json!([
                {"type": "alert", "message": "Settings changed; resetting.", "answered": "accepted"},
                {"type": "confirm", "message": "Delete everything?", "answered": "dismissed"},
            ]),
            "one line each, and a confirm is never agreed to"
        );
        assert!(page_health(&[]).get("dialogs").is_none(), "a page without dialogs keeps the old shape");
        let said = dialogs_said(&h["dialogs"]);
        assert_eq!(said, " The page opened an alert by itself (\"Settings changed; resetting.\") and 1 more; it was accepted so the page could go on.");
        assert_eq!(dialogs_said(&json!([])), "");
        assert_eq!(dialogs_said(&Value::Null), "");
    }

    #[test]
    fn readme_shows_breakpoints_and_the_layout_at_each_width() {
        let b = json!({"url": "https://ex.com/", "breakpoints": [{"query": "min-width 768px", "px": 768, "count": 3}],
        "design": {"spacing": [{"value": "96px", "count": 20}], "max_widths": [{"value": "1280px", "count": 4}]},
        "assets": {"fonts": ["https://ex.com/f.woff2"]},
        "layouts": [
            {"width": 1440, "page_w": 1440, "type": {"h1": 64, "h2": 40, "h3": 24, "body": 16},
             "blocks": [{"tag": "section", "heading": "Plans", "pad": [96, 32, 96, 32], "gap": 24, "per_row": 3, "column": {"max_w": 1280, "x": 80}, "background": "#12181f"}]},
            {"width": 390, "page_w": 512, "type": {"h1": 40}, "blocks": [{"tag": "header", "pad": [16, 16, 16, 16], "per_row": 1}]}
        ]});
        let md = readme(&b, &[], (0, 0), false);
        assert!(md.contains("switches layout at: min-width 768px") && md.contains("`96px`") && md.contains("`1280px`") && md.contains("Font files: 1"), "{md}");
        assert!(md.contains("### 1440 px (no sideways scroll)") && md.contains("Headings 64/40/24 px, text 16 px"), "{md}");
        assert!(md.contains("- section \"Plans\": padding 96 32 96 32, gap 24, 3 per row, column 1280 px at x 80, background #12181f"), "{md}");
        assert!(md.contains("### 390 px (page 512 px wide, so it scrolls sideways)") && md.contains("- header: padding 16 16 16 16\n"), "{md}");
        let cross = readme(&json!({"unreadable_stylesheets": 2}), &[], (0, 0), false);
        assert!(cross.contains("2 stylesheet(s) from other sites"), "{cross}");
    }

    #[test]
    fn a_visitors_view_keeps_what_shows_and_drops_the_pages_code() {
        let mut b = json!({
            "title": "Plans", "meta": {"description": "hidden"}, "breakpoints": [{"query": "min-width 768px"}], "unreadable_stylesheets": 1,
            "health": {"errors": [{"text": "boom"}]}, "design": {"fonts": [{"value": "Inter", "count": 9}], "css_variables": {"--brand": "#123"}, "font_faces": [{"family": "Inter"}]},
            "assets": {"images": [{"src": "https://cdn.test/a.png", "w": 40, "h": 30, "alt": "logo"}], "scripts": ["https://cdn.test/app.js"], "requests": [{"url": "https://api.test/x"}]},
            "headings": [{"level": 1, "text": "Plans"}], "text": "Plans Pricing", "layouts": [{"width": 1440, "page_w": 1440}],
            "outline": [{"tag": "img", "class": "w-full rounded-xl", "src": "https://cdn.test/hero.webp?dpl=dpl_abc123#x"}, {"tag": "a", "class": "btn", "href": "https://shop.test/plans?ref=sk-test-123"}],
            "links": [{"text": "Plans", "href": "https://shop.test/plans?session=abc"}],
            "a11y": {"unnamed_examples": ["button.icon-btn", "a"], "low_contrast_examples": [{"element": "span.muted-note", "ratio": 2.1}], "keyboard": {"not_visible_examples": ["button.bare", "button.icon", "a.nav"]}}
        });
        visitor_view(&mut b);
        // #156: the page's class names and the query strings on its addresses aren't a visitor's view.
        assert!(b["outline"].as_array().unwrap().iter().all(|i| i.get("class").is_none()));
        assert_eq!(b["outline"][0]["src"], "https://cdn.test/hero.webp");
        assert_eq!(b["outline"][1]["href"], "https://shop.test/plans");
        assert_eq!(b["links"][0]["href"], "https://shop.test/plans");
        assert_eq!(b["a11y"]["unnamed_examples"], json!(["button", "a"]));
        assert_eq!(b["a11y"]["low_contrast_examples"][0]["element"], "span");
        assert_eq!(b["a11y"]["keyboard"]["not_visible_examples"], json!(["button", "a"]), "each element once");
        assert!(!b.to_string().contains("dpl_") && !b.to_string().contains("sk-test"));
        for gone in [
            "/meta",
            "/breakpoints",
            "/unreadable_stylesheets",
            "/health",
            "/design/css_variables",
            "/design/font_faces",
            "/assets/scripts",
            "/assets/requests",
            "/assets/images/0/src",
        ] {
            assert!(b.pointer(gone).is_none(), "{gone} isn't something a visitor sees");
        }
        assert_eq!((b["assets"]["images"][0]["w"].as_u64(), b["text"].as_str()), (Some(40), Some("Plans Pricing")));
        let md = readme(&b, &[], (0, 0), true);
        assert!(
            md.contains("Clean room: only what a visitor sees") && !md.contains("Breakpoints") && !md.contains("Page health") && !md.contains("page.html"),
            "{md}"
        );
        assert!(md.contains("### 1440 px"), "{md}");
    }

    #[test]
    fn capture_without_open_site_is_a_clear_error() {
        let e = crate::Engine::new(crate::testing::session());
        let r = e.call(
            "web_site_capture",
            json!({"output_directory": std::env::temp_dir().join("ca-never-created").display().to_string()}),
            crate::CallOptions::new(crate::Caller::Cli),
        );
        let err = r.err().map(|e| e.message).unwrap_or_default();
        assert!(err.contains("no website is open") || err.contains("Chromium") || err.contains("chromium"), "{err}");
    }

    /// #153: from the CLI, a site opened by an earlier command is gone, and the error says why and
    /// where the steps run together. A stand-in browser path gets past the engine check on any
    /// machine; nothing is started.
    #[test]
    fn a_cli_capture_without_a_site_says_each_command_starts_fresh() {
        let fix = |caller| {
            let mut s = crate::testing::session();
            s.settings.engines.tools.chromium = Some(std::env::current_exe().unwrap());
            let dir = tempfile::tempdir().unwrap();
            let out = dir.path().join("capture");
            let err = crate::Engine::new(s)
                .call("web_site_capture", json!({"output_directory": out.display().to_string()}), crate::CallOptions::new(caller))
                .unwrap_err();
            assert!(err.message.contains("no website is open"), "{err:?}");
            assert!(!out.exists());
            err.remediation.unwrap_or_default()
        };
        assert!(fix(crate::Caller::Cli).contains("Each CLI command starts its own browser") && fix(crate::Caller::Cli).contains("craft-anything-cli mcp"));
        assert_eq!(fix(crate::Caller::Mcp), "Open one first with web_site_open.");
    }

    /// A browser that exits before becoming ready gets the exit diagnostic, not the deadline
    /// diagnostic. The stand-in exits without opening a browser or a window.
    #[test]
    fn a_browser_exit_before_readiness_is_reported_as_an_exit() {
        let stand_in = if cfg!(windows) {
            std::path::PathBuf::from(std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into())).join("System32").join("where.exe")
        } else {
            std::path::PathBuf::from("/bin/false")
        };
        if !stand_in.is_file() {
            return;
        }
        let mut s = crate::testing::session();
        s.settings.engines.tools.chromium = Some(stand_in);
        let e = crate::Engine::new(s);
        let opts = crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        // The error identifies which path won. A second wall-clock limit on the whole
        // call also times the test host scheduling, process startup and profile cleanup.
        let err = e.call("web_site_open", json!({"url": "https://example.com", "timeout_secs": 60}), opts).unwrap_err();
        assert_eq!(err.code, ErrorCode::ProgramFailed, "{err:?}");
        assert!(err.message.starts_with("Couldn't start the private browser") && err.message.contains("closed after"), "{}", err.message);
        assert!(err.remediation.is_some_and(|r| r.contains("Settings")));
    }

    /// #160: a section that only shows once it's scrolled into view is in the full-page picture.
    /// Live: it needs a Chromium-based browser, so it runs with `--ignored`.
    #[test]
    #[ignore]
    fn sections_shown_on_scroll_are_photographed() {
        use std::io::{Read, Write};
        let page = r#"<!doctype html><html><body style="margin:0">
<div style="height:1400px;background:#fff">Top</div>
<div id="late" style="height:600px;background:#fff">Shown on scroll</div>
<script>new IntersectionObserver(es => es.forEach(e => { if (e.isIntersecting) e.target.style.background = '#ff0000'; })).observe(document.getElementById('late'));</script>
</body></html>"#;
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for mut s in listener.incoming().flatten() {
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}", page.len());
            }
        });
        // In Clean room, a visitor's view: its evidence says so (#157).
        let mut s = crate::testing::live_session();
        s.settings.deep_mode = false;
        let e = crate::Engine::new(s);
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        e.call("web_site_open", json!({"url": url}), cli()).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("capture");
        let v = e.call("web_site_capture", json!({"output_directory": out.display().to_string(), "widths": [1440]}), cli()).unwrap().to_json();
        assert_eq!(v["environment"]["clean_room"], true, "{}", v["environment"]);
        let shot = web_common::decode_png(&std::fs::read(out.join("screenshot-1440.png")).unwrap()).unwrap();
        let at = |x: u32, y: u32| shot.rgba.get(((y * shot.width + x) * 4) as usize..((y * shot.width + x) * 4 + 3) as usize).unwrap().to_vec();
        assert_eq!(at(20, 1700), [255, 0, 0], "the section shown on scroll is in the picture");
        assert_eq!(at(20, 200), [255, 255, 255]);
    }

    /// #158: after a site fails to open, a capture refuses instead of photographing the browser's
    /// error page as if it were the site. Live: it needs a Chromium-based browser (`--ignored`).
    #[test]
    #[ignore]
    fn the_browsers_error_page_is_not_captured() {
        let e = crate::Engine::new(crate::testing::live_session());
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        // Nothing listens on port 1, so the browser shows its own error page.
        let _ = e.call("web_site_open", json!({"url": "http://127.0.0.1:1/"}), cli());
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("capture");
        let err = e.call("web_site_capture", json!({"output_directory": out.display().to_string(), "widths": [1440]}), cli()).unwrap_err();
        assert_eq!(err.code, ErrorCode::NotFound, "{}", err.message);
        assert!(err.message.contains("didn't open"), "{}", err.message);
        assert!(!out.exists(), "no folder is made for a capture that can't happen");
    }

    /// An engine with `page` open in a real browser, served from a local port. Live: it needs a
    /// Chromium-based browser, so it looks for one the way the app does.
    fn serve_and_open(page: &'static str) -> crate::Engine {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for mut s in listener.incoming().flatten() {
                let mut buf = [0u8; 4096];
                let _ = s.read(&mut buf);
                let _ = write!(s, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}", page.len());
            }
        });
        let e = crate::Engine::new(crate::testing::live_session());
        e.call("web_site_open", json!({"url": url}), crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted)).unwrap();
        e
    }

    /// #339: states captured at a phone width after a desktop width open the phone's own button,
    /// not the desktop one the earlier capture marked. Live (`--ignored`).
    #[test]
    #[ignore]
    fn states_at_a_second_width_open_that_widths_controls() {
        let e = serve_and_open(
            r#"<!doctype html><html><head><meta name="viewport" content="width=device-width"><style>
body{margin:0} .d{display:none} @media (min-width:800px){.d{display:block} .p{display:none}}
.panel{display:none;position:fixed;top:0;left:0;width:100%;height:200px}
</style></head><body style="padding-top:300px">
<button class="d" aria-expanded="false" data-panel="dp">Desktop options</button>
<button class="p" aria-expanded="false" data-panel="pp">Phone options</button>
<div id="dp" class="panel" style="background:#0000ff"></div><div id="pp" class="panel" style="background:#ff0000"></div>
<script>for (const b of document.querySelectorAll('button')) b.onclick = () => {
  const open = b.getAttribute('aria-expanded') !== 'true';
  b.setAttribute('aria-expanded', String(open));
  document.getElementById(b.dataset.panel).style.display = open ? 'block' : 'none';
};</script></body></html>"#,
        );
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let dir = tempfile::tempdir().unwrap();
        let top_colour = |out: &Path| {
            let rows: Value = serde_json::from_str(&std::fs::read_to_string(out.join("states.json")).unwrap()).unwrap();
            let shot = web_common::decode_png(&std::fs::read(out.join("state-1.png")).unwrap()).unwrap();
            let at = shot.rgba.get(((10 * shot.width + 10) * 4) as usize..((10 * shot.width + 10) * 4 + 3) as usize).unwrap().to_vec();
            (rows["states"][0]["label"].as_str().unwrap().to_string(), at)
        };
        for (width, label, colour) in [(1440, "Desktop options", [0, 0, 255]), (390, "Phone options", [255, 0, 0])] {
            let out = dir.path().join(format!("states-{width}"));
            e.call("web_page_states_capture", json!({"output_directory": out.display().to_string(), "width": width, "max_states": 1}), cli()).unwrap();
            assert_eq!(top_colour(&out), (label.to_string(), colour.to_vec()), "at {width} px");
        }
        let _ = e.call("web_site_close", json!({}), cli());
    }

    /// The search field is typed into, never submitted: by default with the word the page's
    /// headings and links use most, in an open shadow root too, and what showed is pictured. An
    /// empty search_text types nothing. Live (`--ignored`).
    #[test]
    #[ignore]
    fn the_search_field_is_typed_into_and_its_suggestions_pictured() {
        let e = serve_and_open(
            r#"<!doctype html><html lang="en"><body><h1>Planets</h1><a href="/p/mars">Planets near us</a><a href="/p/venus">Other planets</a>
<site-search></site-search><ul id="out"></ul>
<script>customElements.define('site-search', class extends HTMLElement { connectedCallback() {
  const root = this.attachShadow({mode: 'open'});
  root.innerHTML = '<form action="/find"><input type="search" aria-label="Search the atlas"><ul role="listbox" id="list"></ul></form>';
  const input = root.querySelector('input'), list = root.querySelector('#list');
  input.addEventListener('input', () => { list.innerHTML = input.value ? '<li role="option">' + input.value + ' one</li><li role="option">' + input.value + ' two</li>' : ''; });
}});</script></body></html>"#,
        );
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let dir = tempfile::tempdir().unwrap();
        let rows = |out: &Path| -> Vec<Value> {
            let v: Value = serde_json::from_str(&std::fs::read_to_string(out.join("states.json")).unwrap()).unwrap();
            v["states"].as_array().unwrap().clone()
        };
        let out = dir.path().join("states");
        e.call("web_page_states_capture", json!({"output_directory": out.display().to_string(), "max_states": 1}), cli()).unwrap();
        let search = rows(&out).into_iter().find(|r| r["kind"] == "search").expect("a search row");
        assert_eq!(search["typed"], "planets", "{search}");
        assert_eq!(search["label"], "Search the atlas");
        assert!(search["changed"].as_array().unwrap().iter().any(|c| c == "suggestions appeared"), "{search}");
        assert!(out.join(search["file"].as_str().unwrap()).is_file());

        let none = dir.path().join("none");
        e.call("web_page_states_capture", json!({"output_directory": none.display().to_string(), "max_states": 1, "search_text": ""}), cli()).unwrap();
        assert!(rows(&none).iter().all(|r| r["kind"] != "search"), "an empty search_text types nothing");
        let _ = e.call("web_site_close", json!({}), cli());
    }

    /// A journey checks states, not only texts: a control disabled until a field is filled, a
    /// control that isn't there, the value a field holds (compared in the page) and an error the
    /// page announces. A state that never comes fails its own step. Live (`--ignored`).
    #[test]
    #[ignore]
    fn a_journey_checks_control_states_field_values_and_announced_errors() {
        let e = serve_and_open(
            r#"<!doctype html><html><body>
<label>Name <input id="n"></label>
<button id="s" disabled>Submit claim</button>
<button id="v">Validate</button>
<p id="e" role="alert"></p>
<script>
const n = document.getElementById('n'), s = document.getElementById('s');
n.oninput = () => { s.disabled = !n.value.trim(); };
document.getElementById('v').onclick = () => { document.getElementById('e').textContent = n.value.trim() ? '' : 'Enter your name.'; };
</script></body></html>"#,
        );
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let run = |steps: Value| e.call("web_site_journey_run", json!({"steps": steps, "wait_ms": 600}), cli()).unwrap().to_json()["result"].clone();

        let good = run(json!([
            {"check": "Submit claim", "is": "disabled"}, {"check": "Approve", "is": "absent"}, {"check": "Approve", "is": "unusable"},
            {"click": "Validate"}, {"alert": "Enter your name."}, {"alert": ""},
            {"type": "Ada", "into": "Name"}, {"check": "Submit claim", "is": "enabled"}, {"value_of": "Name", "equals": "ada"}
        ]));
        assert_eq!(good["passed"], true, "{good}");
        for (steps, why) in [
            (json!([{"check": "Submit claim", "is": "enabled"}]), "disabled until the name is typed"),
            (json!([{"check": "Approve", "is": "present"}]), "there is no Approve"),
            (json!([{"type": "Ada", "into": "Name"}, {"value_of": "Name", "equals": "Bob"}]), "the field holds Ada"),
            (json!([{"alert": "Enter your name."}]), "nothing was announced yet"),
        ] {
            let failed = run(steps);
            assert_eq!(failed["passed"], false, "{why}: {failed}");
        }
        let _ = e.call("web_site_close", json!({}), cli());
    }

    /// A journey is done the way a visitor would do it: fields are found by their label, controls
    /// by their name (inside an open shadow root too), Enter submits, and each run starts from a
    /// page loaded again. A covered control, or a text that never shows, fails its own step. Live
    /// (`--ignored`).
    #[test]
    #[ignore]
    fn a_journey_types_clicks_and_checks_what_shows_and_names_the_step_that_failed() {
        let e = serve_and_open(
            r#"<!doctype html><html><body>
<form id="f"><label for="q">Search the library</label> <input id="q"> <button>Search</button></form>
<label>Kind <select id="k"><option>All</option><option>Texts</option></select></label>
<p id="out"></p>
<x-panel></x-panel>
<div style="position:relative;width:200px;height:40px"><button id="c">Covered action</button>
<div style="position:absolute;inset:0;background:#fff"></div></div>
<script>
const out = document.getElementById('out');
document.getElementById('f').onsubmit = ev => { ev.preventDefault(); out.textContent = 'Results for ' + document.getElementById('q').value; };
document.getElementById('k').onchange = ev => { out.textContent = 'Kind chosen: ' + ev.target.value; };
document.getElementById('c').onclick = () => { out.textContent = 'Covered action ran'; };
customElements.define('x-panel', class extends HTMLElement { connectedCallback() {
  const root = this.attachShadow({mode: 'open'});
  root.innerHTML = '<button>Open panel</button><p></p>';
  root.querySelector('button').onclick = () => { root.querySelector('p').textContent = 'Panel is open'; };
} });
if (location.hash === '#/library') out.textContent = 'Library page';
</script></body></html>"#,
        );
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let run = |args: Value| e.call("web_site_journey_run", args, cli()).unwrap().to_json()["result"].clone();

        let typed = run(json!({"steps": [{"type": "cats", "into": "Search the library"}, {"press": "Enter"}, {"expect": "results for cats"}]}));
        assert_eq!(typed["passed"], true, "{typed}");
        let clicked = run(json!({"steps": [{"type": "maps", "into": "search the library"}, {"click": "Search"}, {"expect": "Results for maps"}]}));
        assert_eq!(clicked["passed"], true, "{clicked}");

        let fresh = run(json!({"steps": [{"expect": "Results for maps"}], "wait_ms": 300}));
        assert_eq!((&fresh["passed"], &fresh["failed_at"]), (&json!(false), &json!(1)), "the page is loaded again before a journey: {fresh}");
        let kept = run(json!({"steps": [{"click": "Search"}, {"expect": "Results for"}], "restart": true}));
        assert_eq!(kept["passed"], true, "{kept}");
        let carried_on = run(json!({"steps": [{"expect": "Results for"}], "restart": false}));
        assert_eq!(carried_on["passed"], true, "without a restart the page is as the last journey left it: {carried_on}");

        let in_shadow = run(json!({"steps": [{"click": "Open panel"}, {"expect": "Panel is open"}]}));
        assert_eq!(in_shadow["passed"], true, "{in_shadow}");
        let chosen = run(json!({"steps": [{"type": "Texts", "into": "Kind"}, {"expect": "Kind chosen: Texts"}]}));
        assert_eq!(chosen["passed"], true, "{chosen}");
        let opened = run(json!({"steps": [{"open": "/#/library"}, {"expect": "Library page"}]}));
        assert_eq!(opened["passed"], true, "{opened}");

        let covered = run(json!({"steps": [{"click": "Covered action"}, {"expect": "Covered action ran"}]}));
        assert_eq!((&covered["passed"], &covered["failed_at"]), (&json!(false), &json!(1)), "{covered}");
        assert!(covered["steps"][0]["note"].as_str().unwrap().contains("covered"), "{covered}");
        assert_eq!(covered["steps"].as_array().unwrap().len(), 1, "the steps after a failed one are not run");
        let missing = run(json!({"steps": [{"click": "Search"}, {"expect": "A text that never shows"}, {"click": "Search"}], "wait_ms": 300}));
        assert_eq!((&missing["passed"], &missing["failed_at"], &missing["steps"][0]["ok"]), (&json!(false), &json!(2), &json!(true)), "{missing}");
        let no_such = run(json!({"steps": [{"click": "A control that is not there"}]}));
        assert_eq!(no_such["passed"], false, "{no_such}");

        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("journey");
        let pictured = run(json!({"steps": [{"click": "Open panel"}, {"expect": "Panel is open"}], "output_directory": out.display().to_string()}));
        assert!(out.join("end.png").is_file() && pictured["picture"].is_string(), "{pictured}");
        let _ = e.call("web_site_close", json!({}), cli());
    }

    /// #335: after photographing another tab, the tab that was selected is selected again, so the
    /// next capture finds the page as it was. Live (`--ignored`).
    #[test]
    #[ignore]
    fn the_tab_that_was_selected_is_selected_again() {
        let e = serve_and_open(
            r#"<!doctype html><html><body><div role="tablist">
<button role="tab" aria-selected="true">One</button><button role="tab" aria-selected="false">Two</button></div>
<script>const tabs = document.querySelectorAll('[role="tab"]');
for (const t of tabs) t.onclick = () => tabs.forEach(o => o.setAttribute('aria-selected', String(o === t)));</script></body></html>"#,
        );
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let dir = tempfile::tempdir().unwrap();
        // Each capture opens the unselected tab: Two both times when One is selected again.
        for n in 1..=2 {
            let out = dir.path().join(format!("states-{n}"));
            e.call("web_page_states_capture", json!({"output_directory": out.display().to_string(), "max_states": 1}), cli()).unwrap();
            let rows: Value = serde_json::from_str(&std::fs::read_to_string(out.join("states.json")).unwrap()).unwrap();
            assert_eq!(rows["states"][0]["label"], "Two", "capture {n}: {rows}");
            assert_eq!(rows["states"][0]["changed"], json!(["tab selected"]), "capture {n}: {rows}");
        }
        let _ = e.call("web_site_close", json!({}), cli());
    }

    /// `capture.json` of `page`, served from a local port and captured at 1440 px. Live: it needs a
    /// Chromium-based browser.
    fn capture_of(page: &'static str) -> Value {
        let e = serve_and_open(page);
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("capture");
        e.call("web_site_capture", json!({"output_directory": out.display().to_string(), "widths": [1440]}), cli()).unwrap();
        serde_json::from_str(&std::fs::read_to_string(out.join("capture.json")).unwrap()).unwrap()
    }

    /// The Tab walk reaches each visible control once, in open shadow roots too, skips what is
    /// taken out of the order, and names a control whose focus can't be seen. Live (`--ignored`).
    #[test]
    #[ignore]
    fn the_keyboard_walk_counts_reached_controls_and_invisible_focus() {
        const PAGE: &str = r#"<!doctype html><html lang="en"><body><main>
<a href="/one">One</a>
<button>Two</button>
<button class="bare" style="outline:none">Three</button>
<button tabindex="-1">Skipped</button>
<div onclick="void 0">Not a control</div>
<tab-host></tab-host>
<script>customElements.define('tab-host', class extends HTMLElement { connectedCallback() {
  this.attachShadow({mode: 'open'}).innerHTML = '<button>Four</button>';
}});</script>
</main></body></html>"#;
        let k = capture_of(PAGE)["a11y"]["keyboard"].clone();
        assert_eq!(k["controls"], 4, "{k}");
        assert_eq!(k["reached"], 4, "each control once: {k}");
        assert_eq!(k["focus_not_visible"], 1, "{k}");
        assert!(k["not_visible_examples"][0].as_str().unwrap().starts_with("button"), "{k}");
        assert_ne!(k["stopped"], "limit", "{k}");
    }

    /// Text is judged against the colour behind it, through its ancestors: pale grey on white is too
    /// faint, the same grey on a dark band is fine, large text needs less, and text over a gradient
    /// isn't judged. Live (`--ignored`).
    #[test]
    #[ignore]
    fn text_contrast_is_measured_against_the_colour_behind_it() {
        const PAGE: &str = r#"<!doctype html><html lang="en"><body style="background:#fff;color:#111"><main>
<p>Plain dark text</p>
<p class="faint" style="color:#aaa">Faint grey note</p>
<div style="background:#202020"><div><p style="color:#aaa">Grey on a dark band</p></div></div>
<h2 style="color:#888;font-size:28px">Large grey heading</h2>
<div style="background:linear-gradient(#000,#fff)"><p style="color:#777">Over a gradient</p></div>
</main></body></html>"#;
        let a = capture_of(PAGE)["a11y"].clone();
        assert_eq!(a["low_contrast"], 1, "only the faint note on white: {a}");
        let worst = &a["low_contrast_examples"][0];
        assert!(worst["element"].as_str().unwrap().starts_with('p') && worst["ratio"].as_f64().unwrap() < worst["needs"].as_f64().unwrap(), "{a}");
        assert_eq!(worst["count"], 1, "{a}");
        assert!(a["text_checked"].as_u64().unwrap() >= 4, "{a}");
        assert!(a["contrast_unknown"].as_u64().unwrap() >= 1, "the text over the gradient: {a}");
    }

    /// What is inside a closed `<details>` keeps a box but isn't seen: it is neither outlined nor
    /// counted as a control without a name, and a journey can't click it. Opened, it is all three.
    /// Live (`--ignored`).
    #[test]
    #[ignore]
    fn what_a_closed_details_holds_is_not_shown_until_it_is_opened() {
        const CLOSED: &str = r#"<!doctype html><html lang="en"><body><main><h1>Menus</h1>
<details><summary>Web</summary><ul><li><a href="/saved">Saved pages</a></li></ul></details></main></body></html>"#;
        const OPEN: &str = r#"<!doctype html><html lang="en"><body><main><h1>Menus</h1>
<details open><summary>Web</summary><ul><li><a href="/saved">Saved pages</a></li></ul></details></main></body></html>"#;
        let outlined = |c: &Value| c["outline"].as_array().unwrap().iter().any(|e| e["text"] == "Saved pages");
        let (closed, open) = (capture_of(CLOSED), capture_of(OPEN));
        assert!(!outlined(&closed) && outlined(&open), "closed: {}", closed["outline"]);
        assert_eq!(closed["a11y"]["unnamed_controls"], 0, "a link with text is not a control without a name: {}", closed["a11y"]);
        assert_eq!(open["a11y"]["unnamed_controls"], 0, "{}", open["a11y"]);

        let e = serve_and_open(CLOSED);
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let run = |steps: Value| e.call("web_site_journey_run", json!({"steps": steps}), cli()).unwrap().to_json()["result"].clone();
        let hidden = run(json!([{"click": "Saved pages"}]));
        assert!(hidden["passed"] == false && hidden["steps"][0]["note"].as_str().unwrap().contains("no control named"), "{hidden}");
        let opened = run(json!([{"click": "Web"}, {"click": "Saved pages"}]));
        assert_eq!(opened["steps"][1]["ok"], true, "{opened}");
        let _ = e.call("web_site_close", json!({}), cli());
    }

    /// #155: a pill made with `calc(infinity * 1px)` (Tailwind's `rounded-full`) is recorded as
    /// `9999px`, not the browser's clamped `3.35544e+07px`. Live (`--ignored`).
    #[test]
    #[ignore]
    fn an_endless_corner_radius_is_recorded_as_a_pill() {
        let page = r#"<!doctype html><html><body>
<a style="border-radius:calc(infinity * 1px);background:#1e66f5;padding:8px">One</a>
<a style="border-radius:calc(infinity * 1px) calc(infinity * 1px) 4px 4px;background:#1e66f5;padding:8px">Two</a>
<a style="border-radius:12px;background:#1e66f5;padding:8px">Three</a>
</body></html>"#;
        let capture = capture_of(page);
        let mut radii: Vec<&str> = capture["design"]["radii"].as_array().unwrap().iter().filter_map(|r| r["value"].as_str()).collect();
        radii.sort_unstable();
        assert_eq!(radii, ["12px", "9999px", "9999px 9999px 4px 4px"]);
    }

    /// #154: colours are the page's own CSS values, not a canvas read that can be off by one per
    /// channel; see-through ones keep their alpha. Live (`--ignored`).
    #[test]
    #[ignore]
    fn captured_colours_are_the_css_values() {
        let capture = capture_of(
            r#"<!doctype html><html><body style="background:#F5F2EC;color:#2D2B26"><p>Text</p><p style="background:rgba(30, 102, 245, 0.5)">Half</p></body></html>"#,
        );
        let values = |k: &str| capture["design"][k].as_array().unwrap().iter().filter_map(|c| c["value"].as_str().map(String::from)).collect::<Vec<_>>();
        assert!(values("backgrounds").contains(&"#f5f2ec".to_string()), "{:?}", values("backgrounds"));
        assert!(values("backgrounds").contains(&"#1e66f580".to_string()), "{:?}", values("backgrounds"));
        assert_eq!(values("text_colors"), ["#2d2b26"]);
    }

    /// #8: a list that draws only the rows in view is blank beyond the window in one tall picture.
    /// It is photographed as it is scrolled, and the bar fixed to the window is in the picture once.
    /// Live (`--ignored`).
    #[test]
    #[ignore]
    fn a_page_that_draws_only_what_is_in_view_is_photographed_as_it_is_scrolled() {
        let e = serve_and_open(
            r#"<!doctype html><html><body style="margin:0"><div id="list" style="position:relative;height:5400px"></div>
<div style="position:fixed;left:0;right:0;bottom:0;height:30px;background:#ff00ff"></div>
<script>
const list = document.getElementById('list');
const draw = () => {
  list.innerHTML = '';
  const first = Math.max(0, Math.floor(scrollY / 100) - 1), last = Math.min(53, Math.ceil((scrollY + innerHeight) / 100));
  for (let i = first; i <= last; i++) {
    const r = document.createElement('div');
    r.style.cssText = 'position:absolute;left:0;right:0;height:100px;top:' + i * 100 + 'px;background:repeating-linear-gradient(90deg,#00aa00 0 20px,#0000ff 20px 40px)';
    list.appendChild(r);
  }
};
addEventListener('scroll', draw); draw();
</script></body></html>"#,
        );
        let cli = || crate::CallOptions::new(crate::Caller::Cli).with_consent(crate::Consent::Granted);
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("capture");
        let v = e.call("web_site_capture", json!({"output_directory": out.display().to_string(), "widths": [1440]}), cli()).unwrap().to_json();
        assert_eq!(v["result"]["screenshots"][0]["in_bands"], true, "{}", v["result"]["screenshots"]);
        assert!(v["limitations"].as_array().unwrap().iter().any(|l| l["code"] == "photographed_in_bands"), "{}", v["limitations"]);
        let shot = web_common::decode_png(&std::fs::read(out.join("screenshot-1440.png")).unwrap()).unwrap();
        let at = |x: u32, y: u32| shot.rgba.get(((y * shot.width + x) * 4) as usize..((y * shot.width + x) * 4 + 3) as usize).unwrap().to_vec();
        for y in [450, 2750, 5250] {
            assert_ne!(at(10, y), [255, 255, 255], "the rows at {y} px are in the picture");
        }
        let bar_rows = (0..shot.height).filter(|y| at(10, *y) == [255, 0, 255]).count();
        assert!((1..=30).contains(&bar_rows), "the fixed bar is in the picture once, not in every band: {bar_rows} rows");
        // The page is left as it was: at its top, with the bar showing.
        let after = e.call("web_page_show", json!({}), cli());
        assert!(after.is_ok(), "the page still answers after the capture");
    }

    /// Components (Lit and the like) keep their content in shadow roots: open ones are read, and a
    /// closed one is counted. Live (`--ignored`).
    #[test]
    #[ignore]
    fn open_shadow_roots_are_read_and_closed_ones_counted() {
        let capture = capture_of(
            r#"<!doctype html><html><head><style>x-locked{display:block}</style></head><body><h1>Light title</h1><x-card></x-card><x-locked></x-locked>
<script>
customElements.define('x-card', class extends HTMLElement { constructor() { super();
  this.attachShadow({mode: 'open'}).innerHTML = '<h2>Shadow title</h2><a href="/inside">Inside link</a><button>Shadow button</button>'; } });
customElements.define('x-locked', class extends HTMLElement { constructor() { super();
  this.attachShadow({mode: 'closed'}).innerHTML = '<p style="margin:0;height:40px">Locked</p>'; } });
</script></body></html>"#,
        );
        let outline = capture["outline"].as_array().unwrap();
        let item = |text: &str| outline.iter().find(|i| i["text"] == text).unwrap_or_else(|| panic!("{text} not in {outline:?}"));
        assert_eq!(item("Shadow button")["tag"], "button");
        assert!(item("Shadow title")["depth"].as_u64() > item("Light title")["depth"].as_u64(), "depth counts through the host");
        let hrefs: Vec<&str> = capture["links"].as_array().unwrap().iter().filter_map(|l| l["href"].as_str()).collect();
        assert!(hrefs.iter().any(|h| h.ends_with("/inside")), "{hrefs:?}");
        let headings: Vec<&str> = capture["headings"].as_array().unwrap().iter().filter_map(|h| h["text"].as_str()).collect();
        assert!(headings.contains(&"Light title") && headings.contains(&"Shadow title"), "{headings:?}");
        let text = capture["text"].as_str().unwrap_or_default();
        assert!(text.contains("Light title") && text.contains("Shadow button"), "the visible text reads through open roots: {text:?}");
        assert!(!text.contains("Locked"), "and not into a closed one: {text:?}");
        assert!(capture["shadow"]["open_roots"].as_u64() >= Some(1) && capture["shadow"]["closed_hosts"].as_u64() >= Some(1), "{}", capture["shadow"]);
    }
}
