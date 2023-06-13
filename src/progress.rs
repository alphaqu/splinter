use std::f64::consts::PI;

use eframe::egui::{
    lerp, vec2, Color32, Id, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui,
};

use splinter_animation::AnimationManager;

const POINTS: i32 = 25;
const TRANSITION_POINTS: i32 = 100;
const LINE_WIDTH: f32 = 4.0;
const OVERFLOW: f32 = 0.5;

// A modified version of egui spinner to support progress.
pub struct LegacySpinner {
    progress: Option<f32>,
}

impl LegacySpinner {
    pub fn new(progress: Option<f32>) -> Self {
        LegacySpinner { progress }
    }

    fn draw_inner(
        painter: &Painter,
        rect: Rect,
        points: i32,
        start: f32,
        end: f32,
        line_width: f32,
        color: Color32,
    ) {
        let radius = (rect.height() / 2.0) - 2.0;
        let points: Vec<Pos2> = (0..points)
            .map(|i| {
                let angle = lerp(start..=end, i as f32 / points as f32);
                let (sin, cos) = angle.sin_cos();
                rect.center() + radius * vec2(cos as f32, sin as f32)
            })
            .collect();

        painter.add(Shape::line(points, Stroke::new(line_width, color)));
    }

    pub fn draw(self, ui: &mut Ui, animation: &AnimationManager, fg: Color32, rect: Rect, width: f32, id: Id) {
        let mut animation = animation.get::<f32>(id);
        animation.set_to(if self.progress.is_some() { 2.0f32 } else { 0.0 });
        let state = animation.get();

        let points = if state == 0.0 || state == 2.0 {
            POINTS
        } else {
            TRANSITION_POINTS
        };

        // Indeterminate
        if state != 2.0 {
            ui.ctx().request_repaint();

            let state = ((1.0 + OVERFLOW) - state.clamp(0.0, 1.0 + OVERFLOW)) / (1.0 + OVERFLOW);

            let time = ui.input(|state| state.time) * 1.5;
            let start = time * (PI * 2.0);
            let end = start + 260f64.to_radians() * (time / 2.0).sin();

            Self::draw_inner(
                ui.painter(),
                rect,
                points,
                start as f32,
                end as f32,
                width * state,
                fg,
            );
        }

        // Determinate
        if state != 0.0 {
            let state = (state.clamp(1.0 - OVERFLOW, 2.0) - (1.0 - OVERFLOW)) / (1.0 + OVERFLOW);
            let progress = self.progress.map(|v| v * state).unwrap_or(1.0);
            Self::draw_inner(
                ui.painter(),
                rect,
                points,
                0.0,
                (progress * 360.0).to_radians(),
                width * state,
                fg,
            );
        }
    }

    pub fn ui(self, ui: &mut Ui, animation: &AnimationManager, size: f32, width: f32, fg: Color32) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
        self.draw(ui, animation, fg, rect,width, response.id);
        response
    }
}
