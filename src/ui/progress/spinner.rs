use std::f32::consts::{PI, TAU};

use eframe::egui::{
    Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget,
};
use crate::ui::progress::Progress;


pub struct ProgressSpinner {
    /// The size of the spinner.
    pub radius: f32,
    pub width: f32,
    /// If the edge is rounded.
    pub rounded: bool,
    pub progress: Progress,
}

impl ProgressSpinner {
    fn draw_points(
        painter: &Painter,
        rect: Rect,
        start: f32,
        end: f32,
        radius: f32,
        width: f32,
        rounded_end: bool,
        color: Color32,
    ) {
        if color.a() != 0 {
            let rad_distance = (start - end).abs() / 2.0;
            let steps = ((rad_distance / PI) * SPINNER_RES as f32).max(2.0) as usize;
            let points: Vec<Pos2> = (0..=steps)
                .map(|i| {
                    let pos = i as f32 / steps as f32;
                    let angle = start + pos * (end - start);
                    let (sin, cos) = angle.sin_cos();
                    rect.center() + (((radius) / 2.0) - (width / 2.0)) * Vec2::new(cos, sin)
                })
                .collect();

            let first = points.first().cloned();
            let last = points.last().cloned();
            painter.add(Shape::line(points, Stroke::new(width, color)));

            if rounded_end {
                if let Some(value) = first {
                    painter.add(Shape::circle_filled(value, (width / 2.0) - 0.5, color));
                }
                if let Some(value) = last {
                    painter.add(Shape::circle_filled(value, (width / 2.0) - 0.5, color));
                }
            }
        }
    }
}

pub const SPINNER_RES: usize = 50;

impl Widget for ProgressSpinner {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.ctx().request_repaint();

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(self.radius, self.radius), Sense::hover());

        let [start, end] = self.progress.tick(ui.ctx(), response.id);
        let painter = ui.painter();

        ProgressSpinner::draw_points(
            painter,
            rect,
            (start * TAU) + TAU,
            end * TAU,
            self.radius,
            self.width,
            false,
            self.progress.track_color,
        );
        ProgressSpinner::draw_points(
            painter,
            rect,
            start * TAU,
            end * TAU,
            self.radius,
            self.width,
            self.rounded,
            self.progress.color,
        );

        response
    }
}
