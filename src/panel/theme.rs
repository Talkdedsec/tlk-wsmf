//! One place for every colour and measurement in the panel, so the four tabs cannot
//! drift apart. Follows the Windows light/dark setting rather than inventing a look.

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Response, RichText,
    Sense, Stroke, TextStyle, Ui, Vec2,
};
use std::sync::Arc;

pub const MARGIN: f32 = 18.0;
pub const GAP: f32 = 10.0;
pub const TOP_BAR_HEIGHT: f32 = 62.0;
pub const TAB_BAR_HEIGHT: f32 = 38.0;
pub const ROW_HEIGHT: f32 = 30.0;

pub struct Palette {
    pub ground: Color32,
    pub raised: Color32,
    /// Checkboxes, radios and slider tracks, which sit on top of `raised`.
    pub control: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub line: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub taken_back: Color32,
    pub gave_up: Color32,
    pub seen: Color32,
}

pub const DARK: Palette = Palette {
    ground: Color32::from_rgb(0x16, 0x18, 0x1D),
    raised: Color32::from_rgb(0x1D, 0x20, 0x27),
    control: Color32::from_rgb(0x2C, 0x31, 0x3B),
    text: Color32::from_rgb(0xE6, 0xE8, 0xEC),
    muted: Color32::from_rgb(0x8B, 0x91, 0x9E),
    line: Color32::from_rgb(0x2A, 0x2E, 0x37),
    accent: Color32::from_rgb(0x5A, 0x9C, 0xF8),
    accent_soft: Color32::from_rgb(0x23, 0x31, 0x47),
    taken_back: Color32::from_rgb(0xE5, 0x7A, 0x6E),
    gave_up: Color32::from_rgb(0xD2, 0xA5, 0x22),
    seen: Color32::from_rgb(0x8B, 0x91, 0x9E),
};

pub const LIGHT: Palette = Palette {
    ground: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    raised: Color32::from_rgb(0xF5, 0xF6, 0xF8),
    control: Color32::from_rgb(0xFF, 0xFF, 0xFF),
    text: Color32::from_rgb(0x1F, 0x23, 0x28),
    muted: Color32::from_rgb(0x6B, 0x72, 0x80),
    line: Color32::from_rgb(0xE3, 0xE6, 0xEA),
    accent: Color32::from_rgb(0x21, 0x5A, 0xD6),
    accent_soft: Color32::from_rgb(0xE8, 0xEF, 0xFD),
    taken_back: Color32::from_rgb(0xC0, 0x39, 0x2B),
    gave_up: Color32::from_rgb(0x8A, 0x62, 0x00),
    seen: Color32::from_rgb(0x6B, 0x72, 0x80),
};

pub fn palette(dark: bool) -> &'static Palette {
    if dark { &DARK } else { &LIGHT }
}

pub fn apply(ctx: &egui::Context, dark: bool) {
    install_system_font(ctx);

    let p = palette(dark);
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = p.ground;
    visuals.window_fill = p.ground;
    visuals.extreme_bg_color = p.raised;
    visuals.faint_bg_color = p.raised;
    visuals.override_text_color = Some(p.text);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.line);
    visuals.widgets.inactive.bg_fill = p.control;
    visuals.widgets.inactive.weak_bg_fill = p.control;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, p.line);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.4, p.text);
    visuals.widgets.hovered.bg_fill = p.control;
    visuals.widgets.hovered.weak_bg_fill = p.control;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.6, p.text);
    visuals.widgets.active.bg_fill = p.accent_soft;
    visuals.widgets.active.weak_bg_fill = p.accent_soft;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    visuals.selection.bg_fill = p.accent;
    visuals.selection.stroke = Stroke::new(1.0, p.accent);
    visuals.window_corner_radius = CornerRadius::same(8);
    visuals.menu_corner_radius = CornerRadius::same(8);
    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(GAP, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.spacing.interact_size.y = 26.0;
        // Without this a slider inside a grid collapses to a few pixels wide.
        style.spacing.slider_width = 210.0;
        style.spacing.icon_width = 16.0;
        style.spacing.icon_width_inner = 9.0;
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(17.0, FontFamily::Proportional),
        );
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(13.5, FontFamily::Proportional));
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(13.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
    });
}

/// Segoe UI if it is there, which on Windows it is. Falls back to the bundled font
/// rather than failing, and the bundled font covers Turkish either way.
fn install_system_font(ctx: &egui::Context) {
    let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") else {
        return;
    };
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("segoe".into(), Arc::new(FontData::from_owned(bytes)));
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "segoe".into());
    ctx.set_fonts(fonts);
}

pub fn title(text: &str, dark: bool) -> RichText {
    RichText::new(text)
        .size(16.0)
        .strong()
        .color(palette(dark).text)
}

pub fn muted(text: &str, dark: bool) -> RichText {
    RichText::new(text).size(12.0).color(palette(dark).muted)
}

pub fn heading(text: &str, dark: bool) -> RichText {
    RichText::new(text)
        .size(13.0)
        .strong()
        .color(palette(dark).muted)
}

/// The mode selector in the title bar: three pills, one of them filled.
pub fn pill(ui: &mut Ui, text: &str, selected: bool, dark: bool) -> Response {
    let p = palette(dark);
    let padding = Vec2::new(14.0, 6.0);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(13.0, FontFamily::Proportional),
        if selected { p.accent } else { p.muted },
    );
    let size = galley.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        p.accent_soft
    } else if hovered {
        p.raised
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::same(14), fill);
    if selected {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(14),
            Stroke::new(1.0, p.accent),
            egui::StrokeKind::Inside,
        );
    }
    ui.painter()
        .galley(rect.min + padding, galley, Color32::PLACEHOLDER);
    response
}

/// A tab: label with an underline when active, nothing else.
pub fn tab(ui: &mut Ui, text: &str, selected: bool, dark: bool) -> Response {
    let p = palette(dark);
    let padding = Vec2::new(12.0, 9.0);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(13.5, FontFamily::Proportional),
        if selected { p.text } else { p.muted },
    );
    let size = Vec2::new(galley.size().x + padding.x * 2.0, TAB_BAR_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if response.hovered() && !selected {
        ui.painter().rect_filled(
            rect.shrink2(Vec2::new(4.0, 6.0)),
            CornerRadius::same(6),
            p.raised,
        );
    }
    ui.painter().galley(
        egui::pos2(rect.min.x + padding.x, rect.min.y + padding.y),
        galley,
        Color32::PLACEHOLDER,
    );
    if selected {
        let y = rect.max.y - 2.0;
        ui.painter().line_segment(
            [
                egui::pos2(rect.min.x + 8.0, y),
                egui::pos2(rect.max.x - 8.0, y),
            ],
            Stroke::new(2.0, p.accent),
        );
    }
    response
}

/// A number with a word under it. Used for the counts above the activity list.
pub fn stat(ui: &mut Ui, value: &str, label: &str, dark: bool) {
    stat_sized(ui, value, label, dark, 22.0);
}

/// The same, for a value that is a name rather than a number and would shout at 22 px.
pub fn stat_text(ui: &mut Ui, value: &str, label: &str, dark: bool) {
    stat_sized(ui, value, label, dark, 15.0);
}

fn stat_sized(ui: &mut Ui, value: &str, label: &str, dark: bool, size: f32) {
    let p = palette(dark);
    ui.vertical(|ui| {
        ui.add_space(26.0 - size.min(22.0));
        ui.label(
            RichText::new(value)
                .size(size)
                .strong()
                .color(p.text)
                .line_height(Some(26.0)),
        );
        ui.label(RichText::new(label).size(11.5).color(p.muted));
    });
}

pub fn card<R>(ui: &mut Ui, dark: bool, add: impl FnOnce(&mut Ui) -> R) -> R {
    let p = palette(dark);
    let width = ui.available_width();
    egui::Frame::new()
        .fill(p.raised)
        .corner_radius(CornerRadius::same(8))
        .stroke(Stroke::new(1.0, p.line))
        .inner_margin(egui::Margin::symmetric(14, 12))
        .show(ui, |ui| {
            ui.set_width(width - 28.0);
            add(ui)
        })
        .inner
}

/// A horizontal bar, drawn rather than plotted: no chart library for four screens
/// of rectangles.
pub fn bar(ui: &mut Ui, fraction: f32, width: f32, dark: bool, colour: Color32) {
    let p = palette(dark);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 8.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), p.line);
    let filled = rect.width() * fraction.clamp(0.0, 1.0);
    if filled > 0.5 {
        let mut filled_rect = rect;
        filled_rect.set_width(filled);
        ui.painter()
            .rect_filled(filled_rect, CornerRadius::same(4), colour);
    }
}
