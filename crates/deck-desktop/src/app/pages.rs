use super::*;
use deck_domain::{
    Action, ActionKind,
    artwork::{Frame as ArtFrame, pixel, set_pixel},
    oled_label,
};
impl Studio {
    pub(super) fn display_page(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().id_salt("display-page").show(ui,|ui|{
  ui.horizontal_wrapped(|ui|{ui.label("Γραφικό πλήκτρου");for id in 1..=9{if ui.selectable_label(self.key==id,format!("K{id}")).clicked(){self.key=id;self.frame=0;self.playing=false;}}});ui.add_space(12.0);
  ui.horizontal_wrapped(|ui|{if ui.button("Εισαγωγή εικόνας / GIF…").clicked(){self.send(Command::ImportImage(self.key));}
if primary(ui,"Αποθήκευση γραφικών",self.art_dirty()).clicked(){self.send(Command::SaveArtwork(self.art.clone()));}
if ui.add_enabled(!self.art_dirty()&&self.status.usb_connected&&!self.status.demo,Button::new(format!("Αποστολή K{} στο pad",self.key))).clicked(){self.send(Command::Upload{key:self.key,clip:self.art_saved.keys[self.key as usize-1].clone()});}});
  muted(ui,"Κοινό γραφικό ανά πλήκτρο για όλα τα modes · 128×64 · έως 24 καρέ. Η πάνω ζώνη δείχνει πάντα το mode.");ui.add_space(14.0);
  let count=self.art.keys[self.key as usize-1].frames.len();self.frame=self.frame.min(count-1);
  if self.playing{let duration=self.art.keys[self.key as usize-1].frames[self.frame].duration_ms as u64;if self.frame_started.elapsed().as_millis()>=duration as u128{self.frame=(self.frame+1)%count;self.frame_started=Instant::now();}ui.ctx().request_repaint_after(std::time::Duration::from_millis(duration));}
  card().show(ui,|ui|{let scale=((ui.available_width()-20.0)/128.0).clamp(3.0,6.0);let size=vec2(128.0*scale,64.0*scale);let(r,response)=ui.allocate_exact_size(size,Sense::click_and_drag());ui.painter().rect_filled(r,0,Color32::from_rgb(4,13,14));let b=&self.art.keys[self.key as usize-1].frames[self.frame].bitmap;
   for y in 14..64{for x in 0..128{if pixel(b,x,y){ui.painter().rect_filled(Rect::from_min_size(r.min+vec2(x as f32*scale,y as f32*scale),vec2(scale,scale)),0,MINT);}}}
   let header=Rect::from_min_size(r.min,vec2(size.x,14.0*scale));ui.painter().rect_filled(header,0,Color32::from_rgb(17,42,35));ui.painter().text(header.center(),Align2::CENTER_CENTER,format!("{}  {}",self.profile+1,oled_label(self.profile(),self.profile)),FontId::monospace(scale*2.8),MINT);
   if !self.playing&&(response.dragged()||response.clicked())&&let Some(pos)=response.interact_pointer_pos(){let x=((pos.x-r.min.x)/scale) as usize;let y=((pos.y-r.min.y)/scale) as usize;let erase=self.eraser||ui.input(|i|i.pointer.secondary_down());set_pixel(&mut self.art.keys[self.key as usize-1].frames[self.frame].bitmap,x,y,!erase);}
   response.on_hover_text("Σχεδίασε κάτω από την ένδειξη mode. Διάλεξε γόμα για διαγραφή.");
  });ui.add_space(12.0);
  ui.horizontal_wrapped(|ui|{if ui.button(if self.playing{"Παύση preview"}else{"Αναπαραγωγή preview"}).clicked(){self.playing= !self.playing;self.frame_started=Instant::now();}ui.selectable_value(&mut self.eraser,false,"Μολύβι");ui.selectable_value(&mut self.eraser,true,"Γόμα");ui.label(format!("Καρέ {} / {}",self.frame+1,count));let mut f=self.frame as u32;if ui.add(Slider::new(&mut f,0..=count as u32-1).show_value(false)).changed(){self.frame=f as usize;self.playing=false;}});
  ui.horizontal_wrapped(|ui|{let clip=&mut self.art.keys[self.key as usize-1];if ui.add_enabled(clip.frames.len()<24,Button::new("Αντιγραφή καρέ")).clicked(){let f=clip.frames[self.frame].clone();clip.frames.insert(self.frame+1,f);self.frame+=1;}
if ui.add_enabled(clip.frames.len()>1,Button::new("Διαγραφή καρέ")).clicked(){clip.frames.remove(self.frame);self.frame=self.frame.saturating_sub(1);}
if ui.button("Καθαρισμός καρέ").clicked(){clip.frames[self.frame]=ArtFrame::default();}ui.label("Διάρκεια");ui.add(Slider::new(&mut clip.frames[self.frame].duration_ms,125..=2000).step_by(125.0).suffix(" ms"));});
  ui.separator();ui.horizontal_wrapped(|ui|{if ui.button("Εξαγωγή artwork JSON…").clicked(){self.send(Command::ExportArtwork(self.art.clone()));}
if ui.button("Εισαγωγή artwork v3…").clicked(){if self.art_dirty(){self.notice=Some(("Αποθήκευσε ή αναίρεσε πρώτα τα γραφικά.".into(),true));}else{self.send(Command::ImportArtwork);}}
if ui.add_enabled(self.art_dirty(),Button::new("Αναίρεση γραφικών")).clicked(){self.art=self.art_saved.clone();self.frame=0;}});
  muted(ui,"Το GIF εισάγεται τοπικά στη Rust, χωρίς browser. Οι χρόνοι προσαρμόζονται στα βήματα 125 ms του firmware. Παλιές συλλογές OLED v2 χρειάζονται ξεχωριστό importer.");
 });
    }
    pub(super) fn home_page(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().id_salt("home-page").show(ui,|ui|{
  card().show(ui,|ui|{ui.heading("Σύνδεση Home Assistant");muted(ui,"Το HA είναι προαιρετικό. Τα Windows macros λειτουργούν ανεξάρτητα.");ui.horizontal(|ui|{ui.label("Διεύθυνση");ui.add(TextEdit::singleline(&mut self.ha_url).hint_text("http://192.168.1.50:8123").desired_width(360.0));});ui.horizontal(|ui|{ui.label("Access token");ui.add(TextEdit::singleline(&mut self.ha_token).password(true).desired_width(360.0));});ui.horizontal_wrapped(|ui|{if primary(ui,"Έλεγχος και σύνδεση",!self.status.demo&&!self.ha_token.is_empty()).clicked(){let token=std::mem::take(&mut self.ha_token);self.send(Command::ConnectHa{url:self.ha_url.clone(),token});}
if ui.add_enabled(self.status.ha_connected,Button::new("Ανανέωση καταλόγου")).clicked(){self.send(Command::RefreshHa);}
if ui.add_enabled(self.status.ha_connected,Button::new("Αποσύνδεση")).clicked(){self.send(Command::DisconnectHa);}ui.label(RichText::new(if self.status.ha_connected{"Συνδεδεμένο"}else{"Δεν υπάρχει σύνδεση"}).color(if self.status.ha_connected{MINT}else{MUTED}));});ui.small("HA profile → Security → Long-lived access tokens. Στα Windows το token αποθηκεύεται με DPAPI, χωριστά από τα προφίλ. Ο έλεγχος σύνδεσης δεν αλλάζει συσκευές.");});
  ui.add_space(12.0);ui.columns(2,|cols|{
   card().show(&mut cols[0],|ui|{ui.heading("Οντότητες");ui.add(TextEdit::singleline(&mut self.ha_search).hint_text("Αναζήτηση ονόματος ή entity_id").desired_width(f32::INFINITY));ui.label(format!("{} οντότητες",self.catalog.entities.len()));let query=self.ha_search.to_lowercase();let entities=self.catalog.entities.clone();ScrollArea::vertical().id_salt("entities").max_height(330.0).show(ui,|ui|{for e in entities.iter().filter(|e|format!("{} {}",e.id,e.name).to_lowercase().contains(&query)).take(500){if ui.add_sized([ui.available_width(),46.0],Button::new(format!("{}  ·  {}\n{}",e.name,e.state,e.id))).clicked(){let domain=e.id.split('.').next().unwrap_or("light");self.ha_service=match domain{"scene"|"script"=>format!("{domain}.turn_on"),"automation"=>"automation.trigger".into(),_=>format!("{domain}.toggle")};self.ha_data=serde_json::json!({"entity_id":e.id}).to_string();}}
if self.catalog.entities.is_empty(){muted(ui,"Σύνδεσε το HA για να εμφανιστούν οι δικές σου συσκευές.");}});});
   card().show(&mut cols[1],|ui|{ui.heading("Ανάθεση ενέργειας");let mut profile=self.profile;ComboBox::from_id_salt("ha-profile").selected_text(self.profile().name.clone()).show_ui(ui,|ui|{for(i,p)in self.draft.profiles.iter().enumerate(){ui.selectable_value(&mut profile,i,p.name.clone());}});self.profile=profile;ComboBox::from_id_salt("ha-key").selected_text(format!("K{}",self.key)).show_ui(ui,|ui|{for k in 1..=9{ui.selectable_value(&mut self.key,k,format!("K{k}"));}});ui.label("Ενέργεια HA");ui.add(TextEdit::singleline(&mut self.ha_service).desired_width(f32::INFINITY));if !self.catalog.services.is_empty(){ComboBox::from_id_salt("ha-services").selected_text("Διαθέσιμες ενέργειες").show_ui(ui,|ui|{for s in &self.catalog.services{ui.selectable_value(&mut self.ha_service,s.id.clone(),format!("{} · {}",s.id,s.name));}});}ui.label("Παράμετροι JSON");ui.add(TextEdit::multiline(&mut self.ha_data).code_editor().desired_rows(5).desired_width(f32::INFINITY));let a=Action{kind:ActionKind::HomeAssistant,value:self.ha_service.clone(),args:self.ha_data.clone()};if let Err(e)=a.validate(){ui.label(RichText::new(e.to_string()).color(RED).size(13.0));}
if primary(ui,"Προσθήκη στο πλήκτρο",a.validate().is_ok()).clicked(){let k=self.draft.profiles[self.profile].keys.iter_mut().find(|k|k.id==self.key).unwrap();if k.actions.len()<12{k.actions.push(a);k.tint=3;self.notice=Some(("Η ενέργεια προστέθηκε στο draft. Αποθήκευσε για ενεργοποίηση.".into(),false));}else{self.notice=Some(("Το πλήκτρο έχει ήδη 12 βήματα.".into(),true));}}ui.small("Μπορείς να αναθέσεις light.turn_off με {\"entity_id\":\"all\"}, σκηνές ή scripts. Η προσθήκη δεν εκτελεί την ενέργεια.");});
  });
 });
    }
    pub(super) fn activity_page(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Ζωντανή δραστηριότητα");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Καθαρισμός προβολής").clicked() {
                    self.activity.clear();
                }
            });
        });
        muted(
            ui,
            "Έως 200 εγγραφές στη μνήμη. Χωρίς tokens, περιεχόμενο κειμένου ή διαδρομές συσκευής.",
        );
        ui.add_space(14.0);
        card().show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("activity-log")
                .show(ui, |ui| {
                    for a in self.activity.iter().rev() {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(&a.time).monospace().size(12.0).color(MUTED));
                            ui.label(RichText::new(&a.message).color(if a.error {
                                RED
                            } else {
                                TEXT
                            }));
                        });
                        ui.separator();
                    }
                    if self.activity.is_empty() {
                        muted(
                            ui,
                            "Οι πραγματικές ενέργειες και τα σφάλματα θα εμφανιστούν εδώ.",
                        );
                    }
                });
        });
    }
    pub(super) fn settings_page(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().id_salt("settings-page").show(ui,|ui|{
  ui.columns(2,|cols|{card().show(&mut cols[0],|ui|{ui.heading("Το mode σου");let p=&mut self.draft.profiles[self.profile];ui.label("Όνομα");ui.add(TextEdit::singleline(&mut p.name).char_limit(50).desired_width(f32::INFINITY));ui.label("Όνομα OLED · έως 16 λατινικοί χαρακτήρες");ui.add(TextEdit::singleline(&mut p.oled_name).char_limit(16).desired_width(f32::INFINITY));ui.small("Κενό: αυτόματη μεταγραφή του ονόματος. Τα πρώτα 8 προφίλ είναι hardware modes.");ui.horizontal_wrapped(|ui|{if ui.add_enabled(self.draft.profiles.len()<12,Button::new("Αντιγραφή mode")).clicked(){self.add_profile(true);}
if ui.add_enabled(self.draft.profiles.len()>1&&self.profile().id!=self.saved.active_profile,Button::new("Αφαίρεση mode")).clicked(){self.confirm=Some(Confirm::DeleteProfile);}});});
   card().show(&mut cols[1],|ui|{ui.heading("Αντίγραφα και μεταφορά");muted(ui,"Η εισαγωγή δέχεται τα προφίλ της προηγούμενης έκδοσης και της v3. Η παλιά εφαρμογή διατηρείται.");if ui.button("Εισαγωγή προφίλ JSON…").clicked(){if self.dirty(){self.confirm=Some(Confirm::Import);}else{self.send(Command::ImportConfig);}}
if ui.button("Εξαγωγή προφίλ JSON…").clicked(){self.send(Command::ExportConfig(self.draft.clone()));}ui.small("Τα tokens HA δεν περιλαμβάνονται ποτέ στην εξαγωγή. Τα config.previous.json / artwork.previous.json κρατούν το προηγούμενο αποθηκευμένο αρχείο.");});});
  ui.add_space(14.0);card().show(ui,|ui|{ui.heading("Σύνδεση και διάγνωση");ui.label(format!("USB: {}",self.status.usb_message));ui.label(format!("Firmware: {}",if self.status.firmware.is_empty(){"δεν έχει αναγνωριστεί"}else{&self.status.firmware}));ui.label(format!("Windows inputs: {} / 72 · {} συγκρούσεις · {} χαμένες είσοδοι",self.status.input.registered,self.status.input.conflicts.len(),self.status.input.dropped));if !self.status.input.conflicts.is_empty(){ui.label(RichText::new(format!("IDs σε σύγκρουση: {:?}",self.status.input.conflicts)).color(RED));}
if ui.add_enabled(!self.status.demo,Button::new("Επανέλεγχος USB")).clicked(){self.send(Command::RefreshDevice);}muted(ui,"Η ένδειξη USB αφορά το vendor HID της συσκευής, ενώ τα πατήματα Windows μπορούν να έρχονται μέσω USB ή Bluetooth. Κλείσε τον παλιό Studio bridge για να ελευθερωθούν τα F13–F21.");ui.separator();ui.label(format!("Δεδομένα v3: {}",self.status.data_dir));});
  ui.add_space(14.0);card().show(ui,|ui|{ui.heading("Εμφάνιση και έκδοση");let old=self.zoom;ui.add(Slider::new(&mut self.zoom,0.8..=1.3).text("Κλίμακα UI").step_by(0.05));if old!=self.zoom{ui.ctx().set_zoom_factor(self.zoom);}ui.label("Native Rust · egui/eframe · OpenGL renderer · 3.0.0-alpha.1");muted(ui,"Χωρίς browser, WebView ή localhost server. Η διεπαφή ανανεώνεται όταν χρειάζεται. Tray, autostart και installer παραμένουν στο roadmap.");ui.label(format!("Συνεδρία: {} s",self.started.elapsed().as_secs()));ui.small("Τα Windows APIs και το USB χρειάζονται τελική δοκιμή σε πραγματικά Windows/ESP32. Δεν γίνεται αυτόματο flash firmware.");});
 });
    }
}
