use super::Panel;
use super::theme::{self, palette};
use crate::config::Mode;
use crate::i18n::Language;
use crate::system;
use eframe::egui::{self, RichText, Ui};

pub fn show(panel: &mut Panel, ui: &mut Ui) {
    ui.add_space(theme::GAP);
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Settings read better in a column than stretched across a wide window, so
        // the column is capped and centred rather than pinned to the left edge.
        const COLUMN: f32 = 820.0;
        ui.horizontal(|ui| {
            let room = ui.available_width() - theme::MARGIN * 2.0;
            ui.add_space(theme::MARGIN + ((room - COLUMN) / 2.0).max(0.0));
            ui.vertical(|ui| {
                ui.set_max_width(room.min(COLUMN));
                mode(panel, ui);
                ui.add_space(theme::GAP);
                timing(panel, ui);
                ui.add_space(theme::GAP);
                behaviour(panel, ui);
                ui.add_space(theme::GAP);
                windows_lock(panel, ui);
                ui.add_space(theme::MARGIN);
            });
        });
    });
}

fn mode(panel: &mut Panel, ui: &mut Ui) {
    let s = panel.strings();
    let dark = panel.dark;
    let p = palette(dark);
    let mut chosen = panel.cfg.mode;

    theme::card(ui, dark, |ui| {
        ui.label(theme::heading(s.settings_mode, dark));
        ui.add_space(6.0);
        for candidate in Mode::ALL {
            let row = ui
                .horizontal(|ui| {
                    let _ = ui.radio(chosen == candidate, "");
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(candidate.label(s))
                                .color(p.text)
                                .size(13.5)
                                .strong(),
                        );
                        ui.label(theme::muted(candidate.hint(s), dark));
                    });
                })
                .response;
            if row.interact(egui::Sense::click()).clicked() {
                chosen = candidate;
            }
            ui.add_space(4.0);
        }
    });

    if chosen != panel.cfg.mode {
        panel.cfg.mode = chosen;
        panel.save();
    }
}

fn timing(panel: &mut Panel, ui: &mut Ui) {
    let s = panel.strings();
    let dark = panel.dark;
    let before = panel.cfg.clone();

    theme::card(ui, dark, |ui| {
        ui.label(theme::heading(s.settings_timing, dark));
        ui.add_space(6.0);
        egui::Grid::new("timing")
            .num_columns(2)
            .spacing([16.0, 10.0])
            .show(ui, |ui| {
                ui.label(s.settings_typing_window);
                ui.add(
                    egui::Slider::new(&mut panel.cfg.typing_window_ms, 200..=5_000)
                        .suffix(format!(" {}", s.ms_suffix))
                        .step_by(50.0),
                );
                ui.end_row();

                ui.label(s.settings_click_grace);
                ui.add(
                    egui::Slider::new(&mut panel.cfg.click_grace_ms, 50..=2_000)
                        .suffix(format!(" {}", s.ms_suffix))
                        .step_by(25.0),
                );
                ui.end_row();

                ui.label(s.settings_max_restores);
                ui.add(
                    egui::Slider::new(&mut panel.cfg.max_restores, 1..=10)
                        .suffix(format!(" {}", s.times_suffix)),
                );
                ui.end_row();

                ui.label(s.settings_restore_window);
                ui.add(
                    egui::Slider::new(&mut panel.cfg.restore_window_secs, 2..=120)
                        .suffix(format!(" {}", s.seconds_suffix)),
                );
                ui.end_row();
            });
    });

    if panel.cfg != before {
        panel.save();
    }
}

fn behaviour(panel: &mut Panel, ui: &mut Ui) {
    let s = panel.strings();
    let dark = panel.dark;
    let before = panel.cfg.clone();
    let mut autostart_changed = None;

    theme::card(ui, dark, |ui| {
        ui.label(theme::heading(s.settings_behaviour, dark));
        ui.add_space(6.0);
        ui.checkbox(&mut panel.cfg.flash_thief, s.settings_flash);

        let mut autostart = system::autostart_enabled();
        if ui.checkbox(&mut autostart, s.settings_autostart).changed() {
            autostart_changed = Some(autostart);
        }

        ui.checkbox(&mut panel.cfg.log_to_file, s.settings_log_to_file);
        ui.checkbox(
            &mut panel.cfg.record_everything,
            s.settings_record_everything,
        );

        ui.add_space(10.0);
        ui.label(theme::heading(s.settings_language, dark));
        ui.horizontal(|ui| {
            for (language, label) in [
                (Language::System, s.settings_language_system),
                (Language::English, "English"),
                (Language::Turkish, "Türkçe"),
            ] {
                if theme::pill(ui, label, panel.cfg.language == language, dark).clicked() {
                    panel.cfg.language = language;
                }
            }
        });
    });

    if let Some(on) = autostart_changed
        && system::set_autostart(on)
    {
        panel.cfg.start_with_windows = on;
    }
    if panel.cfg != before {
        panel.save();
    }
}

fn windows_lock(panel: &mut Panel, ui: &mut Ui) {
    let s = panel.strings();
    let dark = panel.dark;
    let p = palette(dark);
    let current = system::foreground_lock_timeout().unwrap_or(0);
    let on = current >= panel.cfg.foreground_lock_timeout_ms;

    theme::card(ui, dark, |ui| {
        ui.label(theme::heading(s.settings_windows_lock, dark));
        ui.add_space(4.0);
        ui.label(theme::muted(s.settings_windows_lock_hint, dark));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if on {
                    s.settings_windows_lock_on
                } else {
                    s.settings_windows_lock_off
                })
                .color(if on { p.accent } else { p.gave_up })
                .strong(),
            );
            ui.label(theme::muted(&format!("({current} {})", s.ms_suffix), dark));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if on {
                    s.settings_turn_off
                } else {
                    s.settings_turn_on
                };
                if ui.button(label).clicked() {
                    let target = if on {
                        0
                    } else {
                        panel.cfg.foreground_lock_timeout_ms
                    };
                    system::set_foreground_lock_timeout(target);
                }
            });
        });
    });
}
