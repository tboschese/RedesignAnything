//! How alike the prototype and the original are, page by page, measured with what the website
//! kind already uses on a rebuild: `structure_likeness` and `text_likeness` over the two
//! `capture.json` files, and the tile and SSIM scores of `web_common` over the screenshots.
//!
//! The original's pages are the start page (`capture/`) and each captured route
//! (`capture/routes/<slug>/`). The prototype page that stands for each one is captured into
//! `redesign/prototype_capture/start/` or `redesign/prototype_capture/<slug>/`, by
//! `redesign_prototype_capture` or by the agent with `web_site_open` + `web_site_capture`. Nothing in
//! this module opens a browser.
//!
//! A page that can't be compared is named with its reason and never counts as alike or unlike. That
//! includes a screenshot with large blank bands: a page that draws only what is in view comes out
//! blank where the full-page picture reached beyond the window.

use craft_anything_engine::tools::web_common::{self, Image};
use craft_anything_engine::workspace::Workspace;
use craft_anything_evidence::ToolError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

use super::kinds::website;
use super::redesign_checks::CheckResult;
use super::redesign_store::{self, redesign_dir, reject_symlink, safe_child, write_record};

pub const PROTOTYPE_CAPTURE_DIR: &str = "prototype_capture";
pub const LIKENESS_FILE: &str = "likeness.json";
/// The folder name of the start page's prototype capture.
pub const START_PAGE: &str = "start";
const SCHEMA_VERSION: &str = "1.0.0";
/// A screenshot with more than this share of its height in blank bands isn't compared.
const MAX_BLANK_SHARE: f64 = 0.35;
const BAND_PX: u32 = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLikeness {
    /// `start` or the route's slug.
    pub page: String,
    /// 0..=100: the page parts both have in the same order (`website::structure_likeness`).
    pub structure: Option<f64>,
    /// 0..=100: word overlap. Simulated data lowers it, so it is reported, not scored.
    pub text: Option<f64>,
    /// 0..=100: tile layout and SSIM detail of the screenshots over their shared area, at `width`.
    pub picture: Option<f64>,
    pub width: Option<u32>,
    /// Mean of the measures that exist, without `text`. `None`: nothing could be compared.
    pub overall: Option<f64>,
    /// What wasn't measured, and why.
    pub not_measured: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LikenessReport {
    pub schema_version: String,
    pub pages: Vec<PageLikeness>,
}

impl LikenessReport {
    /// Pages with an overall score.
    pub fn measured(&self) -> impl Iterator<Item = (&str, f64)> {
        self.pages.iter().filter_map(|p| Some((p.page.as_str(), p.overall?)))
    }

    /// `page: reason` for every page with no overall score.
    pub fn unmeasured(&self) -> Vec<String> {
        self.pages.iter().filter(|p| p.overall.is_none()).map(|p| format!("{}: {}", p.page, p.not_measured.join("; "))).collect()
    }
}

/// For each band of the picture, top to bottom: whether it is one flat colour.
fn flat_bands(img: &Image) -> Vec<bool> {
    let (w, h) = (img.width as usize, img.height as usize);
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let band = BAND_PX as usize;
    (0..h)
        .step_by(band)
        .map(|top| {
            let rows = &img.rgba[top * w * 4..(top + band).min(h) * w * 4];
            let first = &rows[..4];
            // Every 8th pixel is enough to tell a drawn band from a flat one.
            rows.as_chunks::<4>().0.iter().step_by(8).all(|px| px.iter().zip(first).all(|(a, b)| a.abs_diff(*b) <= 3))
        })
        .collect()
}

/// The share of the picture's height made of bands with one flat colour.
pub fn blank_share(img: &Image) -> f64 {
    let bands = flat_bands(img);
    if bands.is_empty() {
        return 1.0;
    }
    bands.iter().filter(|flat| **flat).count() as f64 / bands.len() as f64
}

/// How far down the picture something is drawn: the bottom of its last band that is not flat.
fn drawn_height(img: &Image) -> u32 {
    let last = flat_bands(img).iter().rposition(|flat| !flat);
    last.map_or(0, |i| ((i as u32 + 1) * BAND_PX).min(img.height))
}

fn read_capture(dir: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(dir.join("capture.json")).ok()?).ok()
}

fn widths(dir: &Path) -> Vec<u32> {
    let mut out: Vec<u32> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().to_str()?.strip_prefix("screenshot-")?.strip_suffix(".png")?.parse().ok())
        .collect();
    out.sort_unstable();
    out
}

/// The window's height when a page was captured.
fn window_height(dir: &Path) -> u32 {
    read_capture(dir).and_then(|c| c["viewport"]["h"].as_u64()).map_or(900, |h| h as u32)
}

/// A page's picture at `width`, ready to compare. A picture no taller than the window was all in
/// view when it was taken, so the empty space under a short page is cut off instead of counted. A
/// taller one with large flat bands was not drawn to the end, and is refused.
fn picture(dir: &Path, width: u32, whose: &str) -> Result<Image, String> {
    let bytes = fs::read(dir.join(format!("screenshot-{width}.png"))).map_err(|e| format!("{whose} screenshot can't be read: {e}"))?;
    let img = web_common::decode_png(&bytes).map_err(|e| format!("{whose} screenshot can't be decoded: {}", e.message))?;
    if img.height <= window_height(dir) {
        return match drawn_height(&img) {
            0 => Err(format!("{whose} screenshot shows nothing drawn")),
            drawn => Ok(web_common::crop(&img, img.width, drawn)),
        };
    }
    let blank = blank_share(&img);
    if blank > MAX_BLANK_SHARE {
        return Err(format!(
            "{whose} screenshot is flat over {:.0}% of its height, too little drawn to compare (a page that draws only what is in view looks like that when it is photographed in one picture)",
            blank * 100.0
        ));
    }
    Ok(img)
}

/// Compare one original page's capture folder with the prototype's.
pub fn compare_page(page: &str, original: &Path, prototype: &Path) -> PageLikeness {
    let mut out = PageLikeness { page: page.into(), structure: None, text: None, picture: None, width: None, overall: None, not_measured: Vec::new() };
    let (Some(a), Some(b)) = (read_capture(original), read_capture(prototype)) else {
        let missing = if original.join("capture.json").is_file() { "the prototype's" } else { "the original's" };
        out.not_measured.push(format!("{missing} capture.json is missing or unreadable"));
        return out;
    };
    match website::structure_likeness(&a, &b) {
        Some(s) => out.structure = Some(s.similarity),
        None => out.not_measured.push("structure: the original's outline has no page parts (an empty outline, as shadow DOM gave before it was read)".into()),
    }
    match website::text_likeness(&a, &b) {
        Some(t) => out.text = Some(t.similarity),
        None => out.not_measured.push("text: the original's capture has no visible text".into()),
    }
    let shared: Vec<u32> = widths(original).into_iter().filter(|w| widths(prototype).contains(w)).collect();
    match shared.last() {
        None => out.not_measured.push("picture: no screenshot at the same width in both captures".into()),
        Some(&w) => match picture(original, w, "the original's").and_then(|a| Ok((a, picture(prototype, w, "the prototype's")?))) {
            Ok((a, b)) => {
                let (cw, ch) = (a.width.min(b.width), a.height.min(b.height));
                let (a, b) = (web_common::crop(&a, cw, ch), web_common::crop(&b, cw, ch));
                let layout = web_common::block_similarity(&a, &b, 16, 24);
                out.picture = Some((layout + web_common::perceptual_similarity(&a, &b)) * 50.0);
                out.width = Some(w);
            }
            Err(why) => out.not_measured.push(format!("picture: {why}")),
        },
    }
    let scored: Vec<f64> = [out.structure, out.picture].into_iter().flatten().collect();
    if !scored.is_empty() {
        out.overall = Some(scored.iter().sum::<f64>() / scored.len() as f64);
    }
    out
}

/// The original's pages: `(page, capture folder)`. Routes come after the start page.
fn original_pages(ws: &Workspace) -> Vec<(String, PathBuf)> {
    let capture = ws.capture_dir();
    let mut pages = Vec::new();
    if capture.join("capture.json").is_file() {
        pages.push((START_PAGE.to_string(), capture.clone()));
    }
    pages.extend(website::routes(ws).into_iter().filter(|r| r.slug != START_PAGE).map(|r| (r.slug.clone(), capture.join("routes").join(r.slug))));
    pages
}

fn prototype_dir(ws: &Workspace) -> PathBuf {
    redesign_dir(ws).join(PROTOTYPE_CAPTURE_DIR)
}

/// The widths the original was photographed at; the usual two when it has no screenshots.
pub fn original_widths(ws: &Workspace) -> Vec<u32> {
    let found = widths(&ws.capture_dir());
    if found.is_empty() { vec![1440, 390] } else { found }
}

/// Which prototype addresses to capture, as `(page, path)`: the start page and every original route
/// at `/` from the prototype's own `/`, then what the caller says stands for the other pages. A page
/// that isn't one of the original's is refused: there would be nothing to compare it with.
pub fn capture_targets(ws: &Workspace, extra: &[(String, String)]) -> Result<Vec<(String, String)>, ToolError> {
    let mut known = vec![START_PAGE.to_string()];
    let mut targets = vec![(START_PAGE.to_string(), "/".to_string())];
    for r in website::routes(ws).into_iter().filter(|r| r.slug != START_PAGE) {
        if r.path.trim_end_matches('/').is_empty() {
            targets.push((r.slug.clone(), "/".to_string()));
        }
        known.push(r.slug);
    }
    for (page, path) in extra {
        if !known.contains(page) {
            return Err(ToolError::invalid(format!("`{page}` is not a page of the original; its pages are: {}", known.join(", "))));
        }
        if !path.starts_with('/') || path.contains("://") {
            return Err(ToolError::invalid(format!("the address for `{page}` must be a path in the prototype, like `/` or `/#/library`")));
        }
        targets.retain(|(p, _)| p != page);
        targets.push((page.clone(), path.clone()));
    }
    Ok(targets)
}

/// An empty folder for one page's prototype capture: `web_site_capture` wants one that isn't there.
pub fn fresh_page_dir(ws: &Workspace, page: &str) -> Result<PathBuf, ToolError> {
    redesign_store::ensure(ws)?;
    let root = prototype_dir(ws);
    reject_symlink(&root)?;
    fs::create_dir_all(&root).map_err(|e| ToolError::io(root.display(), e))?;
    let dir = safe_child(&root, page)?;
    reject_symlink(&dir)?;
    if dir.is_dir() {
        fs::remove_dir_all(&dir).map_err(|e| ToolError::io(dir.display(), e))?;
    }
    Ok(dir)
}

/// Compare every original page with its prototype capture and save the report.
pub fn measure(ws: &Workspace) -> Result<Value, ToolError> {
    redesign_store::ensure(ws)?;
    let root = prototype_dir(ws);
    reject_symlink(&root)?;
    // The folder exists from here on, so `web_site_capture` can make each page's folder in it.
    fs::create_dir_all(&root).map_err(|e| ToolError::io(root.display(), e))?;
    let originals = original_pages(ws);
    let mut pages = Vec::new();
    for (page, original) in &originals {
        let prototype = safe_child(&root, page)?;
        reject_symlink(&prototype)?;
        pages.push(compare_page(page, original, &prototype));
    }
    let report = LikenessReport { schema_version: SCHEMA_VERSION.into(), pages };
    write_record(&root.join(LIKENESS_FILE), &report)?;
    Ok(json!({
        "path": format!("{}/{PROTOTYPE_CAPTURE_DIR}/{LIKENESS_FILE}", redesign_store::DIR),
        "pages": report.pages,
        "not_measured": report.unmeasured(),
        "how": format!(
            "For each original page, open the prototype page that stands for it with web_site_open and save it with web_site_capture \
             (same widths as the original) into {}/<page>/, where <page> is one of: {}. Then run this tool again. \
             Scores are 0..=100; text is reported but not scored, because prototypes use simulated data.",
            root.display(),
            if originals.is_empty() { "(the original has no captured page)".to_string() } else { originals.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>().join(", ") }
        ),
        "limits": ["The scores are as old as the prototype captures: capture again after changing the app.", "One state of each page; menus and dialogs are not compared."],
    }))
}

/// The widths a captured page is wider than its window at, as (window, page) in px.
fn wider_than_window(dir: &Path) -> Vec<(u64, u64)> {
    let Some(capture) = read_capture(dir) else { return Vec::new() };
    let layouts = capture["layouts"].as_array().cloned().unwrap_or_default();
    layouts.iter().filter_map(|l| Some((l["width"].as_u64()?, l["page_w"].as_u64()?))).filter(|(window, page)| *page > window + 1).collect()
}

pub const FITS_CHECK_ID: &str = "page_fits_window";

/// Advisory: whether a captured prototype page runs off the side of its window (it scrolls
/// sideways), and whether the original's page did. As old as the prototype captures.
pub fn fits_window_check(ws: &Workspace) -> CheckResult {
    let said = |(window, page): &(u64, u64)| format!("{page} px wide in a {window} px window");
    let (mut captured, mut wider, mut original_wider) = (0u32, Vec::new(), Vec::new());
    for (page, original) in original_pages(ws) {
        let prototype = prototype_dir(ws).join(&page);
        if !prototype.join("capture.json").is_file() {
            continue;
        }
        captured += 1;
        let before = wider_than_window(&original);
        original_wider.extend(before.iter().map(|w| format!("{page}: {}", said(w))));
        for now in wider_than_window(&prototype) {
            let original_too = if before.iter().any(|(window, _)| *window == now.0) { "the original's page is too" } else { "the original's page fits" };
            wider.push(format!("{page}: {} ({original_too})", said(&now)));
        }
    }
    let (status, summary) = if captured == 0 {
        ("not_evaluated", "the prototype was not captured: run redesign_prototype_capture".to_string())
    } else if !wider.is_empty() {
        ("warn", format!("the prototype scrolls sideways: {}", wider.join("; ")))
    } else if original_wider.is_empty() {
        ("pass", format!("no captured prototype page is wider than its window ({captured} page(s))"))
    } else {
        (
            "pass",
            format!("no captured prototype page is wider than its window ({captured} page(s)); the original scrolled sideways: {}", original_wider.join("; ")),
        )
    };
    CheckResult {
        id: FITS_CHECK_ID.into(),
        status: status.into(),
        summary,
        details: json!({"advisory": true, "pages": captured, "wider": wider, "original_wider": original_wider, "note": "Advisory: read from the prototype's captures, which are as old as the last redesign_prototype_capture."}),
    }
}

pub const BASICS_CHECK_ID: &str = "prototype_basics";

/// What one captured prototype page still gets wrong, from its capture's `a11y` (and `health` in
/// Deep mode): what is worse than the original ([`website::health_notes`]), then what it has at
/// all, since a redesign is not excused by the original's own gaps.
fn basics_of(original: &Value, prototype: &Value) -> Vec<String> {
    let mut out = website::health_notes(original, prototype);
    let Some(a11y) = prototype.get("a11y") else { return out };
    let count = |v: &Value, key: &str| v.pointer(&format!("/a11y/{key}")).and_then(Value::as_u64).unwrap_or(0);
    for (key, what) in [("unnamed_controls", "control(s) without a name"), ("images_without_alt", "picture(s) without alt text")] {
        let (now, before) = (count(prototype, key), count(original, key));
        // health_notes already said it when the prototype has more than the original.
        if now > 0 && now <= before {
            let examples: Vec<&str> = if key == "unnamed_controls" {
                a11y["unnamed_examples"].as_array().into_iter().flatten().filter_map(Value::as_str).collect()
            } else {
                Vec::new()
            };
            let shown = if examples.is_empty() { String::new() } else { format!(": {}", examples.join(", ")) };
            out.push(format!("{now} {what}{shown} (the original has {before})"));
        }
    }
    let lang = |v: &Value| v.pointer("/a11y/lang").and_then(Value::as_str).unwrap_or_default().trim().to_string();
    if lang(prototype).is_empty() && lang(original).is_empty() {
        out.push("the page language isn't set (nor is the original's)".into());
    }
    let landmarks =
        |v: &Value| -> Vec<String> { v.pointer("/a11y/landmarks").and_then(Value::as_object).map(|m| m.keys().cloned().collect()).unwrap_or_default() };
    let ours = landmarks(prototype);
    let missing: Vec<String> = landmarks(original).into_iter().filter(|l| !ours.contains(l)).collect();
    if !missing.is_empty() {
        out.push(format!("no {} landmark, which the original has", missing.join(", ")));
    }
    out
}

/// Advisory: the basics each captured prototype page gets wrong (controls without a name, pictures
/// without alt text, no page language, a landmark of the original it lacks, and errors or failed
/// requests when the captures recorded them). As old as the prototype captures; no certification.
pub fn basics_check(ws: &Workspace) -> CheckResult {
    let (mut captured, mut found) = (0u32, Vec::new());
    for (page, original) in original_pages(ws) {
        let Some(prototype) = read_capture(&prototype_dir(ws).join(&page)) else { continue };
        captured += 1;
        let original = read_capture(&original).unwrap_or(Value::Null);
        found.extend(basics_of(&original, &prototype).into_iter().map(|note| format!("{page}: {note}")));
    }
    let (status, summary) = if captured == 0 {
        ("not_evaluated", "the prototype was not captured: run redesign_prototype_capture".to_string())
    } else if found.is_empty() {
        (
            "pass",
            format!(
                "{captured} captured page(s): every control has a name, every picture has alt text, the language is set and the original's landmarks are there"
            ),
        )
    } else {
        ("warn", found.join("; "))
    };
    CheckResult {
        id: BASICS_CHECK_ID.into(),
        status: status.into(),
        summary,
        details: json!({"advisory": true, "wcag_certified": false, "pages": captured, "found": found, "note": "Advisory: read from the prototype's captures, which are as old as the last redesign_prototype_capture. Not an accessibility audit."}),
    }
}

/// The saved report, if any. A report that can't be read counts as none.
pub fn load(ws: &Workspace) -> Option<LikenessReport> {
    serde_json::from_slice(&fs::read(prototype_dir(ws).join(LIKENESS_FILE)).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::tools::web_common::encode_png;

    const PART_PX: u32 = 200;

    /// A page whose parts sit at the given heights, each drawn as a striped block on white.
    fn capture(dir: &Path, parts: &[(&str, u32)], blank_from: Option<u32>) {
        fs::create_dir_all(dir).unwrap();
        let outline: Vec<Value> =
            parts.iter().map(|(tag, y)| json!({"tag": tag, "depth": 1, "box": {"x": 0, "y": y, "w": 640, "h": PART_PX}, "text": tag})).collect();
        fs::write(dir.join("capture.json"), json!({"text": "catalogue search results", "outline": outline}).to_string()).unwrap();
        let (w, h) = (640u32, 960u32);
        let mut rgba = vec![255u8; (w * h * 4) as usize];
        for (tag, y) in parts {
            for row in *y..(*y + PART_PX).min(blank_from.unwrap_or(h)) {
                for x in 0..w {
                    let at = ((row * w + x) * 4) as usize;
                    // A different texture per kind of part, so a moved part is a different picture.
                    let shade = if (x / (8 + 8 * tag.len() as u32)).is_multiple_of(2) { 20 } else { 90 };
                    rgba[at..at + 3].copy_from_slice(&[shade, shade, shade]);
                }
            }
        }
        fs::write(dir.join("screenshot-640.png"), encode_png(&Image { width: w, height: h, rgba }).unwrap()).unwrap();
    }

    const PAGE: [(&str, u32); 4] = [("header", 0), ("nav", 220), ("main", 440), ("footer", 700)];

    #[test]
    fn the_same_page_is_more_alike_than_a_reordered_one_which_is_more_alike_than_another_page() {
        let d = tempfile::tempdir().unwrap();
        let at = |name: &str| d.path().join(name);
        capture(&at("original"), &PAGE, None);
        capture(&at("same"), &PAGE, None);
        capture(&at("reordered"), &[("nav", 0), ("header", 220), ("footer", 440), ("main", 700)], None);
        capture(&at("other"), &[("form", 0), ("aside", 220), ("article", 440), ("section", 700)], None);
        let overall = |name: &str| compare_page("p", &at("original"), &at(name)).overall.unwrap();
        let (same, reordered, other) = (overall("same"), overall("reordered"), overall("other"));
        assert!(same > reordered, "same {same} vs reordered {reordered}");
        assert!(reordered > other, "reordered {reordered} vs other {other}");
    }

    #[test]
    fn a_picture_with_large_blank_bands_is_not_compared_and_says_why() {
        let d = tempfile::tempdir().unwrap();
        let at = |name: &str| d.path().join(name);
        capture(&at("drawn"), &PAGE, None);
        // The same page photographed with everything below the first part left undrawn.
        capture(&at("blank"), &PAGE, Some(PART_PX));
        let cut = compare_page("p", &at("blank"), &at("drawn"));
        assert_eq!(cut.picture, None, "{cut:?}");
        assert!(cut.not_measured.iter().any(|r| r.contains("flat")), "{cut:?}");
        assert!(cut.structure.is_some(), "the outline is still compared: {cut:?}");
        assert!(compare_page("p", &at("drawn"), &at("drawn")).picture.is_some(), "a drawn page with white space is compared");
    }

    /// A page shorter than the window is all in the picture: the empty space under it is neither a
    /// reason to refuse it nor something two pages have in common.
    #[test]
    fn a_page_shorter_than_the_window_is_compared_over_what_it_draws() {
        let d = tempfile::tempdir().unwrap();
        let at = |name: &str| d.path().join(name);
        let short = |name: &str, parts: &[(&str, u32)]| {
            capture(&at(name), parts, None);
            let mut c = read_capture(&at(name)).unwrap();
            c["viewport"] = json!({"w": 640, "h": 960});
            fs::write(at(name).join("capture.json"), c.to_string()).unwrap();
        };
        short("results", &[("header", 0), ("main", 220)]);
        short("same", &[("header", 0), ("main", 220)]);
        short("other", &[("form", 0), ("aside", 220)]);
        short("empty", &[]);
        capture(&at("tall"), &[("header", 0), ("main", 220)], None);

        let picture = |name: &str| compare_page("p", &at("results"), &at(name));
        let (same, other) = (picture("same").picture.expect("a short page is compared"), picture("other").picture.unwrap());
        assert!(same > other, "same {same} vs other {other}");
        // Compared over the whole window, the white under both pages would make them look alike.
        let whole = |name: &str| web_common::decode_png(&fs::read(at(name).join("screenshot-640.png")).unwrap()).unwrap();
        let over_the_window = (web_common::block_similarity(&whole("results"), &whole("other"), 16, 24)
            + web_common::perceptual_similarity(&whole("results"), &whole("other")))
            * 50.0;
        assert!(other < over_the_window, "over what is drawn {other} vs over the window {over_the_window}");

        let nothing = picture("empty");
        assert!(nothing.picture.is_none() && nothing.not_measured.iter().any(|r| r.contains("nothing drawn")), "{nothing:?}");
        let undrawn = picture("tall");
        assert!(undrawn.not_measured.iter().any(|r| r.contains("flat")), "the same picture of a page taller than the window is still refused: {undrawn:?}");
    }

    /// A prototype that runs off the side of a narrow window is told so, with whether the original
    /// did; one that fits where the original did not says what was gained.
    #[test]
    fn a_prototype_wider_than_its_window_is_warned_about_and_one_that_fits_is_not() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "L", "kind=redesign", craft_anything_engine::workspace::TargetRef::None).unwrap();
        let layouts = |narrow: u64| json!({"layouts": [{"width": 1440, "page_w": 1440}, {"width": 390, "page_w": narrow}]}).to_string();
        fs::create_dir_all(ws.capture_dir()).unwrap();
        fs::write(ws.capture_dir().join("capture.json"), layouts(1120)).unwrap();
        assert_eq!(fits_window_check(&ws).status, "not_evaluated", "nothing to read before the prototype is captured");

        let prototype = prototype_dir(&ws).join(START_PAGE);
        fs::create_dir_all(&prototype).unwrap();
        fs::write(prototype.join("capture.json"), layouts(640)).unwrap();
        let wide = fits_window_check(&ws);
        assert_eq!(wide.status, "warn", "{wide:?}");
        assert!(wide.summary.contains("640 px wide in a 390 px window") && wide.summary.contains("the original's page is too"), "{}", wide.summary);

        fs::write(prototype.join("capture.json"), layouts(390)).unwrap();
        let fits = fits_window_check(&ws);
        assert_eq!(fits.status, "pass", "{fits:?}");
        assert!(fits.summary.contains("1120 px wide in a 390 px window"), "it says the original did not fit: {}", fits.summary);

        fs::write(ws.capture_dir().join("capture.json"), layouts(390)).unwrap();
        fs::write(prototype.join("capture.json"), layouts(640)).unwrap();
        assert!(fits_window_check(&ws).summary.contains("the original's page fits"), "a prototype that made it worse is told so");
    }

    /// A prototype is told what it still gets wrong, whether or not the original got it wrong too,
    /// and is not told about what only the original had.
    #[test]
    fn a_prototype_is_told_about_unnamed_controls_and_missing_landmarks_and_not_about_what_it_fixed() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "L", "kind=redesign", craft_anything_engine::workspace::TargetRef::None).unwrap();
        let page = |lang: &str, unnamed: u64, without_alt: u64, landmarks: Value| {
            json!({"a11y": {"lang": lang, "unnamed_controls": unnamed, "unnamed_examples": ["a.icon"], "images_without_alt": without_alt, "landmarks": landmarks}}).to_string()
        };
        fs::create_dir_all(ws.capture_dir()).unwrap();
        fs::write(ws.capture_dir().join("capture.json"), page("pt", 8, 3, json!({"main": 1, "search": 1}))).unwrap();
        assert_eq!(basics_check(&ws).status, "not_evaluated");

        let prototype = prototype_dir(&ws).join(START_PAGE);
        fs::create_dir_all(&prototype).unwrap();
        fs::write(prototype.join("capture.json"), page("pt", 0, 0, json!({"main": 1, "search": 1, "navigation": 2}))).unwrap();
        let clean = basics_check(&ws);
        assert_eq!(clean.status, "pass", "what only the original had is not held against the prototype: {clean:?}");

        fs::write(prototype.join("capture.json"), page("pt", 2, 0, json!({"main": 1}))).unwrap();
        let fewer = basics_check(&ws);
        assert_eq!(fewer.status, "warn", "{fewer:?}");
        assert!(
            fewer.summary.contains("2 control(s) without a name") && fewer.summary.contains("a.icon"),
            "fewer than the original is still told: {}",
            fewer.summary
        );
        assert!(fewer.summary.contains("no search landmark"), "{}", fewer.summary);

        fs::write(prototype.join("capture.json"), page("", 9, 0, json!({"main": 1, "search": 1}))).unwrap();
        let worse = basics_check(&ws);
        assert!(
            worse.summary.contains("9 controls without a name") && worse.summary.contains("language"),
            "more than the original, and a lost language: {}",
            worse.summary
        );
    }

    #[test]
    fn the_start_page_is_always_captured_and_only_the_originals_pages_are_accepted() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "L", "kind=redesign", craft_anything_engine::workspace::TargetRef::None).unwrap();
        fs::create_dir_all(ws.capture_dir()).unwrap();
        let routes = json!([{"path": "/", "slug": "home", "url": "https://example.test/"}, {"path": "/details/", "slug": "details", "url": "https://example.test/details/"}]);
        fs::write(ws.capture_dir().join("routes.json"), routes.to_string()).unwrap();
        let pages = |extra: &[(&str, &str)]| capture_targets(&ws, &extra.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect::<Vec<_>>());
        let plain: Vec<String> = pages(&[]).unwrap().into_iter().map(|(p, _)| p).collect();
        assert_eq!(plain, [START_PAGE, "home"], "the page at / needs no address; the one elsewhere does");
        assert!(pages(&[("details", "/#/library")]).unwrap().contains(&("details".to_string(), "/#/library".to_string())));
        assert!(pages(&[("pricing", "/")]).is_err(), "nothing to compare it with");
        assert!(pages(&[("details", "https://elsewhere.test/")]).is_err(), "only the prototype is opened");
    }

    #[test]
    fn a_page_without_a_prototype_capture_is_named_and_the_start_page_is_measured() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "L", "kind=redesign", craft_anything_engine::workspace::TargetRef::None).unwrap();
        capture(&ws.capture_dir(), &PAGE, None);
        capture(&ws.capture_dir().join("routes/details"), &PAGE, None);
        fs::write(ws.capture_dir().join("routes.json"), json!([{"path": "/details", "slug": "details", "url": "https://example.test/details"}]).to_string())
            .unwrap();

        measure(&ws).unwrap();
        capture(&prototype_dir(&ws).join(START_PAGE), &PAGE, None);
        measure(&ws).unwrap();
        let report = load(&ws).unwrap();
        assert_eq!(report.measured().map(|(p, _)| p).collect::<Vec<_>>(), [START_PAGE], "{report:?}");
        assert!(report.unmeasured().iter().any(|r| r.starts_with("details:") && r.contains("prototype")), "{report:?}");
    }
}
