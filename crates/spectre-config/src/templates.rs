// Author: Jeff
// Date: 2026-06-08
// Description: Declarative workflow template schema.
// Notes: Templates instantiate through undoable app commands, never by mutating state directly.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Named template reference exposed by browser, command palette, or empty states
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TemplateRef {
    pub name: String,
    pub kind: TemplateKind,
    #[serde(default)]
    pub args: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    Project,
    Track,
    Rack,
    Graph,
    Modulation,
}
