# DX, OSS, dependencies and upstream — v1.3.0

## Agent-first
`README` and `AGENTS.md` must let an agent read the URL, understand the project and run `doctor` without implicit installations. Bootstrap is assisted: present the plan, dependencies, commands and risks, and ask for consent before installing, compiling or registering an MCP. Compatibility must not be announced before real smoke tests in Cursor, Claude Code and Codex.

## Compatibility matrix
Record for each OS/agent: version, authentication, CLI/MCP, bootstrap, permissions, build, check, resume, token usage (reported or unavailable), result and limitations. Minimum for release: Linux/macOS/Windows depending on CI availability and real tests; untested platforms must be labeled unverified.

## Private design systems
Capture name, exact version, lockfile/digest, origin, token and component hashes, license and redistribution permissions. Do not copy private Storybook, Figma, assets, fonts or components into a public package. Segregated cache, pinned version; a hash divergence requires revalidation.

## Fork/upstream
Record the origin SHA, license/attribution and local patches. Upstream updates go on a dedicated branch with regression CI, migrations and a comparison of the seams (`TargetKind`, `Check`, `ToolSpec`, MCP, CLI). Periodic review of CVEs/dependencies. An ADR is required if the core delta grows or the extension API breaks; compare keeping the fork against external skill/CLI/MCP alternatives.

## Accessibility
Automation (axe, semantics, contrast) **does not replace** keyboard, focus, 200%/400% zoom as applicable to the scenario, screen readers on a sample, and user testing when necessary. The result must state which modalities were tested.

## Tests
DX-01 doctor neither installs nor runs postinstall.
DX-02 installation asks for consent and offers rollback.
DX-03 DS version changes: cache invalidated, checks repeated.
DX-04 private asset does not appear in the ZIP/telemetry.
DX-05 upstream merge preserves the Craft and redesign tests.
DX-06 docs do not claim support for an unverified platform.
DX-07 keyboard, focus and zoom are included in the verification plan.
**Status:** future criteria; not verified in real execution.
