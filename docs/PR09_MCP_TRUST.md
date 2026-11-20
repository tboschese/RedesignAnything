# PR-09 — Application MCP Trust Gateway

## Tools

- `redesign_mcp_catalogue` — schema hash, effects, allowlist, hostility / fake-readOnly blocks
- `redesign_mcp_gate` — per-call allow/block; secret bodies refused; schema changes need reapproval

## Guarantees tested

- Fake `readOnly` write tools blocked
- Hostile descriptions blocked
- Safe `get_rule` → knowledge `declared` (not observed)
- Schema hash change blocked
- Secret-looking body refused
- Empty catalogue: redesign lean path still works
