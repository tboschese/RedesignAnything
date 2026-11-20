//! Redesign Anything kind: evidence-based redesign of an existing web app.
//! Selected only by an explicit project `kind` / CLI/MCP `kind=redesign` — never by ambiguous
//! wording that would steal ordinary website or app projects.

use super::*;

pub struct RedesignKind;

impl RedesignKind {
    pub const ID: &'static str = "redesign";
    /// Default stack for redesign projects (React + TypeScript via Vite).
    pub const DEFAULT_STACK: &'static str = "React + TypeScript (Vite)";
}

/// Brief contract shared with builders and host agents.
pub const REDESIGN_BRIEF: &str = concat!(
    "This is a Redesign Anything project (kind=redesign), not a visual clone.\n",
    "- Preserve capabilities that Study observed and recorded; do not invent backend contracts.\n",
    "- Prefer meaningful UX structure over pixel likeness to the original; visual similarity is not a pass/fail gate.\n",
    "- Offer keep_as_is as a valid option when change isn't justified.\n",
    "- Mark unknowns explicitly; never promote inferred rules to validated without an independent oracle.\n",
    "- Default stack is React + TypeScript (Vite) unless the person named another supported stack.\n",
    "- Write the app under app/; do not modify the original target. Use synthetic/mock data when the backend is unknown."
);

impl TargetKind for RedesignKind {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn words(&self) -> KindWords {
        KindWords {
            button: "Open the redesign",
            icon: "layout-template",
            ready: "Your redesign is ready",
            change_hint: "Ask for a change, e.g. \"simplify the form\" or \"keep the submit flow as is\"",
        }
    }

    /// Explicit markers only. Ordinary phrases like "redesign this page" must not win over website.
    fn detect(&self, cx: &DetectCx) -> Option<u32> {
        if cx.says(&[" kind=redesign ", " [kind=redesign] ", " mode=redesign "]) {
            Some(200)
        } else {
            None
        }
    }

    fn default_stack(&self, _cx: &DetectCx) -> &'static str {
        Self::DEFAULT_STACK
    }

    fn goal(&self) -> &'static str {
        "a redesign that preserves known capabilities with clearer UX, not a pixel clone"
    }

    fn self_screenshots(&self) -> bool {
        false
    }

    fn brief_sections(&self, cx: &BriefCx) -> Vec<String> {
        let mut out = vec![REDESIGN_BRIEF.into()];
        // Compact redesign refs — pass by path, do not dump full stores into the prompt.
        out.push(
            "Redesign workspace refs (read these files; do not invent contracts): redesign/store.json, \
             redesign/capabilities/, redesign/decisions/, redesign/design_system.json, \
             redesign/transformation_plan.json, redesign/scaffold_refs.json. \
             Prefer redesign_scaffold_app under host_agent instead of nested rebuild_start."
                .into(),
        );
        // Reuse website capture notes when Study photographed the target.
        let routes = website::routes(cx.ws);
        if !routes.is_empty() {
            let list = routes.iter().map(|r| format!("{} (capture/routes/{}/)", r.path, r.slug)).collect::<Vec<_>>().join(", ");
            out.push(format!(
                "Study captured related pages: {list}. Preserve navigation and task flows that were observed; list pages not captured as unknowns."
            ));
        }
        let states = website::pictured_states(cx.ws);
        if !states.is_empty() {
            out.push(format!(
                "Observed UI states (menus, dialogs, tabs): {}. Preserve those behaviours when redesigning; do not invent states that were not observed.",
                states.join(", ")
            ));
        }
        if !cx.ws.manifest.own_project() {
            out.push(website::HOW_TO_RUN.into());
        }
        out
    }

    fn checks(&self, _ws: &Workspace) -> Vec<Box<dyn Check>> {
        // No WebVisualCheck / pixel-similarity gate. Build success is the first native check;
        // richer redesign checks land in later PRs.
        vec![Box::new(BuildCheck)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use craft_anything_engine::workspace::TargetRef;

    #[test]
    fn redesign_is_only_chosen_with_an_explicit_marker() {
        let kinds = KindRegistry::built_in();
        let web = TargetRef::Website { url: "https://example.test".into() };
        assert_eq!(kinds.detect("Redesign this website please", &web, "").unwrap().id(), "website");
        assert_eq!(kinds.detect("Clone this website kind=redesign", &web, "").unwrap().id(), "redesign");
        assert_eq!(kinds.get("redesign").unwrap().default_stack(&DetectCx::new("", &web, "")), RedesignKind::DEFAULT_STACK);
    }

    #[test]
    fn redesign_checks_do_not_include_visual_likeness() {
        let d = tempfile::tempdir().unwrap();
        let ws = Workspace::create(d.path(), "R", "kind=redesign", TargetRef::Website { url: "https://example.test".into() }).unwrap();
        let ids: Vec<_> = RedesignKind.checks(&ws).iter().map(|c| c.id()).collect();
        assert_eq!(ids, [CheckResult::BUILD]);
        assert!(!ids.contains(&CheckResult::WEB_VISUAL));
    }
}
