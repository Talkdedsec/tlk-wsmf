//! The panel: a separate process (`wsmf --panel`) so that the part of the program
//! guarding your focus stays small, and a crash while drawing a chart cannot take
//! the guard down with it. The two talk through the settings file and the log.

mod activity;
mod rules;
mod settings;
mod stats;
mod theme;

use crate::config::{self, Config, Mode};
use crate::history;
use crate::i18n::Strings;
use crate::journal::Entry;
use eframe::egui;
use std::time::{Duration, Instant, SystemTime};

const HISTORY_DAYS: i64 = 14;
const POLL: Duration = Duration::from_millis(900);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Activity,
    Rules,
    Stats,
    Settings,
}

impl Tab {
    /// `wsmf --panel --tab stats` opens straight onto a tab.
    fn from_arguments() -> Self {
        let mut args = std::env::args().skip_while(|arg| arg != "--tab");
        match args.nth(1).as_deref() {
            Some("rules") => Tab::Rules,
            Some("stats") => Tab::Stats,
            Some("settings") => Tab::Settings,
            _ => Tab::Activity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Restored,
    Seen,
}

pub struct Panel {
    cfg: Config,
    entries: Vec<Entry>,
    stats: history::Stats,
    tab: Tab,
    filter: Filter,
    search: String,
    new_block: String,
    new_allow: String,
    dark: bool,
    last_poll: Instant,
    log_stamp: Option<SystemTime>,
    config_stamp: Option<SystemTime>,
    saved_at: Option<Instant>,
}

impl Panel {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let dark = crate::system::dark_mode();
        theme::apply(&cc.egui_ctx, dark);
        let cfg = Config::load();
        let entries = history::load(HISTORY_DAYS);
        let stats = history::summarize(&entries, HISTORY_DAYS);
        Self {
            cfg,
            entries,
            stats,
            tab: Tab::from_arguments(),
            filter: Filter::All,
            search: String::new(),
            new_block: String::new(),
            new_allow: String::new(),
            dark,
            last_poll: Instant::now(),
            log_stamp: file_stamp(&config::log_path()),
            config_stamp: config::changed_at(),
            saved_at: None,
        }
    }

    fn strings(&self) -> &'static Strings {
        self.cfg.strings()
    }

    /// The guard process owns the log and may own the settings too, if the user
    /// changed the mode from the tray while this window was open.
    fn poll_for_outside_changes(&mut self, ctx: &egui::Context) {
        if self.last_poll.elapsed() < POLL {
            return;
        }
        self.last_poll = Instant::now();

        let log_stamp = file_stamp(&config::log_path());
        if log_stamp != self.log_stamp {
            self.log_stamp = log_stamp;
            self.reload_history();
        }

        let config_stamp = config::changed_at();
        if config_stamp != self.config_stamp {
            self.config_stamp = config_stamp;
            let fresh = Config::load();
            if fresh != self.cfg {
                self.cfg = fresh;
            }
        }

        let dark = crate::system::dark_mode();
        if dark != self.dark {
            self.dark = dark;
            theme::apply(ctx, dark);
        }
    }

    fn reload_history(&mut self) {
        self.entries = history::load(HISTORY_DAYS);
        self.stats = history::summarize(&self.entries, HISTORY_DAYS);
    }

    fn save(&mut self) {
        let _ = self.cfg.save();
        self.config_stamp = config::changed_at();
        self.saved_at = Some(Instant::now());
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let s = self.strings();
        egui::Panel::top("top")
            .resizable(false)
            .default_size(theme::TOP_BAR_HEIGHT)
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(theme::MARGIN);
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        ui.label(theme::title(s.app_name, self.dark));
                        ui.label(theme::muted(s.tagline, self.dark));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(theme::MARGIN);
                        let mut mode = self.cfg.mode;
                        for candidate in Mode::ALL.into_iter().rev() {
                            if theme::pill(ui, candidate.label(s), mode == candidate, self.dark)
                                .on_hover_text(candidate.hint(s))
                                .clicked()
                            {
                                mode = candidate;
                            }
                        }
                        if mode != self.cfg.mode {
                            self.cfg.mode = mode;
                            self.save();
                        }
                    });
                });
            });
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        let s = self.strings();
        egui::Panel::top("tabs")
            .resizable(false)
            .default_size(theme::TAB_BAR_HEIGHT)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(theme::MARGIN);
                    for (tab, label) in [
                        (Tab::Activity, s.tab_activity),
                        (Tab::Rules, s.tab_rules),
                        (Tab::Stats, s.tab_stats),
                        (Tab::Settings, s.tab_settings),
                    ] {
                        if theme::tab(ui, label, self.tab == tab, self.dark).clicked() {
                            self.tab = tab;
                        }
                    }
                });
            });
    }
}

impl eframe::App for Panel {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_for_outside_changes(&ctx);
        self.top_bar(ui);
        self.tabs(ui);
        egui::CentralPanel::default().show(ui, |ui| match self.tab {
            Tab::Activity => activity::show(self, ui),
            Tab::Rules => rules::show(self, ui),
            Tab::Stats => stats::show(self, ui),
            Tab::Settings => settings::show(self, ui),
        });
        // Nothing animates, so the window only wakes up to notice new entries.
        ctx.request_repaint_after(POLL);
    }
}

fn file_stamp(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

pub fn run() -> Result<(), eframe::Error> {
    let icon = load_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 620.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("Who Stole My Focus")
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "wsmf-panel",
        options,
        Box::new(|cc| Ok(Box::new(Panel::new(cc)))),
    )
}

/// The window icon. `tools/make-icon.ps1` writes this raw RGBA alongside the .ico
/// so the panel needs no image decoder for one 64 px square.
fn load_icon() -> egui::IconData {
    const SIDE: u32 = 64;
    const PIXELS: &[u8] = include_bytes!("../../assets/wsmf-64.rgba");
    egui::IconData {
        rgba: PIXELS.to_vec(),
        width: SIDE,
        height: SIDE,
    }
}
