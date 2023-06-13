use std::sync::Arc;

use eframe::egui::{Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, Painter, Pos2, Rect, Vec2};
use eframe::egui::text::LayoutJob;

pub fn draw_icon(painter: &Painter, icon: u32, pos: Pos2, size: f32, color: Color32) {
    let icon = char::from_u32(icon).expect("Could not parse icon char");
    let text = icon.to_string();
    let font_id = FontId::new(
        size,
        FontFamily::Name("Icons".into()),
    );
    let job = LayoutJob::simple(text, font_id, color, f32::INFINITY);
    let arc = painter.ctx().fonts(|fonts| {
        fonts.layout_job(job)
    });

    let mut vec2 = arc.rect.size();
    vec2.y -= size / 4.0;
    let rect = Align2::CENTER_CENTER.anchor_rect(Rect::from_min_size(pos, vec2));
    let pos2 = rect.min;
   // painter.rect_filled(rect, 0.0, Color32::RED);
    painter.galley(pos2, arc);
}

pub fn load_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::empty();
    add_font(&mut fonts, FontData::from_static(include_bytes!("./Icons2.ttf")), "Icons");
    add_font(&mut fonts, FontData::from_static(include_bytes!("./Roboto-Regular.ttf")), "Roboto-Regular");

    fonts
        .families
        .insert(FontFamily::Proportional, vec!["Roboto-Regular".to_string()]);
    fonts
        .families
        .insert(FontFamily::Monospace, vec!["Roboto-Regular".to_string()]);

    fonts
}

fn add_font(fonts: &mut FontDefinitions, font: FontData, name: &str) {
    fonts.font_data.insert(name.to_owned(), font);
    fonts.families.insert(
        FontFamily::Name(Arc::from(name)),
        vec![name.to_string(), "Roboto-Regular".to_string()],
    );
}