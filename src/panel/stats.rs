use super::Panel;
use super::theme::{self, palette};
use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Ui, Vec2};

const TOP_APPS: usize = 8;

pub fn show(panel: &mut Panel, ui: &mut Ui) {
    ui.add_space(theme::GAP);
    let s = panel.strings();
    let dark = panel.dark;

    if panel.stats.total == 0 {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(theme::muted(s.stats_no_data, dark));
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(theme::MARGIN);
            ui.vertical(|ui| {
                ui.set_max_width(ui.available_width() - theme::MARGIN);
                totals(panel, ui);
                ui.add_space(theme::GAP);
                by_app(panel, ui);
                ui.add_space(theme::GAP);
                by_day(panel, ui);
                ui.add_space(theme::GAP);
                by_hour(panel, ui);
                ui.add_space(theme::MARGIN);
            });
        });
    });
}

fn totals(panel: &Panel, ui: &mut Ui) {
    let s = panel.strings();
    theme::card(ui, panel.dark, |ui| {
        ui.horizontal(|ui| {
            theme::stat(
                ui,
                &panel.stats.total.to_string(),
                &format!("{} · {}", s.stats_last_days, s.stats_interruptions),
                panel.dark,
            );
            ui.add_space(28.0);
            theme::stat(
                ui,
                &panel.stats.restored.to_string(),
                s.summary_taken_back,
                panel.dark,
            );
            ui.add_space(28.0);
            let days = panel.stats.by_day.len().max(1) as f32;
            theme::stat(
                ui,
                &format!("{:.1}", panel.stats.total as f32 / days),
                s.stats_daily_average,
                panel.dark,
            );
        });
    });
}

fn by_app(panel: &Panel, ui: &mut Ui) {
    let s = panel.strings();
    let dark = panel.dark;
    let p = palette(dark);
    let worst = panel.stats.by_app.first().map(|(_, n)| *n).unwrap_or(1) as f32;

    theme::card(ui, dark, |ui| {
        ui.label(theme::heading(s.stats_by_app, dark));
        ui.add_space(8.0);
        let width = (ui.available_width() - 260.0).max(120.0);
        for (exe, count) in panel.stats.by_app.iter().take(TOP_APPS) {
            ui.horizontal(|ui| {
                ui.add_sized(
                    Vec2::new(170.0, 18.0),
                    egui::Label::new(RichText::new(exe).color(p.text).size(13.0)).truncate(),
                );
                theme::bar(ui, *count as f32 / worst, width, dark, p.taken_back);
                ui.label(RichText::new(count.to_string()).color(p.muted).size(12.5));
            });
        }
    });
}

fn by_day(panel: &Panel, ui: &mut Ui) {
    let s = panel.strings();
    let dark = panel.dark;
    let p = palette(dark);
    let peak = panel
        .stats
        .by_day
        .iter()
        .map(|(_, n)| *n)
        .max()
        .unwrap_or(1)
        .max(1) as f32;

    theme::card(ui, dark, |ui| {
        ui.label(theme::heading(s.stats_last_days, dark));
        ui.add_space(10.0);
        let count = panel.stats.by_day.len().max(1);
        let width = ui.available_width();
        let slot = width / count as f32;
        let bar_width = (slot - 6.0).clamp(6.0, 28.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 96.0), Sense::hover());
        let painter = ui.painter();
        let mut hover_text = None;

        for (index, (date, value)) in panel.stats.by_day.iter().enumerate() {
            let height = (*value as f32 / peak) * (rect.height() - 18.0);
            let x = rect.min.x + slot * index as f32 + (slot - bar_width) / 2.0;
            let bar = egui::Rect::from_min_size(
                egui::pos2(x, rect.max.y - 16.0 - height.max(2.0)),
                Vec2::new(bar_width, height.max(2.0)),
            );
            let colour = if *value == 0 { p.line } else { p.accent };
            painter.rect_filled(bar, CornerRadius::same(3), colour);
            if let Some(pointer) = response.hover_pos()
                && pointer.x >= x - 3.0
                && pointer.x <= x + bar_width + 3.0
            {
                hover_text = Some(format!(
                    "{} · {value} {}",
                    date.format("%d.%m"),
                    s.stats_interruptions
                ));
            }
            if index == 0 || index + 1 == count {
                painter.text(
                    egui::pos2(x + bar_width / 2.0, rect.max.y - 12.0),
                    egui::Align2::CENTER_TOP,
                    date.format("%d.%m").to_string(),
                    egui::FontId::proportional(11.0),
                    p.muted,
                );
            }
        }
        if let Some(text) = hover_text {
            response.on_hover_text(text);
        }
    });
}

fn by_hour(panel: &Panel, ui: &mut Ui) {
    let s = panel.strings();
    let dark = panel.dark;
    let p = palette(dark);
    let peak = panel
        .stats
        .by_hour
        .iter()
        .copied()
        .max()
        .unwrap_or(1)
        .max(1) as f32;

    theme::card(ui, dark, |ui| {
        ui.label(theme::heading(s.stats_by_hour, dark));
        ui.add_space(10.0);
        let width = ui.available_width();
        let slot = width / 24.0;
        let bar_width = (slot - 5.0).clamp(5.0, 26.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 84.0), Sense::hover());
        let painter = ui.painter();
        let mut hover_text = None;

        for (hour, value) in panel.stats.by_hour.iter().enumerate() {
            let height = (*value as f32 / peak) * (rect.height() - 18.0);
            let x = rect.min.x + slot * hour as f32 + (slot - bar_width) / 2.0;
            let bar = egui::Rect::from_min_size(
                egui::pos2(x, rect.max.y - 16.0 - height.max(2.0)),
                Vec2::new(bar_width, height.max(2.0)),
            );
            let colour = if *value == 0 {
                p.line
            } else {
                blend(p.accent, p.taken_back, *value as f32 / peak)
            };
            painter.rect_filled(bar, CornerRadius::same(3), colour);
            if let Some(pointer) = response.hover_pos()
                && pointer.x >= x - 3.0
                && pointer.x <= x + bar_width + 3.0
            {
                hover_text = Some(format!("{hour:02}:00 · {value} {}", s.stats_interruptions));
            }
            if hour % 6 == 0 {
                painter.text(
                    egui::pos2(x + bar_width / 2.0, rect.max.y - 12.0),
                    egui::Align2::CENTER_TOP,
                    format!("{hour:02}"),
                    egui::FontId::proportional(11.0),
                    p.muted,
                );
            }
        }
        if let Some(text) = hover_text {
            response.on_hover_text(text);
        }
    });
}

fn blend(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    Color32::from_rgb(
        mix(from.r(), to.r()),
        mix(from.g(), to.g()),
        mix(from.b(), to.b()),
    )
}
