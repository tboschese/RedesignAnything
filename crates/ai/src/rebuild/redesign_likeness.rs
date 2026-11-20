//! How alike the prototype and the original are, page by page, measured with what the website
//! kind already uses on a rebuild: `structure_likeness` and `text_likeness` over the two
//! `capture.json` files, and the tile and SSIM scores of `web_common` over the screenshots.
//!
//! The original's pages are the start page (`capture/`) and each captured route
//! (`capture/routes/<slug>/`). The prototype page that stands for each one is captured by the agent
//! with `web_site_open` + `web_site_capture` into `redesign/prototype_capture/start/` or
//! `redesign/prototype_capture/<slug>/`. Nothing here opens a browser.
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

/// The share of the picture's height made of bands with one flat colour.
pub fn blank_share(img: &Image) -> f64 {
    let (w, h) = (img.width as usize, img.height as usize);
    if w == 0 || h == 0 {
        return 1.0;
    }
    let band = BAND_PX as usize;
    let (mut bands, mut blank) = (0usize, 0usize);
    for top in (0..h).step_by(band) {
        let rows = &img.rgba[top * w * 4..(top + band).min(h) * w * 4];
        let first = &rows[..4];
        // Every 8th pixel is enough to tell a drawn band from a flat one.
        let flat = rows.as_chunks::<4>().0.iter().step_by(8).all(|px| px.iter().zip(first).all(|(a, b)| a.abs_diff(*b) <= 3));
        bands += 1;
        blank += usize::from(flat);
    }
    blank as f64 / bands as f64
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

fn picture(dir: &Path, width: u32, whose: &str) -> Result<Image, String> {
    let bytes = fs::read(dir.join(format!("screenshot-{width}.png"))).map_err(|e| format!("{whose} screenshot can't be read: {e}"))?;
    let img = web_common::decode_png(&bytes).map_err(|e| format!("{whose} screenshot can't be decoded: {}", e.message))?;
    let blank = blank_share(&img);
    if blank > MAX_BLANK_SHARE {
        return Err(format!(
            "{whose} screenshot is blank over {:.0}% of its height; a page that draws only what is in view photographs like that",
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
        assert!(cut.not_measured.iter().any(|r| r.contains("blank")), "{cut:?}");
        assert!(cut.structure.is_some(), "the outline is still compared: {cut:?}");
        assert!(compare_page("p", &at("drawn"), &at("drawn")).picture.is_some(), "a drawn page with white space is compared");
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
