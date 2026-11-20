# Trust Boundaries — normative requirements v1.3.0

## Principle
REA/Redesign Anything is **not** a sandbox for the host agent. `host_agent` controls commands, network access, models and permissions according to the Cursor/Claude/Codex configuration. REA controls only its own tools, files, subprocesses and responses. No report may claim global isolation without independent validation of the host.

## Boundaries
| Boundary | Control Redesign can implement | Outside its control |
|---|---|---|
| Own MCP/CLI | parameter validation, path allowlist, limits, redaction, timeout, confirmation for mutating operations | shell and native tools of the host agent |
| Observation browser | isolated context, temporary profiles, allowed origins, sanitized captures | browser or external plugins used directly by the host |
| Application MCP | allowlist of servers/tools, read-only by default, approval for writes, trust metadata | correctness of the rules declared by the server, direct host actions |
| Remote model | minimize and filter data sent by the product's own tools | provider retention policy and data sent directly by the host |
| Installation/dependencies | side-effect-free `doctor`, action manifest, hashes when available | `npm`, `cargo` and shell commands run outside the gateway |

## P0 requirements
- `project_scope`: allowed canonical paths, URLs/origins, authorized MCPs, read-only or write mode, blocked environments. Resolve symlinks and block traversal.
- `approval_record`: host identity when available, action, scope, justification, timestamp and result. No approval = do not run the product's own mutating action.
- `secrets`: deny export of cookies, Authorization headers, API keys, tokens and known personal data; sanitize before persistence and before MCP responses.
- `evidence`: storage segregated by project/tenant/role, configurable TTL, purge, no implicit synchronization, manifests without secret values.
- `prompt_injection`: pages, HAR files, code, docs and MCP responses are **untrusted data**; they do not elevate permissions, do not alter agent instructions and do not authorize commands.
- `dependencies`: install/build scripts in an untrusted project require explicit consent and an isolated environment when possible. Never run them from `doctor`.
- `patch_apply`: only after diff, confirmation and checks. By default, generate in a separate workspace.
- Do not promise that MCP approval prevents direct host actions. The README must state this clearly.

## Blocking tests
TB-01 host agent calls the shell directly: the report does not claim the gateway prevented it.
TB-02 malicious MCP asks for a secret and a command: the response is treated as data, nothing is executed.
TB-03 symlink/traversal tries to escape the workspace: the tool refuses.
TB-04 HAR contains a cookie/token: persisted and returned data is sanitized.
TB-05 mutating operation without approval: the product's own tool refuses.
TB-06 `doctor` in a project with a malicious postinstall: no script is executed.
TB-07 two runs/tenants: no cache/evidence leakage.
TB-08 retention expires and purge removes artifacts controlled by the product.
**Status:** specified; no control is considered implemented without a test.
