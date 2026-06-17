// Author: Jeff
// Date: 2026-06-08
// Description: Renderer-neutral widget models built from configured workflow surfaces.
// Notes: Widgets are labeled command affordances; they do not mutate project/audio truth directly.

pub mod cable;
pub mod fader;
pub mod keyboard;
pub mod knob;
pub mod meter;
pub mod piano;
pub mod waveform;

// Tactile control widgets, re-exported for ergonomic use
pub use fader::Fader;
pub use keyboard::{KeyEvent, Keyboard};
pub use knob::{Knob, Taper};
pub use meter::Meter;

use crate::renderer::{PanelPlacement, PanelSlot};
use crate::views::{ActionChip, WorkspaceSurface};
use geist_config::schema::{Density, PanelId};

// Complete widget plan for a workspace frame
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceWidgets {
    pub workflow_id: String,
    pub density: Density,
    pub lens_tabs: Vec<TabWidget>,
    pub panels: Vec<PanelWidget>,
    pub main: MainWidget,
    pub context_shelf: ContextShelfWidget,
    pub command_palette: CommandPaletteWidget,
}

// Visible lens switcher item
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabWidget {
    pub label: String,
    pub active: bool,
    pub command: String,
}

// Configured persistent panel placeholder
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PanelWidget {
    pub slot: PanelSlot,
    pub panel: PanelId,
    pub title: &'static str,
}

// Active lens content placeholder with visible empty-state actions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainWidget {
    pub title: String,
    pub purpose: &'static str,
    pub empty_actions: Vec<ButtonWidget>,
}

// Context shelf action strip for the selected object
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextShelfWidget {
    pub actions: Vec<ButtonWidget>,
}

// Command palette visibility and searchable aliases/actions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPaletteWidget {
    pub open: bool,
}

// Labeled action affordance. Renderers may draw as button, chip, menu row, or hardware control mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonWidget {
    pub label: String,
    pub command: String,
}

// Convert configured surfaces into concrete widget inputs for egui/wgpu renderers
pub fn workspace_widgets_from_surface(surface: &WorkspaceSurface) -> WorkspaceWidgets {
    WorkspaceWidgets {
        workflow_id: surface.workflow_id.clone(),
        density: surface.density,
        lens_tabs: surface
            .lens_tabs
            .iter()
            .map(|label| tab_from_label(label))
            .collect(),
        panels: surface.panels.iter().map(panel_widget).collect(),
        main: MainWidget {
            title: surface.main.title.clone(),
            purpose: surface.main.purpose,
            empty_actions: button_widgets(&surface.main.empty_actions),
        },
        context_shelf: ContextShelfWidget {
            actions: button_widgets(&surface.context_actions),
        },
        command_palette: CommandPaletteWidget {
            open: surface.command_palette_open,
        },
    }
}

fn tab_from_label(label: &str) -> TabWidget {
    let active = label.ends_with('*');
    let clean = label.trim_end_matches('*').to_string();
    TabWidget {
        command: format!("switch_lens:{}", clean.to_lowercase()),
        label: clean,
        active,
    }
}

fn panel_widget(placement: &PanelPlacement) -> PanelWidget {
    PanelWidget {
        slot: placement.slot,
        panel: placement.panel,
        title: panel_title(placement.panel),
    }
}

fn panel_title(panel: PanelId) -> &'static str {
    match panel {
        PanelId::Browser => "Browser",
        PanelId::ContextShelf => "Context Shelf",
        PanelId::ModulationOverview => "Modulation Overview",
        PanelId::Meters => "Meters",
        PanelId::MacroStrip => "Macro Strip",
    }
}

fn button_widgets(actions: &[ActionChip]) -> Vec<ButtonWidget> {
    actions
        .iter()
        .map(|action| ButtonWidget {
            label: action.label.clone(),
            command: action.command.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::frame_from_state;
    use crate::state::{SelectedObject, UIState};
    use crate::views::workspace_surface_from_frame;
    use geist_config::schema::{ContextShelfConfig, LensId, PanelId, WorkflowProfile};

    #[test]
    fn widgets_use_configured_tabs_panels_and_actions() {
        let mut workflow = WorkflowProfile::default_profile();
        workflow.profile_id = "modular".to_string();
        workflow.startup_lens = LensId::Build;
        workflow.lenses.visible = vec![LensId::Build, LensId::Shape];
        workflow.layout.bottom_panel = Some(PanelId::ModulationOverview);
        workflow.graph.empty_actions = vec!["add_source".to_string(), "open_browser".to_string()];
        workflow.context_shelf.insert(
            "track".to_string(),
            ContextShelfConfig {
                actions: vec!["add_effect".to_string()],
            },
        );
        let mut state = UIState::from_workflow(workflow);
        state.select_object(SelectedObject::Track("track-1".to_string()));
        let frame = frame_from_state(&state);
        let surface = workspace_surface_from_frame(&frame);

        let widgets = workspace_widgets_from_surface(&surface);

        assert_eq!(widgets.workflow_id, "modular");
        assert_eq!(widgets.lens_tabs[0].label, "Build");
        assert!(widgets.lens_tabs[0].active);
        assert_eq!(widgets.lens_tabs[0].command, "switch_lens:build");
        assert!(widgets.panels.iter().any(|panel| {
            panel.slot == PanelSlot::Bottom && panel.title == "Modulation Overview"
        }));
        assert_eq!(widgets.main.empty_actions[0].label, "Add Source");
        assert_eq!(widgets.context_shelf.actions[0].command, "add_effect");
    }
}
