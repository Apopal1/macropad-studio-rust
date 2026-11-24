mod deck;
mod inspector;
mod pages;
use crate::theme::*;
use deck_domain::{Config, Profile, artwork::Artwork};
use deck_platform::ha::Catalog;
use deck_runtime::{Activity, Command, Event, Runtime, Status};
use eframe::egui::*;
use std::{collections::VecDeque, path::PathBuf, sync::Arc, time::Instant};
#[derive(Clone, Copy, PartialEq)]
enum Page {
    Deck,
    Display,
    Home,
    Activity,
    Settings,
}
#[derive(Clone, Copy)]
enum Confirm {
    Revert,
    Import,
    DeleteProfile,
    Close,
}
pub struct Studio {
    runtime: Runtime,
    draft: Config,
    saved: Config,
    art: Artwork,
    art_saved: Artwork,
    status: Status,
    ready: bool,
    fatal: bool,
    page: Page,
    profile: usize,
    key: u8,
    activity: VecDeque<Activity>,
    notice: Option<(String, bool)>,
    confirm: Option<Confirm>,
    allow_close: bool,
    frame: usize,
    playing: bool,
    frame_started: Instant,
    eraser: bool,
    catalog: Catalog,
    ha_url: String,
    ha_token: String,
    ha_search: String,
    ha_service: String,
    ha_data: String,
    zoom: f32,
    started: Instant,
}
impl Studio {
    pub fn new(cc: &eframe::CreationContext<'_>, demo: bool, root: Option<PathBuf>) -> Self {
        Self::create(&cc.egui_ctx, demo, root)
    }
    fn create(ctx: &Context, demo: bool, root: Option<PathBuf>) -> Self {
        setup(ctx);
        let ctx = ctx.clone();
        Self {
            runtime: Runtime::start(demo, root, Arc::new(move || ctx.request_repaint())),
            draft: Config::default(),
            saved: Config::default(),
            art: Artwork::default(),
            art_saved: Artwork::default(),
            status: Status {
                demo,
                ..Default::default()
            },
            ready: false,
            fatal: false,
            page: Page::Deck,
            profile: 0,
            key: 1,
            activity: VecDeque::new(),
            notice: None,
            confirm: None,
            allow_close: false,
            frame: 0,
            playing: false,
            frame_started: Instant::now(),
            eraser: false,
            catalog: Catalog::default(),
            ha_url: String::new(),
            ha_token: String::new(),
            ha_search: String::new(),
            ha_service: "light.toggle".into(),
            ha_data: "{}".into(),
            zoom: 1.0,
            started: Instant::now(),
        }
    }
    fn send(&mut self, c: Command) {
        if let Err(e) = self.runtime.send(c) {
            self.notice = Some((e.to_string(), true));
        }
    }
    fn dirty(&self) -> bool {
        self.draft != self.saved
    }
    fn art_dirty(&self) -> bool {
        self.art != self.art_saved
    }
    fn profile(&self) -> &Profile {
        &self.draft.profiles[self.profile.min(self.draft.profiles.len() - 1)]
    }
    fn save(&mut self) {
        match self.draft.validate() {
            Ok(()) => self.send(Command::Save(self.draft.clone())),
            Err(e) => self.notice = Some((e.to_string(), true)),
        }
    }
    fn events(&mut self) {
        while let Ok(e) = self.runtime.events.try_recv() {
            match e {
                Event::Ready {
                    config,
                    artwork,
                    status,
                } => {
                    self.draft = config.clone();
                    self.saved = config;
                    self.art = artwork.clone();
                    self.art_saved = artwork;
                    self.status = status;
                    self.ready = true;
                }
                Event::Saved(c) => {
                    self.draft.active_profile = c.active_profile.clone();
                    self.saved = c;
                    self.notice=Some((if self.dirty(){"Αποθηκεύτηκε η προηγούμενη αλλαγή. Υπάρχουν νεότερες αλλαγές στο draft."}else{"Οι ρυθμίσεις αποθηκεύτηκαν στον υπολογιστή."}.into(),false));
                }
                Event::Imported(c) => {
                    self.draft = c;
                    self.profile = 0;
                    self.notice = Some((
                        "Το αρχείο φορτώθηκε ως draft. Έλεγξέ το και αποθήκευσε.".into(),
                        false,
                    ));
                }
                Event::ArtworkImported { key, clip } => {
                    self.art.keys[key as usize - 1] = clip;
                    self.frame = 0;
                }
                Event::ArtworkSaved(a) => {
                    self.art_saved = a;
                    self.notice = Some(("Τα γραφικά αποθηκεύτηκαν στον υπολογιστή.".into(), false));
                }
                Event::Status(s) => {
                    if !s.ha_url.trim().is_empty() {
                        self.ha_url = s.ha_url.clone();
                    }
                    self.status = s;
                }
                Event::Catalog(c) => self.catalog = c,
                Event::Activity(a) => {
                    if self.activity.len() >= 200 {
                        self.activity.pop_front();
                    }
                    if a.error {
                        self.notice = Some((a.message.clone(), true));
                    }
                    self.activity.push_back(a);
                }
                Event::ApplicationPicked {
                    profile,
                    key,
                    step,
                    path,
                } => {
                    if let Some(a) = self
                        .draft
                        .profiles
                        .iter_mut()
                        .find(|p| p.id == profile)
                        .and_then(|p| p.keys.iter_mut().find(|k| k.id == key))
                        .and_then(|k| k.actions.get_mut(step))
                        && a.kind == deck_domain::ActionKind::Application
                    {
                        a.value = path;
                    }
                }
                Event::Notice(s) => self.notice = Some((s, true)),
                Event::Fatal(s) => {
                    self.notice = Some((s, true));
                    self.fatal = true;
                    self.ready = false;
                }
            }
        }
    }
    fn sidebar(&mut self, ctx: &Context) {
        SidePanel::left("navigation")
            .exact_width(206.0)
            .resizable(false)
            .frame(
                Frame::new()
                    .fill(PANEL)
                    .inner_margin(Margin::symmetric(18, 24)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
                    for y in 0..3 {
                        for x in 0..3 {
                            ui.painter().rect_filled(
                                Rect::from_min_size(
                                    r.min + vec2(x as f32 * 9.0, y as f32 * 9.0),
                                    vec2(6.0, 6.0),
                                ),
                                2,
                                MINT,
                            );
                        }
                    }
                    ui.label(RichText::new("CONTROL\nDECK").strong().size(17.0));
                });
                ui.add_space(8.0);
                eyebrow(ui, "STUDIO 3 / NATIVE RUST");
                ui.add_space(32.0);
                for (page, name) in [
                    (Page::Deck, "Πλήκτρα"),
                    (Page::Display, "Οθόνη OLED"),
                    (Page::Home, "Home Assistant"),
                    (Page::Activity, "Δραστηριότητα"),
                    (Page::Settings, "Ρυθμίσεις"),
                ] {
                    let active = self.page == page;
                    let response = ui.add_sized(
                        [170.0, 44.0],
                        Button::new(RichText::new(name).color(if active { MINT } else { MUTED }))
                            .fill(if active {
                                Color32::from_rgb(30, 49, 45)
                            } else {
                                PANEL
                            })
                            .stroke(if active {
                                stroke(1.0, Color32::from_rgb(49, 91, 77))
                            } else {
                                Stroke::NONE
                            })
                            .corner_radius(8),
                    );
                    if response.clicked() {
                        self.page = page;
                    }
                }
                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    muted(ui, "3.0.0 alpha · εγγενής εφαρμογή");
                    ui.add_space(8.0);
                    card().show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            vec2(128.0, 110.0),
                            Layout::top_down(Align::LEFT),
                            |ui| {
                                ui.set_min_width(128.0);
                                eyebrow(ui, "ΣΥΣΚΕΥΗ");
                                ui.label(
                                    RichText::new(if self.status.demo {
                                        "Προσομοίωση"
                                    } else if self.status.usb_connected {
                                        "USB συνδεδεμένο"
                                    } else {
                                        "USB εκτός σύνδεσης"
                                    })
                                    .color(
                                        if self.status.usb_connected {
                                            MINT
                                        } else {
                                            MUTED
                                        },
                                    ),
                                );
                                ui.small(if self.status.demo {
                                    "Χωρίς εξωτερικές ενέργειες"
                                } else {
                                    "ESP32-S3 · 9 πλήκτρα"
                                });
                            },
                        );
                    });
                });
            });
    }
    fn header(&mut self, ctx: &Context) {
        TopBottomPanel::top("header")
            .frame(
                Frame::new()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(26, 20)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        eyebrow(ui, "Ο ΔΙΚΟΣ ΣΟΥ ΧΩΡΟΣ ΕΛΕΓΧΟΥ");
                        ui.label(
                            RichText::new(match self.page {
                                Page::Deck => "Κάθε πλήκτρο, στα μέτρα σου.",
                                Page::Display => "Μια εικόνα για κάθε πάτημα.",
                                Page::Home => "Το σπίτι σου, σε ένα πλήκτρο.",
                                Page::Activity => "Κάθε ενέργεια, ξεκάθαρα.",
                                Page::Settings => "Ρύθμισε τον χώρο σου.",
                            })
                            .size(25.0)
                            .strong(),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if primary(ui, "Αποθήκευση", self.ready && self.dirty()).clicked()
                        {
                            self.save();
                        }
                        if ui
                            .add_enabled(self.dirty(), Button::new("Αναίρεση"))
                            .clicked()
                        {
                            self.confirm = Some(Confirm::Revert);
                        }
                        ui.label(
                            RichText::new(if self.dirty() {
                                "Μη αποθηκευμένες αλλαγές"
                            } else {
                                "Αποθηκευμένο"
                            })
                            .size(13.0)
                            .color(if self.dirty() {
                                tint(3)
                            } else {
                                MUTED
                            }),
                        );
                    });
                });
            });
    }
    fn footer(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("status")
            .frame(
                Frame::new()
                    .fill(PANEL)
                    .inner_margin(Margin::symmetric(22, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if self.status.demo {
                            "DEMO"
                        } else if self.status.paused {
                            "ΣΕ ΠΑΥΣΗ"
                        } else {
                            "NATIVE RUNTIME"
                        })
                        .color(if self.status.paused { tint(3) } else { MINT })
                        .strong(),
                    );
                    ui.separator();
                    let p = self.saved.active();
                    ui.label(format!("Ενεργό mode: {}", p.name));
                    ui.separator();
                    ui.small(format!("{} / 72 είσοδοι", self.status.input.registered));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                self.ready,
                                Button::new(if self.status.paused {
                                    "Συνέχιση macros"
                                } else {
                                    "Παύση macros"
                                }),
                            )
                            .clicked()
                        {
                            self.send(Command::Pause(!self.status.paused));
                        }
                        if self.status.io_busy {
                            ui.spinner();
                            ui.small("Εργασία σε εξέλιξη");
                        }
                        if self.status.running_jobs > 0 {
                            ui.spinner();
                            ui.label(format!("{} ακολουθίες", self.status.running_jobs));
                        }
                    });
                });
                if let Some((message, error)) = self.notice.clone() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(&message)
                                .color(if error { RED } else { MINT })
                                .size(14.0),
                        );
                        if ui.small_button("Κλείσιμο μηνύματος").clicked() {
                            self.notice = None;
                        }
                    });
                }
            });
    }
    fn confirmation(&mut self, ctx: &Context) {
        if let Some(kind) = self.confirm {
            Window::new("Επιβεβαίωση αλλαγής").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER,Vec2::ZERO).show(ctx,|ui|{ui.set_max_width(420.0);ui.label(match kind{Confirm::Revert=>"Να απορριφθούν οι μη αποθηκευμένες αλλαγές των πλήκτρων;",Confirm::Import=>"Η εισαγωγή θα αντικαταστήσει το draft των πλήκτρων. Να συνεχίσει;",Confirm::DeleteProfile=>"Να αφαιρεθεί αυτό το προφίλ από το draft; Έλεγξε τυχόν ενέργειες που δείχνουν σε αυτό.",Confirm::Close=>"Υπάρχουν μη αποθηκευμένες αλλαγές σε πλήκτρα ή γραφικά. Να κλείσει η εφαρμογή χωρίς αποθήκευση;"});ui.horizontal(|ui|{if ui.button("Ακύρωση").clicked(){self.confirm=None;}
if primary(ui,"Συνέχεια",true).clicked(){match kind{Confirm::Revert=>self.draft=self.saved.clone(),Confirm::Import=>self.send(Command::ImportConfig),Confirm::DeleteProfile=>{self.draft.profiles.remove(self.profile);self.profile=0;},Confirm::Close=>{self.allow_close=true;ctx.send_viewport_cmd(ViewportCommand::Close);}}self.confirm=None;}});});
        }
    }
    fn add_profile(&mut self, duplicate: bool) {
        if self.draft.profiles.len() >= 12 {
            return;
        }
        let mut p = self.profile().clone();
        let mut n = 1;
        while self
            .draft
            .profiles
            .iter()
            .any(|p| p.id == format!("custom-{n}"))
        {
            n += 1;
        }
        p.id = format!("custom-{n}");
        p.name = if duplicate {
            format!("{} copy", p.name.chars().take(40).collect::<String>())
        } else {
            format!("Mode {}", self.draft.profiles.len() + 1)
        };
        p.oled_name.clear();
        if !duplicate {
            for k in &mut p.keys {
                k.label = "Χωρίς ενέργεια".into();
                k.actions.clear();
            }
        }
        self.draft.profiles.push(p);
        self.profile = self.draft.profiles.len() - 1;
    }
}
impl eframe::App for Studio {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.render(ctx);
    }
}
impl Studio {
    fn render(&mut self, ctx: &Context) {
        self.events();
        if self.page != Page::Display {
            self.playing = false;
        }
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allow_close
            && (self.dirty() || self.art_dirty())
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.confirm = Some(Confirm::Close);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::S)) && self.ready {
            if self.page == Page::Display {
                self.send(Command::SaveArtwork(self.art.clone()));
            } else {
                self.save();
            }
        }
        self.sidebar(ctx);
        self.header(ctx);
        self.footer(ctx);
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(26, 18)),
            )
            .show(ctx, |ui| {
                if !self.ready {
                    ui.heading(if self.fatal {
                        "Δεν ξεκίνησε η υπηρεσία."
                    } else {
                        "Φόρτωση τοπικών ρυθμίσεων…"
                    });
                    if !self.fatal {
                        ui.spinner();
                    }
                    return;
                }
                match self.page {
                    Page::Deck => self.deck_page(ui),
                    Page::Display => self.display_page(ui),
                    Page::Home => self.home_page(ui),
                    Page::Activity => self.activity_page(ui),
                    Page::Settings => self.settings_page(ui),
                }
            });
        self.confirmation(ctx);
    }
}

#[cfg(test)]
mod headless;
