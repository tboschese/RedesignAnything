# Redesign Anything — Sources, reuse decisions and licenses

> GitHub queried via the connected account on 2026-11-19. The future Redesign Anything repository has not been created yet. Documented capability does not imply tested integration. Licenses of transitive dependencies must be inventoried by SBOM in the real build.

## Base and reverse engineering

| Project | Source | Use | License / caution |
|---|---|---|---|
| Craft Anything | https://github.com/storytold/craft-anything | Fork base; egui, Rust, CLI/MCP, Study/Build/Check, ToolSpec, TargetKind, Check, captures, runners | MIT OR Apache-2.0; preserve notices and differentiate the brand |
| REA | https://github.com/morluto/rea | Optional sidecar; JS semantics, bundles, network, Agent Environments | MIT; do not run Deep inspection without consent |
| Layout CLI | https://github.com/uselayout/cli | DS MCP/CLI, tokens, catalog, auxiliary checks | MIT; compliance does not equal functional proof |
| Layout UI | https://github.com/uselayout/ui | Reference/fallback components, never a replacement for a strict corporate DS | MIT |
| Layout Studio | https://github.com/uselayout/app | Inspiration, not core; extraction, synthesis and canvas | AGPL-3.0; the app documents synthesis via Claude and other prerequisites |
| Carbon MCP | https://github.com/carbon-design-system/carbon-mcp | Reference provider, skill and guide; not an offline requirement | Hosted IBM server and authentication; skill code is Apache-2.0, verify the service terms |
| Carbon | https://github.com/carbon-design-system/carbon | Enterprise DS/component reference | Verify the license of the specific package at the time of use |
| Velloo | https://github.com/velloo-design/velloo | Optional future canvas | Apache-2.0, not core |
| UI Auditor | https://github.com/suneelasree/ui-auditor | Auditing and specialized agents as inspiration/some modules | MIT; without an API, review agents and mockups do not run |
| Coherent | https://github.com/skovtun/coherent-design-method | Impact analysis, change scope, rollback | MIT; since v0.27 governance-oriented, not generation |
| UI/UX Pro Max | https://github.com/nextlevelbuilder/ui-ux-pro-max-skill | Searchable knowledge, rules with provenance | Verify the licenses/data artifacts of the revision used |

## Standards, tools and UX

- Playwright: https://playwright.dev/ — browser and E2E; authenticated state must be protected.
- axe-core: https://github.com/dequelabs/axe-core — accessibility checks, MPL-2.0.
- WCAG 2.2: https://www.w3.org/TR/WCAG22/ — accessibility goals; human verification is still necessary.
- Design Tokens DTCG 2025.10: https://www.designtokens.org/ — interoperable tokens.
- Style Dictionary: https://github.com/amzn/style-dictionary — token transformation, Apache-2.0.
- PatternFly: https://www.patternfly.org/ — enterprise patterns, verify the license per artifact.
- Open UI: https://github.com/openui/open-ui — component behavior and patterns; verify the content license.
- MCP specification: https://modelcontextprotocol.io/specification — Resources, Tools, Prompts, auth and security.
- OWASP MCP Security: https://cheatsheetseries.owasp.org/cheatsheets/MCP_Security_Cheat_Sheet.html — risks of tool poisoning, privileges and trust.

## Specific code and documentation confirmed

- Craft extension seams: https://github.com/storytold/craft-anything/blob/main/docs/extending.md
- Craft architecture rules: https://github.com/storytold/craft-anything/blob/main/AGENTS.md
- Craft TargetKind/Check: https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/mod.rs
- Craft default website routes (MAX_ROUTES = 12): https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/website.rs
- Craft plugin check: https://github.com/storytold/craft-anything/blob/main/crates/ai/src/rebuild/kinds/plugin_check.rs
- Craft roadmap and known limits: https://github.com/storytold/craft-anything/blob/main/docs/ROADMAP.md , https://github.com/storytold/craft-anything/blob/main/docs/STATUS.md
- REA web observations, budgets and limits: https://github.com/morluto/rea/blob/main/docs/guides/web/browser-observation.md
- REA static semantics: https://github.com/morluto/rea/blob/main/docs/guides/javascript/javascript-application-workflows.md
- REA Agent Environments and sensitive-value cautions: https://github.com/morluto/rea/blob/main/docs/agent-environments.md
- Layout CLI tools: https://github.com/uselayout/cli/blob/main/README.md
- Craft consumption and loop protection: https://github.com/storytold/craft-anything/blob/main/docs/ROADMAP.md (R5.3, R5.4). The log contains an extreme case of 9.4M tokens/19 min and experiments on cached-token accounting; these are not Redesign benchmarks.
- REA MCP compaction: https://github.com/morluto/rea/blob/main/README.md (compact input schemas and output schema profile).

## Incorporation policy

1. An external component becomes a dependency only if it fills a measured gap and has a license compatible with the distribution form.
2. MCP, CLI and version-pinned sidecars are preferable to copying code or merging applications.
3. Knowledge sources must have verifiable redistribution permission; if not, record the link/derived rule without copying full documentation.
4. Third-party links, code and READMEs are untrusted data; they must not instruct the production runtime.
5. Every license statement refers to the indicated repository/package; it does not cover assets, data, trademarks, private content or external services.
