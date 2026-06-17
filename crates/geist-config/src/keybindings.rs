// Author: Jeff
// Date: 2026-06-08
// Description: Shortcut and controller binding schema for workflow profiles.
// Notes: Bindings resolve to command names; app command validation remains authoritative.

use std::collections::BTreeMap;

pub type KeyBindingMap = BTreeMap<String, String>;

// Keybindings are declarative labels, not direct code execution
pub fn validate_keybindings(bindings: &KeyBindingMap) -> bool {
    bindings
        .iter()
        .all(|(key, command)| !key.trim().is_empty() && !command.trim().is_empty())
}
