// Author: Jeff
// Date: 2026-06-15
// Description: Tactile-dark visual theme for the egui renderer.
// Notes: VCV/Phase-Plant blend: deep panels, soft depth, signal-colored ports.
//        Palette lives here so every widget reads one source of color truth.

use egui::{Color32, CornerRadius, Margin, Shadow, Stroke, Style, Vec2, Visuals};

// Surfaces, from deepest background to raised modules
pub const BG: Color32 = Color32::from_rgb(13, 14, 17);
pub const PANEL: Color32 = Color32::from_rgb(22, 24, 29);
pub const PANEL_RAISED: Color32 = Color32::from_rgb(30, 33, 40);
pub const PANEL_HOVER: Color32 = Color32::from_rgb(38, 42, 50);
// Inset wells: knob tracks, meter backgrounds, text fields
pub const INSET: Color32 = Color32::from_rgb(15, 16, 20);
pub const FAINT: Color32 = Color32::from_rgb(26, 28, 34);

// Borders and separators
pub const STROKE: Color32 = Color32::from_rgb(44, 48, 58);
pub const STROKE_STRONG: Color32 = Color32::from_rgb(64, 70, 84);

// Text
pub const TEXT: Color32 = Color32::from_rgb(228, 231, 238);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 147, 160);

// Brand accent (mint/teal) and its dimmed pressed form
pub const ACCENT: Color32 = Color32::from_rgb(90, 224, 180);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(54, 134, 108);

// Signal-type colors for ports, cables, and value arcs
pub const AUDIO: Color32 = Color32::from_rgb(74, 163, 255);
pub const CV: Color32 = Color32::from_rgb(255, 180, 84);
pub const NOTE: Color32 = Color32::from_rgb(199, 146, 234);
pub const CONTROL: Color32 = Color32::from_rgb(90, 224, 180);

// Level-meter gradient stops, low to clipping
pub const METER_LOW: Color32 = Color32::from_rgb(90, 224, 180);
pub const METER_MID: Color32 = Color32::from_rgb(240, 200, 90);
pub const METER_HIGH: Color32 = Color32::from_rgb(255, 92, 108);

// Corner radii: panels read softer than inline controls
pub const RADIUS_PANEL: u8 = 8;
pub const RADIUS_CONTROL: u8 = 6;

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

// The Geist tactile-dark theme; installs an egui Style for the whole context
pub struct GeistTheme;

impl GeistTheme {
    // Install the theme on an egui context; call once at startup
    pub fn apply(ctx: &egui::Context) {
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
    v.window_stroke = Stroke::new(1.0, STROKE);
    v.window_corner_radius = CornerRadius::same(RADIUS_PANEL);
    v.window_shadow = soft_shadow();
    v.menu_corner_radius = CornerRadius::same(RADIUS_PANEL);
    v.extreme_bg_color = INSET;
    v.faint_bg_color = FAINT;
    v.hyperlink_color = ACCENT;
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(90, 224, 180, 40);
    v.selection.stroke = Stroke::new(1.0, ACCENT);

    let control = CornerRadius::same(RADIUS_CONTROL);

    // Panels, labels, separators
    v.widgets.noninteractive.bg_fill = PANEL;
    v.widgets.noninteractive.weak_bg_fill = PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, STROKE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_MUTED);
    v.widgets.noninteractive.corner_radius = control;

    // Buttons and controls at rest
    v.widgets.inactive.bg_fill = PANEL_RAISED;
    v.widgets.inactive.weak_bg_fill = PANEL_RAISED;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, STROKE);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.corner_radius = control;
    v.widgets.inactive.expansion = 0.0;

    // Hover lifts the surface and brightens the border
    v.widgets.hovered.bg_fill = PANEL_HOVER;
    v.widgets.hovered.weak_bg_fill = PANEL_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, STROKE_STRONG);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.corner_radius = control;
    v.widgets.hovered.expansion = 1.0;

    // Pressed/engaged controls take the accent
    v.widgets.active.bg_fill = ACCENT_DIM;
    v.widgets.active.weak_bg_fill = ACCENT_DIM;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.active.corner_radius = control;
    v.widgets.active.expansion = 1.0;

    // Open combo/menu surfaces
    v.widgets.open.bg_fill = PANEL_RAISED;
    v.widgets.open.weak_bg_fill = PANEL_RAISED;
    v.widgets.open.bg_stroke = Stroke::new(1.0, STROKE_STRONG);
    v.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.open.corner_radius = control;

    v
}

// Soft drop shadow that gives panels their layered depth
fn soft_shadow() -> Shadow {
    Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(140),
    }
}

// Comfortable spacing: roomy controls, clear separation between modules
fn tune_spacing(style: &mut Style) {
    let s = &mut style.spacing;
    s.item_spacing = Vec2::new(8.0, 6.0);
    s.button_padding = Vec2::new(10.0, 6.0);
    s.window_margin = Margin::same(10);
    s.menu_margin = Margin::same(6);
    s.interact_size.y = 24.0;
}

// Legible type scale; family stays egui default until fonts are embedded
fn tune_text(style: &mut Style) {
    use egui::{FontFamily, FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
    ]
    .into();
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
        // Audio/Cv/Note are visually distinct; Control intentionally shares accent
        assert_ne!(SignalKind::Audio.color(), SignalKind::Cv.color());
        assert_ne!(SignalKind::Audio.color(), SignalKind::Note.color());
        assert_eq!(SignalKind::Control.color(), ACCENT);
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
}
