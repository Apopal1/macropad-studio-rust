use super::*;
use deck_domain::{ActionKind, KEY_ORDER, oled_label};
impl Studio {
    pub(super) fn modes(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            eyebrow(ui, "MODES");
            let width = (ui.available_width() - 160.0).max(180.0);
            ScrollArea::horizontal()
                .id_salt("mode-list")
                .max_width(width)
                .max_height(42.0)
                .min_scrolled_height(0.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (i, p) in self.draft.profiles.iter().enumerate() {
                            let active = p.id == self.saved.active_profile;
                            let s = format!(
                                "{}  {}{}",
                                i + 1,
                                p.name,
                                if active { " · ενεργό" } else { "" }
                            );
                            if ui
                                .add(
                                    Button::new(RichText::new(s).color(if self.profile == i {
                                        MINT
                                    } else {
                                        MUTED
                                    }))
                                    .fill(if self.profile == i {
                                        Color32::from_rgb(32, 53, 47)
                                    } else {
                                        CARD
                                    })
                                    .stroke(stroke(
                                        1.0,
                                        if self.profile == i { MINT } else { LINE },
                                    )),
                                )
                                .clicked()
                            {
                                self.profile = i;
                            }
                        }
                    });
                });
            if ui
                .add_enabled(self.draft.profiles.len() < 12, Button::new("+ Νέο mode"))
                .clicked()
            {
                self.add_profile(false);
            }
        });
        ui.add_space(10.0);
    }
    pub(super) fn deck_page(&mut self, ui: &mut Ui) {
        self.modes(ui);
        ui.separator();
        ui.add_space(10.0);
        let available = ui.available_rect_before_wrap();
        let right_width = 330.0_f32.min(available.width() * 0.39);
        let left = Rect::from_min_max(
            available.min,
            pos2(available.max.x - right_width - 22.0, available.max.y),
        );
        let right = Rect::from_min_max(
            pos2(available.max.x - right_width, available.min.y),
            available.max,
        );
        ui.scope_builder(UiBuilder::new().max_rect(left).layout(Layout::top_down(Align::Min)),|ui|{
            ScrollArea::vertical().id_salt("deck-scroll").show(ui,|ui|{
                ui.horizontal(|ui|{
                    ui.label(RichText::new(self.profile().name.clone()).strong().size(20.0));
                    ui.with_layout(Layout::right_to_left(Align::Center),|ui|{
                        let id=self.profile().id.clone();
                        if ui.add_enabled(self.profile<8&&!self.dirty()&&id!=self.saved.active_profile,Button::new("Ενεργοποίηση mode")).clicked(){self.send(Command::Activate(id));}
                    });
                });
                muted(ui,if self.profile<8{format!("Επιλογή από το pad: κράτησε K9 και πάτησε K{}",self.profile+1)}else{"Προφίλ βιβλιοθήκης · χωρίς φυσικό συνδυασμό".into()});
                ui.add_space(16.0);
                let device_width=(left.width()-12.0).clamp(280.0,390.0);
                let offset=((ui.available_width()-device_width)/2.0).max(0.0);
                ui.horizontal(|ui|{
                    ui.add_space(offset);
                    Frame::new().fill(Color32::from_rgb(26,31,37)).stroke(stroke(1.5,LINE)).corner_radius(20).inner_margin(22).show(ui,|ui|{
                        ui.vertical(|ui|{
                            ui.set_width(device_width-44.0);
                            self.oled_header(ui);ui.add_space(6.0);
                            let tile=((device_width-44.0-24.0)/3.0).max(60.0);
                            Grid::new("physical-keys").spacing(vec2(12.0,12.0)).show(ui,|ui|{
                                for ids in KEY_ORDER.chunks(3){for id in ids{self.key_tile(ui,*id,tile);}ui.end_row();}
                            });
                            ui.add_space(8.0);
                            ui.horizontal(|ui|{
                                let width=(device_width-64.0)/3.0;
                                for name in ["MASTER","APP 01","APP 02"]{
                                    ui.allocate_ui_with_layout(vec2(width,64.0),Layout::top_down(Align::Center),|ui|{
                                        let(r,response)=ui.allocate_exact_size(vec2(34.0,34.0),Sense::hover());
                                        ui.painter().circle_filled(r.center(),17.0,CARD);
                                        ui.painter().circle_stroke(r.center(),17.0,stroke(2.0,LINE));
                                        ui.painter().line_segment([r.center()+vec2(0.0,-12.0),r.center()+vec2(0.0,-5.0)],stroke(2.0,MUTED));
                                        response.on_hover_text("Προεπισκόπηση θέσης. Η είσοδος ποτενσιόμετρων δεν υλοποιείται ακόμα.");
                                        ui.label(RichText::new(name).size(11.0).color(MUTED));
                                    });
                                }
                            });
                        });
                    });
                });
                ui.add_space(12.0);ui.vertical_centered(|ui|{muted(ui,"Επίλεξε πλήκτρο για επεξεργασία.");ui.small("Τα ποτενσιόμετρα είναι προεπισκόπηση σχεδίου.");});
            });
        });
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(right)
                .layout(Layout::top_down(Align::Min)),
            |ui| self.inspector(ui),
        );
    }
    fn key_tile(&mut self, ui: &mut Ui, id: u8, size: f32) {
        let key = self.profile().keys.iter().find(|k| k.id == id).unwrap();
        let selected = self.key == id;
        let accent = tint(key.tint);
        let glyph = key
            .actions
            .first()
            .map(|a| match a.kind {
                ActionKind::Hotkey => a.value.to_uppercase(),
                ActionKind::VolumeUp => "+".into(),
                ActionKind::VolumeDown => "−".into(),
                ActionKind::VolumeMute => "MUTE".into(),
                ActionKind::DiscordMute => "MIC".into(),
                ActionKind::DiscordDeafen => "DEAF".into(),
                ActionKind::PlayPause => "▶ Ⅱ".into(),
                ActionKind::Application => "APP".into(),
                ActionKind::HomeAssistant => "HA".into(),
                ActionKind::Mode | ActionKind::ModeNext => "MODE".into(),
                ActionKind::Text => "Aa".into(),
                ActionKind::Url => "URL".into(),
                _ => "•".into(),
            })
            .unwrap_or_else(|| "+".into());
        let response = ui.add_sized(
            [size, size * 0.86],
            Button::new("")
                .fill(if selected {
                    Color32::from_rgb(37, 62, 54)
                } else {
                    CARD
                })
                .stroke(stroke(
                    if selected { 2.0 } else { 1.0 },
                    if selected { MINT } else { LINE },
                ))
                .corner_radius(12),
        );
        response.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Button,
                ui.is_enabled(),
                format!("K{id}: {}", key.label),
            )
        });
        let r = response.rect;
        ui.painter().text(
            r.min + vec2(10.0, 9.0),
            Align2::LEFT_TOP,
            format!("K{id}"),
            FontId::proportional(11.0),
            MUTED,
        );
        if key.actions.len() > 1 {
            ui.painter().text(
                r.right_top() + vec2(-9.0, 9.0),
                Align2::RIGHT_TOP,
                format!("{}", key.actions.len()),
                FontId::proportional(11.0),
                MUTED,
            );
        }
        let glyph_size = if glyph.chars().count() > 7 {
            17.0
        } else {
            24.0
        };
        ui.painter().text(
            r.center() + vec2(0.0, -10.0),
            Align2::CENTER_CENTER,
            glyph,
            FontId::proportional(glyph_size),
            accent,
        );
        let mut job = eframe::egui::text::LayoutJob::simple(
            key.label.clone(),
            FontId::proportional(14.0),
            TEXT,
            r.width() - 16.0,
        );
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        let galley = ui.painter().layout_job(job);
        let textpos = pos2(
            r.center().x - galley.size().x / 2.0,
            r.bottom() - galley.size().y - 12.0,
        );
        ui.painter().galley(textpos, galley, TEXT);
        let clicked = response.clicked();
        response.on_hover_text(format!("K{id}: {}", key.label));
        if clicked {
            self.key = id;
            self.frame = 0;
        }
    }
    fn oled_header(&self, ui: &mut Ui) {
        let width = ui.available_width().min(280.0);
        ui.vertical_centered(|ui| {
            Frame::new()
                .fill(Color32::from_rgb(7, 16, 17))
                .stroke(stroke(1.0, Color32::from_rgb(51, 83, 76)))
                .corner_radius(8)
                .inner_margin(Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.set_width(width - 28.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{:02}", self.profile + 1))
                                .monospace()
                                .color(MINT)
                                .strong()
                                .size(19.0),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(oled_label(self.profile(), self.profile))
                                .monospace()
                                .color(MINT)
                                .size(16.0),
                        );
                    });
                    ui.label(
                        RichText::new("128 × 64  /  OLED PREVIEW")
                            .size(10.0)
                            .color(Color32::from_rgb(101, 145, 130)),
                    );
                });
        });
    }
}
