// Author: Jeff
// Date: 2026-06-08
// Description: Public entrypoint for Spectre workflow configuration.
// Notes: Config is declarative; it binds UI shape to typed app commands.

#![deny(unsafe_code)]

pub mod commands;
pub mod keybindings;
pub mod loader;
pub mod schema;
pub mod templates;
pub mod validate;

pub mod prelude {
    pub use crate::commands::CommandIntent;
    pub use crate::loader::{load_workflow_toml, resolve_workflow, WorkflowSource};
    pub use crate::schema::{Density, LensId, PanelId, WorkflowProfile, WorkflowVersion};
    pub use crate::validate::{validate_workflow, ConfigDiagnostic, ConfigReport};
}
