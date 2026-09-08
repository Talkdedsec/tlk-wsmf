use super::Panel;
use super::theme::{self, palette};
use eframe::egui::{self, RichText, Ui};

pub fn show(panel: &mut Panel, ui: &mut Ui) {
    ui.add_space(theme::GAP);
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(theme::MARGIN as i8, 0))
        .show(ui, |ui| {
            ui.columns(2, |columns| {
                list(panel, &mut columns[0], Which::Block);
                list(panel, &mut columns[1], Which::Allow);
            });
        });
}

#[derive(Clone, Copy, PartialEq)]
enum Which {
    Block,
    Allow,
}

impl Which {
    fn id(self) -> &'static str {
        match self {
            Which::Block => "blocklist",
            Which::Allow => "allowlist",
        }
    }
}

fn list(panel: &mut Panel, ui: &mut Ui, which: Which) {
    let s = panel.strings();
    let dark = panel.dark;
    let p = palette(dark);
    let (heading, hint) = match which {
        Which::Block => (s.rules_blocked, s.rules_blocked_hint),
        Which::Allow => (s.rules_allowed, s.rules_allowed_hint),
    };
    let mut remove: Option<String> = None;
    let mut add: Option<String> = None;

    theme::card(ui, dark, |ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(heading).size(13.5).strong().color(p.text));
            ui.label(theme::muted(hint, dark));
            ui.add_space(10.0);

            let names: Vec<String> = match which {
                Which::Block => panel.cfg.blocklist.clone(),
                Which::Allow => panel.cfg.allowlist.clone(),
            };

            if names.is_empty() {
                ui.label(theme::muted(s.rules_empty, dark));
            } else {
                egui::ScrollArea::vertical()
                    .max_height(340.0)
                    .id_salt(which.id())
                    .show(ui, |ui| {
                        for name in &names {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(name).color(p.text).size(13.0));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .small_button("×")
                                            .on_hover_text(s.rules_remove)
                                            .clicked()
                                        {
                                            remove = Some(name.clone());
                                        }
                                    },
                                );
                            });
                        }
                    });
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let buffer = match which {
                    Which::Block => &mut panel.new_block,
                    Which::Allow => &mut panel.new_allow,
                };
                let field = egui::TextEdit::singleline(buffer)
                    .hint_text(s.rules_new_placeholder)
                    .desired_width(ui.available_width() - 74.0);
                let entered = ui.add(field).lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                let pressed = ui.button(s.rules_add).clicked();
                if (entered || pressed) && !buffer.trim().is_empty() {
                    add = Some(std::mem::take(buffer));
                }
            });
        });
    });

    if let Some(name) = remove {
        panel.cfg.forget(&name);
        panel.save();
    }
    if let Some(name) = add {
        match which {
            Which::Block => panel.cfg.block(&name),
            Which::Allow => panel.cfg.allow(&name),
        }
        panel.save();
    }
}
