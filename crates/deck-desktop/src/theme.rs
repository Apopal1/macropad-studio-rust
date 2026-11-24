use eframe::egui::*;
pub const BG: Color32 = Color32::from_rgb(17, 20, 25);
pub const PANEL: Color32 = Color32::from_rgb(23, 27, 33);
pub const CARD: Color32 = Color32::from_rgb(30, 35, 42);
pub const LINE: Color32 = Color32::from_rgb(48, 56, 65);
pub const TEXT: Color32 = Color32::from_rgb(234, 240, 245);
pub const MUTED: Color32 = Color32::from_rgb(156, 168, 181);
pub const MINT: Color32 = Color32::from_rgb(112, 232, 190);
pub const RED: Color32 = Color32::from_rgb(255, 151, 151);
pub fn tint(i: u8) -> Color32 {
    match i {
        1 => Color32::from_rgb(142, 177, 255),
        2 => Color32::from_rgb(197, 161, 255),
        3 => Color32::from_rgb(248, 197, 128),
        4 => Color32::from_rgb(244, 150, 191),
        _ => MINT,
    }
}
pub fn setup(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "deck".into(),
        FontData::from_static(include_bytes!("../../../assets/DejaVuSans.ttf")).into(),
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "deck".into());
    ctx.set_fonts(fonts);
    let mut v = Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = BG;
    v.faint_bg_color = CARD;
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = Color32::from_rgb(43, 85, 74);
    v.selection.stroke = stroke(1.5, MINT);
    v.widgets.inactive.bg_fill = CARD;
    v.widgets.inactive.weak_bg_fill = CARD;
    v.widgets.inactive.fg_stroke = stroke(1.0, TEXT);
    v.widgets.inactive.bg_stroke = stroke(1.0, LINE);
    v.widgets.hovered.bg_fill = Color32::from_rgb(43, 52, 59);
    v.widgets.hovered.weak_bg_fill = v.widgets.hovered.bg_fill;
    v.widgets.hovered.bg_stroke = stroke(1.0, MINT);
    v.widgets.active.bg_fill = Color32::from_rgb(40, 77, 67);
    v.widgets.active.bg_stroke = stroke(1.5, MINT);
    v.widgets.noninteractive.bg_stroke = stroke(1.0, LINE);
    v.window_stroke = stroke(1.0, LINE);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.text_styles
            .insert(TextStyle::Body, FontId::proportional(16.0));
        s.text_styles
            .insert(TextStyle::Button, FontId::proportional(15.0));
        s.text_styles
            .insert(TextStyle::Small, FontId::proportional(13.0));
        s.text_styles
            .insert(TextStyle::Heading, FontId::proportional(25.0));
        s.spacing.item_spacing = vec2(10.0, 12.0);
        s.spacing.button_padding = vec2(14.0, 10.0);
        s.spacing.interact_size = vec2(40.0, 36.0);
        s.animation_time = 0.0;
    });
}
pub fn eyebrow(ui: &mut Ui, s: &str) {
    ui.label(RichText::new(s).size(12.0).color(MUTED).strong());
}
pub fn muted(ui: &mut Ui, s: impl Into<String>) {
    ui.label(RichText::new(s.into()).color(MUTED));
}
pub fn primary(ui: &mut Ui, s: &str, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        Button::new(RichText::new(s).color(BG).strong())
            .fill(MINT)
            .corner_radius(7),
    )
}
pub fn card() -> Frame {
    Frame::new()
        .fill(CARD)
        .stroke(stroke(1.0, LINE))
        .corner_radius(12)
        .inner_margin(18)
}

pub fn stroke(width: f32, color: Color32) -> Stroke {
    Stroke::new(width, color)
}
