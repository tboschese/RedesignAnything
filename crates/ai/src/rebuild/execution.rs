//! Single-controller policy for Redesign Anything / Craft rebuild runs.
//!
//! `host_agent` (default for MCP from Cursor/Claude/Codex) must not start the nested Craft
//! builder unless the person hands off explicitly. `craft_runner` is used by CLI/`rebuild`.

use craft_anything_evidence::{ErrorCode, ToolError};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Who owns model/execution control for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Controller {
    /// Cursor / Claude Code / Codex (or another host) drives work via CLI/MCP tools.
    HostAgent,
    /// Craft's own rebuild runner owns the builder loop (desktop, `rebuild` CLI, and tests).
    #[default]
    CraftRunner,
}

/// Lean vs deep investigation profile (orthogonal to Clean room / Deep inspection mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    #[default]
    Lean,
    Deep,
}

/// Policy attached to a session or a tool call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPolicy {
    pub controller: Controller,
    pub profile: Profile,
    /// When false, `rebuild_start` from a `host_agent` session is refused unless handoff is set.
    pub allow_nested_builder: bool,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self::craft_runner()
    }
}

impl ExecutionPolicy {
    /// Policy for the headless `rebuild` CLI / desktop: Craft owns the runner.
    pub fn craft_runner() -> Self {
        Self { controller: Controller::CraftRunner, profile: Profile::Lean, allow_nested_builder: true }
    }

    /// Policy for MCP served to Cursor/Claude/Codex: host owns the builder; no nested rebuild.
    pub fn host_agent() -> Self {
        Self { controller: Controller::HostAgent, profile: Profile::Lean, allow_nested_builder: false }
    }

    /// Refuse nested `rebuild_start` when the host agent is in control.
    pub fn guard_rebuild_start(&self, handoff_to_craft_runner: bool) -> Result<(), ToolError> {
        if self.controller == Controller::HostAgent && !self.allow_nested_builder && !handoff_to_craft_runner {
            return Err(ToolError::new(ErrorCode::BadInput, "controller_conflict: host_agent is in control; rebuild_start would nest a second builder.")
                .with_remediation(
                    "Keep control in the host agent (use redesign_project_init / study tools), or pass handoff_to_craft_runner=true after explicit consent.",
                )
                .with_details(json!({
                    "code": "controller_conflict",
                    "controller": "host_agent",
                    "allow_nested_builder": false
                })));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_agent_blocks_nested_rebuild_without_handoff() {
        let p = ExecutionPolicy::host_agent();
        let err = p.guard_rebuild_start(false).unwrap_err();
        assert!(err.message.contains("controller_conflict"), "{}", err.message);
        assert!(p.guard_rebuild_start(true).is_ok());
        assert!(ExecutionPolicy::craft_runner().guard_rebuild_start(false).is_ok());
        assert!(ExecutionPolicy::default().guard_rebuild_start(false).is_ok());
    }
}
