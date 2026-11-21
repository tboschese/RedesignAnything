# PR-16 — How alike the prototype is to the original

## What landed

- `redesign_likeness_measure {project}` (`crates/ai/src/rebuild/redesign_likeness.rs`) compares each original page with the prototype page captured for it and writes `redesign/prototype_capture/likeness.json`.
  - Original pages: the start page (`capture/`, page `start`) and every captured route (`capture/routes/<slug>/`).
  - Prototype pages live in `redesign/prototype_capture/start/` or `redesign/prototype_capture/<slug>/`. `redesign_prototype_capture {project, pages?, widths?}` makes them: it serves the built prototype, captures its start page (and every original route at `/`) at the original's widths, closes the browser and runs the measure. `pages` gives the prototype address that stands for another original page. It needs approval, like the browser tools it runs, and it runs no build. An agent can also capture by hand with `web_site_open` + `web_site_capture`; `redesign_likeness_measure` itself opens no browser.
  - Measures, each 0..=100: `structure` (`website::structure_likeness`), `text` (`website::text_likeness`; reported, not scored, because prototypes use simulated data), `picture` (tile layout and SSIM detail from `web_common`, over the screenshots' shared area at the widest shared width). `overall` is the mean of structure and picture.
  - A page that can't be compared is named with its reason: no prototype capture, an empty outline, no screenshot at a shared width, or a screenshot taller than the window with more than 35% of its height in blank bands.
  - A screenshot no taller than the window shows a whole short page. It is compared over the height it draws, so the empty space under it is neither refused nor counted as alike.
- `mode_scope` reports the scores and what wasn't measured in its summary. It does not judge them.
- `page_fits_window` (advisory) reads the same prototype captures: it warns when a captured prototype page is wider than its window, so it scrolls sideways, and says whether the original's page was too. It is not evaluated before the prototype is captured.
- `prototype_basics` (advisory) reads them too: controls without a name, pictures without alt text, a page language that isn't set, and a landmark of the original the prototype lacks. It reuses `website::health_notes` for what is worse than the original and adds what the prototype still has at all. It is not an accessibility audit and claims no certification.
- `copied_wording` (advisory) reads them too: a run of 10 or more words, inside one element of the page, that the prototype shares with the original's page is the original's wording copied. Labels shorter than that (menu items, subjects) are shared by design. It only informs: a prototype may keep the original's text when the person allows it, and nothing makes the builder rewrite it.
- `redesign_variants_show` gains `likeness_order`: on a page measured in two variants, the variant that changes less must not be less like the original. Out of order is a warning. Two scores that round to the same whole point are a tie.

## Why the scores are not judged against a limit

Measured on a real site: a prototype that kept the original's blocks in the same places scored 23 of 100 in structure, because the real page has dozens of tiles and images that a prototype with simulated data does not. Any fixed limit either flags every faithful prototype or none. The order between variants of the same study needs no limit: on that site it came out 23 ≥ 20 ≥ 14, and swapping two prototypes was flagged.

## Guarantees tested

- The same page is more alike than a reordered one, which is more alike than another page.
- A picture with large blank bands is not compared and says why; the outline still is.
- A page shorter than the window is compared over what it draws; one with nothing drawn says so.
- A page without a prototype capture is named, and the start page is measured.
- Likeness is reported without judging, and what was not measured is said.
- A variant that changes less but is less like the original is out of order.

## Limits

- The scores are as old as the prototype captures; nothing ties them to the current build.
- `redesign_prototype_capture` captures what the built site serves; it does not build it, and a page behind an interaction (no address of its own) is not captured.
- One state per page; menus and dialogs are not compared.
- The picture score is refused, not repaired, when the original's full-page screenshot has blank bands (see `docs/pitfalls.md`).
