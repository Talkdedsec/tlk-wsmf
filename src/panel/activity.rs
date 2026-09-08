use super::theme::{self, palette};
use super::{Filter, Panel};
use crate::journal::{Entry, Verdict};
use chrono::Local;
use eframe::egui::{self, Color32, RichText, Ui};
use egui_extras::{Column, TableBuilder};

pub fn show(panel: &mut Panel, ui: &mut Ui) {
    ui.add_space(theme::GAP);
    summary(panel, ui);
    ui.add_space(theme::GAP);
    controls(panel, ui);
    ui.add_space(6.0);

    let rows = filtered(panel);
    if rows.is_empty() {
        let s = panel.strings();
        let message = if panel.entries.is_empty() {
            s.nothing_yet
        } else {
            s.nothing_matches
        };
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(theme::muted(message, panel.dark));
        });
        return;
    }
    table(panel, ui, &rows);
}

fn summary(panel: &Panel, ui: &mut Ui) {
    let s = panel.strings();
    let today = Local::now().date_naive();
    let todays: Vec<&Entry> = panel
        .entries
        .iter()
        .filter(|e| e.at.date_naive() == today && e.verdict.is_interruption())
        .collect();
    let restored = todays
        .iter()
        .filter(|e| e.verdict == Verdict::Restored)
        .count();
    let worst = panel
        .stats
        .by_app
        .first()
        .map(|(exe, count)| format!("{exe} ({count})"))
        .unwrap_or_else(|| s.summary_none.to_string());

    ui.horizontal(|ui| {
        ui.add_space(theme::MARGIN);
        theme::card(ui, panel.dark, |ui| {
            ui.horizontal(|ui| {
                theme::stat(
                    ui,
                    &todays.len().to_string(),
                    &format!("{} · {}", s.summary_today, s.summary_interruptions),
                    panel.dark,
                );
                ui.add_space(28.0);
                theme::stat(ui, &restored.to_string(), s.summary_taken_back, panel.dark);
                ui.add_space(28.0);
                theme::stat_text(ui, &worst, s.summary_worst, panel.dark);
            });
        });
    });
}

fn controls(panel: &mut Panel, ui: &mut Ui) {
    let s = panel.strings();
    ui.horizontal(|ui| {
        ui.add_space(theme::MARGIN);
        let search = egui::TextEdit::singleline(&mut panel.search)
            .hint_text(s.search_placeholder)
            .desired_width(280.0);
        ui.add(search);
        ui.add_space(theme::GAP);
        for (filter, label) in [
            (Filter::All, s.filter_all),
            (Filter::Restored, s.filter_restored),
            (Filter::Seen, s.filter_seen),
        ] {
            if theme::pill(ui, label, panel.filter == filter, panel.dark).clicked() {
                panel.filter = filter;
            }
        }
    });
}

/// Whether one entry belongs in the list as it is currently filtered. Split out
/// from the drawing so it can be tested without a window on screen.
pub fn shows(entry: &Entry, filter: Filter, needle: &str, record_everything: bool) -> bool {
    let passes_filter = match filter {
        Filter::All => entry.verdict.is_interruption() || record_everything,
        Filter::Restored => entry.verdict == Verdict::Restored,
        Filter::Seen => matches!(entry.verdict, Verdict::Observed | Verdict::GaveUp),
    };
    if !passes_filter {
        return false;
    }
    needle.is_empty()
        || entry.exe.to_lowercase().contains(needle)
        || entry.title.to_lowercase().contains(needle)
}

fn filtered(panel: &Panel) -> Vec<usize> {
    let needle = panel.search.trim().to_lowercase();
    panel
        .entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| shows(entry, panel.filter, &needle, panel.cfg.record_everything))
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::Reason;

    fn entry(exe: &str, title: &str, verdict: Verdict) -> Entry {
        Entry {
            at: Local::now(),
            exe: exe.into(),
            path: String::new(),
            title: title.into(),
            verdict,
            reason: Reason::Typing,
        }
    }

    #[test]
    fn all_hides_the_ones_that_were_left_alone() {
        let allowed = entry("explorer.exe", "Desktop", Verdict::Allowed);
        assert!(!shows(&allowed, Filter::All, "", false));
        // Unless you asked to see them.
        assert!(shows(&allowed, Filter::All, "", true));
    }

    #[test]
    fn each_filter_keeps_only_its_own() {
        let took_back = entry("teams.exe", "Call", Verdict::Restored);
        let seen = entry("teams.exe", "Call", Verdict::Observed);
        let gave_up = entry("teams.exe", "Call", Verdict::GaveUp);

        assert!(shows(&took_back, Filter::Restored, "", false));
        assert!(!shows(&seen, Filter::Restored, "", false));

        assert!(shows(&seen, Filter::Seen, "", false));
        assert!(shows(&gave_up, Filter::Seen, "", false));
        assert!(!shows(&took_back, Filter::Seen, "", false));
    }

    #[test]
    fn search_looks_at_both_the_name_and_the_title() {
        let item = entry("teams.exe", "Someone is calling you", Verdict::Restored);
        assert!(shows(&item, Filter::All, "teams", false));
        assert!(shows(&item, Filter::All, "calling", false));
        assert!(!shows(&item, Filter::All, "outlook", false));
    }

    #[test]
    fn search_ignores_case_on_both_sides() {
        let item = entry("Teams.exe", "Someone Is Calling", Verdict::Restored);
        assert!(shows(&item, Filter::All, "teams", false));
        assert!(shows(&item, Filter::All, "calling", false));
    }

    #[test]
    fn a_search_still_obeys_the_filter() {
        let seen = entry("teams.exe", "Call", Verdict::Observed);
        assert!(!shows(&seen, Filter::Restored, "teams", false));
    }
}

fn table(panel: &mut Panel, ui: &mut Ui, rows: &[usize]) {
    let s = panel.strings();
    let dark = panel.dark;
    let p = palette(dark);
    let mut rule: Option<(Rule, String)> = None;

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(theme::MARGIN as i8, 0))
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(96.0))
                .column(Column::initial(190.0).at_least(120.0))
                .column(Column::exact(110.0))
                .column(Column::initial(170.0).at_least(120.0))
                .column(Column::remainder())
                .header(26.0, |mut header| {
                    for label in [
                        s.column_when,
                        s.column_app,
                        s.column_what,
                        s.column_why,
                        s.column_title,
                    ] {
                        header.col(|ui| {
                            ui.label(theme::heading(label, dark));
                        });
                    }
                })
                .body(|body| {
                    // Virtual rows: only what is on screen is laid out, so a full
                    // fortnight of history scrolls without costing anything.
                    body.rows(theme::ROW_HEIGHT, rows.len(), |mut row| {
                        let entry = &panel.entries[rows[row.index()]];
                        row.col(|ui| {
                            ui.label(
                                RichText::new(entry.at.format("%d.%m %H:%M").to_string())
                                    .color(p.muted)
                                    .size(12.5),
                            );
                        });
                        row.col(|ui| {
                            let response = ui.label(RichText::new(&entry.exe).color(p.text));
                            context_menu(panel, response, entry, &mut rule);
                        });
                        row.col(|ui| {
                            ui.label(
                                RichText::new(entry.verdict.label(s))
                                    .color(verdict_colour(entry.verdict, dark))
                                    .size(12.5),
                            );
                        });
                        row.col(|ui| {
                            ui.label(
                                RichText::new(entry.reason.label(s))
                                    .color(p.muted)
                                    .size(12.5),
                            );
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(&entry.title).color(p.muted).size(12.5))
                                .on_hover_text(&entry.title);
                        });
                    });
                });
        });

    if let Some((rule, exe)) = rule {
        match rule {
            Rule::Block => panel.cfg.block(&exe),
            Rule::Allow => panel.cfg.allow(&exe),
            Rule::Forget => panel.cfg.forget(&exe),
        }
        panel.save();
    }
}

#[derive(Clone, Copy)]
enum Rule {
    Block,
    Allow,
    Forget,
}

fn context_menu(
    panel: &Panel,
    response: egui::Response,
    entry: &Entry,
    out: &mut Option<(Rule, String)>,
) {
    let s = panel.strings();
    response.context_menu(|ui| {
        ui.set_min_width(220.0);
        if ui
            .button(format!("{} {}", s.menu_block, entry.exe))
            .clicked()
        {
            *out = Some((Rule::Block, entry.exe.clone()));
            ui.close();
        }
        if ui
            .button(format!("{} {}", s.menu_allow, entry.exe))
            .clicked()
        {
            *out = Some((Rule::Allow, entry.exe.clone()));
            ui.close();
        }
        if ui
            .button(format!("{} {}", s.menu_forget, entry.exe))
            .clicked()
        {
            *out = Some((Rule::Forget, entry.exe.clone()));
            ui.close();
        }
        if !entry.path.is_empty() {
            ui.separator();
            if ui.button(s.menu_reveal).clicked() {
                let _ = std::process::Command::new("explorer.exe")
                    .arg(format!("/select,{}", entry.path))
                    .spawn();
                ui.close();
            }
        }
    });
}

fn verdict_colour(verdict: Verdict, dark: bool) -> Color32 {
    let p = palette(dark);
    match verdict {
        Verdict::Restored => p.taken_back,
        Verdict::GaveUp => p.gave_up,
        Verdict::Observed => p.seen,
        Verdict::Allowed => p.muted,
    }
}
