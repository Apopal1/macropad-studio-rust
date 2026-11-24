use super::*;
use deck_domain::ActionKind;
impl Studio {
    pub(super) fn inspector(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().id_salt("inspector").show(ui, |ui| {
            card().show(ui, |ui| {
                ui.horizontal(|ui| {
                    eyebrow(ui, "ΕΠΕΞΕΡΓΑΣΙΑ ΠΛΗΚΤΡΟΥ");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("K{}", self.key))
                                .color(MINT)
                                .strong()
                                .size(18.0),
                        );
                    });
                });
                ui.add_space(10.0);
                let profile_id = self.profile().id.clone();
                let profile_names: Vec<_> = self
                    .draft
                    .profiles
                    .iter()
                    .take(8)
                    .map(|p| (p.id.clone(), p.name.clone()))
                    .collect();
                let key_id = self.key;
                let key = self.draft.profiles[self.profile]
                    .keys
                    .iter_mut()
                    .find(|k| k.id == key_id)
                    .unwrap();
                ui.label("Όνομα πλήκτρου");
                ui.add(
                    TextEdit::singleline(&mut key.label)
                        .char_limit(40)
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal(|ui| {
                    ui.label("Χρώμα");
                    for i in 0..5 {
                        if ui
                            .add(
                                Button::new(" ")
                                    .fill(tint(i))
                                    .min_size(vec2(22.0, 22.0))
                                    .stroke(stroke(if key.tint == i { 2.0 } else { 0.0 }, TEXT)),
                            )
                            .on_hover_text(format!("Χρώμα {}", i + 1))
                            .clicked()
                        {
                            key.tint = i;
                        }
                    }
                });
                ui.add_space(6.0);
                ui.separator();
                eyebrow(ui, "ΑΚΟΛΟΥΘΙΑ ΕΝΕΡΓΕΙΩΝ");
                ui.label(
                    RichText::new("Τα βήματα εκτελούνται με τη σειρά.")
                        .size(13.0)
                        .color(MUTED),
                );
                let mut remove = None;
                let mut swap = None;
                let mut pick = None;
                let count = key.actions.len();
                for (i, a) in key.actions.iter_mut().enumerate() {
                    ui.push_id(i, |ui| {
                        Frame::new()
                            .fill(BG)
                            .corner_radius(8)
                            .inner_margin(12)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{:02}", i + 1))
                                            .monospace()
                                            .color(MUTED),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.small_button("Αφαίρεση").clicked() {
                                            remove = Some(i);
                                        }
                                    });
                                });
                                let old = a.kind;
                                ComboBox::from_id_salt("action-kind")
                                    .width(ui.available_width() - 5.0)
                                    .selected_text(a.kind.title())
                                    .show_ui(ui, |ui| {
                                        for k in ActionKind::ALL {
                                            ui.selectable_value(&mut a.kind, k, k.title());
                                        }
                                    });
                                if old != a.kind {
                                    *a = a.kind.default_action();
                                    if a.kind == ActionKind::Mode {
                                        a.value = profile_names
                                            .first()
                                            .map(|p| p.0.clone())
                                            .unwrap_or_default();
                                    }
                                }
                                match a.kind {
                                    ActionKind::Hotkey => {
                                        ui.add(
                                            TextEdit::singleline(&mut a.value)
                                                .hint_text("Ctrl+Shift+S")
                                                .desired_width(f32::INFINITY),
                                        );
                                    }
                                    ActionKind::Text => {
                                        ui.add(
                                            TextEdit::multiline(&mut a.value)
                                                .desired_rows(3)
                                                .char_limit(4000)
                                                .desired_width(f32::INFINITY),
                                        );
                                    }
                                    ActionKind::Application => {
                                        ui.add(
                                            TextEdit::singleline(&mut a.value)
                                                .hint_text("C:\\...\\app.exe")
                                                .desired_width(f32::INFINITY),
                                        );
                                        if ui.small_button("Επιλογή αρχείου…").clicked()
                                        {
                                            pick = Some(i);
                                        }
                                        ui.label(
                                            RichText::new("Ορίσματα (προαιρετικά)")
                                                .size(13.0)
                                                .color(MUTED),
                                        );
                                        ui.add(
                                            TextEdit::singleline(&mut a.args)
                                                .desired_width(f32::INFINITY),
                                        );
                                    }
                                    ActionKind::Url => {
                                        ui.add(
                                            TextEdit::singleline(&mut a.value)
                                                .hint_text("https://...")
                                                .desired_width(f32::INFINITY),
                                        );
                                    }
                                    ActionKind::Delay => {
                                        ui.horizontal(|ui| {
                                            ui.add(
                                                TextEdit::singleline(&mut a.value)
                                                    .desired_width(90.0),
                                            );
                                            ui.label("ms · έως 5000");
                                        });
                                    }
                                    ActionKind::HomeAssistant => {
                                        ui.add(
                                            TextEdit::singleline(&mut a.value)
                                                .hint_text("light.turn_off")
                                                .desired_width(f32::INFINITY),
                                        );
                                        ui.label(
                                            RichText::new("Παράμετροι JSON")
                                                .size(13.0)
                                                .color(MUTED),
                                        );
                                        ui.add(
                                            TextEdit::multiline(&mut a.args)
                                                .code_editor()
                                                .desired_rows(3)
                                                .desired_width(f32::INFINITY),
                                        );
                                    }
                                    ActionKind::Mode => {
                                        ComboBox::from_id_salt("target-mode")
                                            .selected_text(
                                                profile_names
                                                    .iter()
                                                    .find(|p| p.0 == a.value)
                                                    .map(|p| p.1.as_str())
                                                    .unwrap_or("Διάλεξε mode"),
                                            )
                                            .show_ui(ui, |ui| {
                                                for (id, name) in &profile_names {
                                                    ui.selectable_value(
                                                        &mut a.value,
                                                        id.clone(),
                                                        name,
                                                    );
                                                }
                                            });
                                    }
                                    ActionKind::DiscordMute | ActionKind::DiscordDeafen => {
                                        ui.label(
                                            RichText::new(if a.kind == ActionKind::DiscordMute {
                                                "Discord keybind: Ctrl+Alt+Shift+M"
                                            } else {
                                                "Discord keybind: Ctrl+Alt+Shift+D"
                                            })
                                            .size(13.0)
                                            .color(MUTED),
                                        );
                                    }
                                    _ => {}
                                }
                                if count > 1 {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add_enabled(i > 0, Button::new("Πάνω").small())
                                            .clicked()
                                        {
                                            swap = Some((i, i - 1));
                                        }
                                        if ui
                                            .add_enabled(i + 1 < count, Button::new("Κάτω").small())
                                            .clicked()
                                        {
                                            swap = Some((i, i + 1));
                                        }
                                    });
                                }
                                if let Err(e) = a.validate() {
                                    ui.label(RichText::new(e.to_string()).size(13.0).color(RED));
                                }
                            });
                    });
                }
                if let Some(i) = remove {
                    key.actions.remove(i);
                }
                if let Some((a, b)) = swap {
                    key.actions.swap(a, b);
                }
                if ui
                    .add_enabled(key.actions.len() < 12, Button::new("+ Προσθήκη βήματος"))
                    .clicked()
                {
                    key.actions.push(ActionKind::Hotkey.default_action());
                }
                if let Some(step) = pick {
                    self.send(Command::PickApplication {
                        profile: profile_id.clone(),
                        key: key_id,
                        step,
                    });
                }
                ui.add_space(12.0);
                ui.separator();
                ui.label(
                    RichText::new("Η δοκιμή χρησιμοποιεί τις αποθηκευμένες ενέργειες.")
                        .size(13.0)
                        .color(MUTED),
                );
                let enabled = !self.dirty() && !self.status.paused;
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(enabled, Button::new("Προσομοίωση"))
                        .clicked()
                    {
                        self.send(Command::Test {
                            profile: profile_id.clone(),
                            key: key_id,
                            simulate: true,
                        });
                    }
                    if primary(ui, "Δοκιμή σε 3 s", enabled && !self.status.demo).clicked()
                    {
                        self.send(Command::Test {
                            profile: profile_id,
                            key: key_id,
                            simulate: false,
                        });
                    }
                });
            });
        });
    }
}
