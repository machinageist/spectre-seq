// Author: Jeff
// Date: 2026-06-08
// Description: Public entrypoint for Geist UI state, commands, renderer, views, and widgets.
// Notes: UI renders state and emits commands; it does not own project or audio truth.

#![deny(unsafe_code)]

pub mod app;
pub mod commands;
pub mod egui_renderer;
pub mod model;
pub mod renderer;
pub mod shell;
pub mod state;
pub mod theme;
pub mod views;
pub mod widgets;

pub mod prelude {
    pub use crate::app::App;
    pub use crate::commands::{command_from_intent, UICommand, UICommandError};
    pub use crate::model::SessionModel;
    pub use crate::shell::{draw_studio, StudioResponse};
    pub use crate::state::{SelectedObject, UIState, UIStateError};
    pub use crate::theme::{GeistTheme, SignalKind};
    pub use crate::widgets::{Fader, KeyEvent, Keyboard, Knob, Meter, Taper};
}
