use eframe::egui::text::LayoutJob;
use eframe::egui::{
    Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Response, Sense, Ui, Vec2, Widget,
};

pub fn draw_icon(painter: &Painter, icon: u32, pos: Pos2, size: f32, color: Color32) {
    let icon = char::from_u32(icon).expect("Could not parse icon char");
    let text = icon.to_string();

    let font_id = FontId::new(size, FontFamily::Name("Icons".into()));
    let job = LayoutJob::simple(text, font_id.clone(), color, f32::INFINITY);
    let arc = painter.ctx().fonts(|fonts| fonts.layout_job(job));

    let mut rect = Align2::CENTER_CENTER.anchor_rect(Rect::from_min_size(pos, arc.rect.size()));

    let data = painter.ctx().fonts(|fonts| fonts.glyph_width(&font_id, icon));

    rect.set_height(data);
    // + Vec2::new(0.0, size * 0.1)
    painter.galley(rect.min,arc, color);
}

pub struct Icon {
    icon: u32,
    size: f32,
    color: Color32,
}

impl Icon {
    pub fn new(icon: u32, size: f32, color: Color32) -> Icon {
        Icon { icon, size, color }
    }
}

impl Widget for Icon {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(self.size, self.size), Sense::hover());

       //ui.painter().rect_stroke(rect, 0.0, (1.0, Color32::RED));
       //ui.painter().rect_stroke(rect, 1000.0, (1.0, Color32::RED));

       //ui.painter().line_segment([
       //                              rect.center_top(),
       //                              rect.center_bottom(),
       //                          ], (1.0 ,Color32::BLUE));
       //ui.painter().line_segment([
       //                              rect.right_center(),
       //                              rect.left_center(),
       //                          ], (1.0 ,Color32::BLUE));
        let pos2 = rect.center();
        draw_icon(
            ui.painter(),
            self.icon,
            pos2,
            self.size,
            self.color,
        );
        response
    }
}
