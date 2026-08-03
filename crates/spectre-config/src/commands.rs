// Author: Jeff
// Date: 2026-06-08
// Description: Declarative command alias schema for workflow profiles.
// Notes: Aliases name typed UI intents; they are not scripts or shell commands.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// A declarative command invocation selected by UI, shortcut, or palette alias
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandIntent {
    pub command: String,
    #[serde(default)]
    pub args: BTreeMap<String, String>,
}

impl CommandIntent {
    // Build a command with no arguments
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: BTreeMap::new(),
        }
    }

    // Reject blank and shell-like command names before app dispatch
    pub fn is_declarative(&self) -> bool {
        let trimmed = self.command.trim();
        !trimmed.is_empty()
            && !trimmed.contains('/')
            && !trimmed.contains('\\')
            && !trimmed.starts_with("shell:")
            && !trimmed.starts_with("exec:")
    }
}
