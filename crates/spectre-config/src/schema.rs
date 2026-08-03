// Author: Jeff
// Date: 2026-06-08
// Description: Versioned schema for creator-authored workflow profiles.
// Notes: The schema describes UI shape; it does not own project or audio truth.

use crate::commands::CommandIntent;
use crate::keybindings::KeyBindingMap;
use crate::templates::TemplateRef;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type WorkflowVersion = u16;

pub const CURRENT_WORKFLOW_VERSION: WorkflowVersion = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowProfile {
    pub version: WorkflowVersion,
    pub profile_id: String,
    pub display_name: String,
    pub startup_lens: LensId,
    #[serde(default)]
    pub lenses: LensConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub graph: GraphConfig,
    #[serde(default)]
    pub context_shelf: BTreeMap<String, ContextShelfConfig>,
    #[serde(default)]
    pub commands: CommandAliasConfig,
    #[serde(default)]
    pub keybindings: KeyBindingMap,
    #[serde(default)]
    pub templates: Vec<TemplateRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LensConfig {
    pub order: Vec<LensId>,
    pub visible: Vec<LensId>,
}

impl Default for LensConfig {
    fn default() -> Self {
        Self {
            order: LensId::default_order(),
            visible: LensId::default_order(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub density: Density,
    pub left_panel: Option<PanelId>,
    pub right_panel: Option<PanelId>,
    pub bottom_panel: Option<PanelId>,
    pub transport: PanelEdge,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            density: Density::Normal,
            left_panel: Some(PanelId::Browser),
            right_panel: Some(PanelId::ContextShelf),
            bottom_panel: None,
            transport: PanelEdge::Top,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GraphConfig {
    pub cable_labels: CableLabelPolicy,
    pub show_latency: bool,
    pub show_route_health: bool,
    #[serde(default)]
    pub empty_actions: Vec<String>,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            cable_labels: CableLabelPolicy::OnHover,
            show_latency: true,
            show_route_health: true,
            empty_actions: vec![
                "add_source".to_string(),
                "add_processor".to_string(),
                "add_modulator".to_string(),
                "open_browser".to_string(),
            ],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct ContextShelfConfig {
    #[serde(default)]
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct CommandAliasConfig {
    #[serde(default)]
    pub aliases: BTreeMap<String, CommandIntent>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LensId {
    Arrange,
    Build,
    Shape,
    Mix,
    Browser,
    Modulation,
}

impl LensId {
    pub fn default_order() -> Vec<Self> {
        vec![
            Self::Arrange,
            Self::Build,
            Self::Shape,
            Self::Mix,
            Self::Browser,
            Self::Modulation,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelId {
    Browser,
    ContextShelf,
    ModulationOverview,
    Meters,
    MacroStrip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelEdge {
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Compact,
    Normal,
    Spacious,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CableLabelPolicy {
    Always,
    OnHover,
    Hidden,
}

impl WorkflowProfile {
    // Strong default profile keeps Spectre usable without user config
    pub fn default_profile() -> Self {
        Self {
            version: CURRENT_WORKFLOW_VERSION,
            profile_id: "default".to_string(),
            display_name: "Default".to_string(),
            startup_lens: LensId::Arrange,
            lenses: LensConfig::default(),
            layout: LayoutConfig::default(),
            graph: GraphConfig::default(),
            context_shelf: BTreeMap::new(),
            commands: CommandAliasConfig::default(),
            keybindings: BTreeMap::from([
                ("Cmd+K".to_string(), "open_command_palette".to_string()),
                ("G".to_string(), "switch_lens:build".to_string()),
                ("A".to_string(), "switch_lens:arrange".to_string()),
            ]),
            templates: Vec::new(),
        }
    }
}
