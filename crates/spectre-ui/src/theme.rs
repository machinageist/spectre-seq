// Author: Jeff
// Date: 2026-06-15
// Description: Hyprland-inspired visual theme for the egui renderer.
// Notes: Dracula chrome and focus states with DAW-specific signal colors.
//        Palette lives here so every widget reads one source of color truth.

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Margin, Shadow, Stroke, Style,
    Vec2, Visuals,
};
use std::path::Path;
use std::sync::Arc;

// Dracula surfaces, from compositor-dark background to raised modules
pub const BG: Color32 = Color32::from_rgb(25, 26, 33);
pub const PANEL: Color32 = Color32::from_rgb(40, 42, 54);
pub const PANEL_RAISED: Color32 = Color32::from_rgb(52, 55, 70);
pub const PANEL_HOVER: Color32 = Color32::from_rgb(68, 71, 90);
// Inset wells: knob tracks, meter backgrounds, text fields
pub const INSET: Color32 = Color32::from_rgb(33, 34, 44);
pub const FAINT: Color32 = Color32::from_rgb(48, 50, 65);

// Borders and separators
pub const STROKE: Color32 = Color32::from_rgb(68, 71, 90);
pub const STROKE_STRONG: Color32 = Color32::from_rgb(189, 147, 249);

// Text
pub const TEXT: Color32 = Color32::from_rgb(248, 248, 242);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(169, 168, 182);

// Hyprland active-border gradient endpoints and dimmed pressed form
pub const ACCENT: Color32 = Color32::from_rgb(189, 147, 249);
pub const ACCENT_ALT: Color32 = Color32::from_rgb(255, 121, 198);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(92, 70, 119);

// Distinct Dracula signal colors preserve fast DAW recognition
pub const AUDIO: Color32 = Color32::from_rgb(139, 233, 253);
pub const CV: Color32 = Color32::from_rgb(255, 184, 108);
pub const NOTE: Color32 = Color32::from_rgb(80, 250, 123);
pub const CONTROL: Color32 = ACCENT_ALT;

// Level-meter gradient stops, low to clipping
pub const METER_LOW: Color32 = Color32::from_rgb(80, 250, 123);
pub const METER_MID: Color32 = Color32::from_rgb(255, 184, 108);
pub const METER_HIGH: Color32 = Color32::from_rgb(255, 85, 85);

// Match Hyprland's 12px windows and Wofi's 8px inline controls
pub const RADIUS_PANEL: u8 = 12;
pub const RADIUS_CONTROL: u8 = 8;

// Signal type a port or cable carries; drives its color across every view
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SignalKind {
    Audio,
    Cv,
    Note,
    Control,
}

impl SignalKind {
    // The canonical color for this signal type
    pub fn color(self) -> Color32 {
        match self {
            SignalKind::Audio => AUDIO,
            SignalKind::Cv => CV,
            SignalKind::Note => NOTE,
            SignalKind::Control => CONTROL,
        }
    }
}

// Color for a 0..=1 meter fill, blending low -> mid -> high toward clipping
pub fn meter_color(fraction: f32) -> Color32 {
    let f = fraction.clamp(0.0, 1.0);
    if f < 0.75 {
        METER_LOW
    } else if f < 0.92 {
        METER_MID
    } else {
        METER_HIGH
    }
}

// The Spectre tactile-dark theme; installs an egui Style for the whole context
pub struct SpectreTheme;

impl SpectreTheme {
    // Install the theme on an egui context; call once at startup
    pub fn apply(ctx: &egui::Context) {
        install_hyprland_font(ctx);
        let mut style = (*ctx.global_style()).clone();
        style.visuals = visuals();
        tune_spacing(&mut style);
        tune_text(&mut style);
        ctx.set_global_style(style);
    }
}

// Build the dark visuals: surfaces, widget states, selection, and depth
fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.dark_mode = true;
    v.override_text_color = Some(TEXT);
    v.panel_fill = PANEL;
    v.window_fill = PANEL_RAISED;
    v.window_stroke = Stroke::new(1.0_f32, STROKE);
    v.window_corner_radius = CornerRadius::same(RADIUS_PANEL);
    v.window_shadow = soft_shadow();
    v.menu_corner_radius = CornerRadius::same(RADIUS_PANEL);
    v.extreme_bg_color = INSET;
    v.faint_bg_color = FAINT;
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(189, 147, 249, 48);
    v.selection.stroke = Stroke::new(1.0_f32, ACCENT);

    let control = CornerRadius::same(RADIUS_CONTROL);

    // Panels, labels, separators
    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.weak_bg_fill = PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, STROKE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_MUTED);
    v.widgets.noninteractive.corner_radius = control;

    // Buttons and controls at rest
    v.widgets.inactive.bg_fill = PANEL_RAISED;
    v.widgets.inactive.weak_bg_fill = PANEL_RAISED;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, STROKE);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    v.widgets.inactive.corner_radius = control;
    v.widgets.inactive.expansion = 0.0;

    // Hover lifts the surface and brightens the border
    v.widgets.hovered.bg_fill = PANEL_HOVER;
    v.widgets.hovered.weak_bg_fill = PANEL_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, STROKE_STRONG);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, TEXT);
    v.widgets.hovered.corner_radius = control;
    v.widgets.hovered.expansion = 1.0;

    // Pressed/engaged controls take the accent
    v.widgets.active.bg_fill = ACCENT_DIM;
    v.widgets.active.weak_bg_fill = ACCENT_DIM;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0_f32, TEXT);
    v.widgets.active.corner_radius = control;
    v.widgets.active.expansion = 1.0;

    // Open combo/menu surfaces
    v.widgets.open.bg_fill = PANEL_RAISED;
    v.widgets.open.weak_bg_fill = PANEL_RAISED;
    v.widgets.open.bg_stroke = Stroke::new(1.0_f32, STROKE_STRONG);
    v.widgets.open.fg_stroke = Stroke::new(1.0_f32, TEXT);
    v.widgets.open.corner_radius = control;

    v
}

// Soft drop shadow that gives panels their layered depth
fn soft_shadow() -> Shadow {
    Shadow {
        offset: [0, 4],
        blur: 20,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(189, 147, 249, 64),
    }
}

// Compact spacing follows the compositor's four-pixel gap rhythm
fn tune_spacing(style: &mut Style) {
    let s = &mut style.spacing;
    s.item_spacing = Vec2::new(6.0, 4.0);
    s.button_padding = Vec2::new(10.0, 6.0);
    s.window_margin = Margin::same(10);
    s.menu_margin = Margin::same(6);
    s.interact_size.y = 24.0;
}

// JetBrains Mono drives the desktop; fall back to egui monospace elsewhere
fn tune_text(style: &mut Style) {
    use egui::{FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(18.0, FontFamily::Monospace)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(11.0, FontFamily::Monospace)),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();
}

// Load the user's desktop font where installed without making it a build input
fn install_hyprland_font(ctx: &egui::Context) {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf",
        "/usr/share/fonts/truetype/jetbrains-mono/JetBrainsMono-Regular.ttf",
        "/Library/Fonts/JetBrainsMonoNerdFont-Regular.ttf",
    ];
    let Some(bytes) = CANDIDATES
        .iter()
        .find_map(|path| std::fs::read(Path::new(path)).ok())
    else {
        return;
    };
    let mut fonts = FontDefinitions::default();
    let name = "JetBrainsMono Nerd Font".to_string();
    fonts
        .font_data
        .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
    if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
        family.insert(0, name);
    }
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_kinds_have_distinct_colors() {
        let kinds = [
            SignalKind::Audio,
            SignalKind::Cv,
            SignalKind::Note,
            SignalKind::Control,
        ];
        // Semantic signals remain distinct from each other and from chrome focus
        assert_ne!(SignalKind::Audio.color(), SignalKind::Cv.color());
        assert_ne!(SignalKind::Audio.color(), SignalKind::Note.color());
        assert_eq!(SignalKind::Control.color(), ACCENT_ALT);
        assert_ne!(SignalKind::Control.color(), ACCENT);
        assert_eq!(kinds.len(), 4);
    }

    #[test]
    fn meter_color_escalates_toward_clipping() {
        assert_eq!(meter_color(0.10), METER_LOW);
        assert_eq!(meter_color(0.80), METER_MID);
        assert_eq!(meter_color(0.99), METER_HIGH);
        // Clamps out-of-range input
        assert_eq!(meter_color(-1.0), METER_LOW);
        assert_eq!(meter_color(2.0), METER_HIGH);
    }

    #[test]
    fn hyprland_theme_applies_dracula_chrome() {
        let ctx = egui::Context::default();
        SpectreTheme::apply(&ctx);
        let style = ctx.global_style();
        assert_eq!(style.visuals.panel_fill, PANEL);
        assert_eq!(style.visuals.selection.stroke.color, ACCENT);
        assert_eq!(style.visuals.window_corner_radius, CornerRadius::same(12));
        assert_eq!(style.spacing.item_spacing, Vec2::new(6.0, 4.0));
        assert_eq!(
            style.text_styles[&egui::TextStyle::Body].family,
            FontFamily::Monospace
        );
    }
}
