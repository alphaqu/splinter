use egui::{Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use palette::convert::FromColorUnclamped;
use palette::{Lab, Srgb};
use std::any::Any;
use std::ops::Range;

pub trait Lerp: PartialEq + Clone + Any {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        Self::lerp_static(self, to, t)
    }
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self {
        ((v1 - v0) * t) + v0
    }
}

impl Lerp for Pos2 {
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self {
        (((*v1 - *v0) * t) + v0.to_vec2()).to_pos2()
    }
}

impl Lerp for Vec2 {
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self {
        ((*v1 - *v0) * t) + *v0
    }
}

impl Lerp for Rounding {
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self {
        Rounding {
            nw: v0.nw.lerp(&v1.nw, t),
            ne: v0.ne.lerp(&v1.ne, t),
            sw: v0.sw.lerp(&v1.sw, t),
            se: v0.se.lerp(&v1.se, t),
        }
    }
}

impl Lerp for Rect {
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self {
        let size = Vec2::lerp_static(&v0.size(), &v1.size(), t);
        let center = Vec2::lerp_static(&v0.center().to_vec2(), &v1.center().to_vec2(), t);
        Rect::from_center_size(center.to_pos2(), size)
    }
}

impl Lerp for Color32 {
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self {
        let v0_l = lab(*v0);
        let v1_l = lab(*v1);
        from_lab(
            Lab {
                l: v0_l.l.lerp(&v1_l.l, t),
                a: v0_l.a.lerp(&v1_l.a, t),
                b: v0_l.b.lerp(&v1_l.b, t),
                white_point: Default::default(),
            },
            ((v0.a() as f32 / 255.0).lerp(&(v1.a() as f32 / 255.0), t) * 255.0) as u8,
        )
    }
}

impl Lerp for Stroke {
    fn lerp_static(v0: &Self, v1: &Self, t: f32) -> Self {
        Stroke {
            width: Lerp::lerp_static(&v0.width, &v1.width, t),
            color: Lerp::lerp_static(&v0.color, &v1.color, t),
        }
    }
}

pub fn extend(range: Range<f32>, t: f32) -> f32 {
    let t = t - range.start;
    let t = t / (range.end - range.start);
    t
}

fn lab(color: Color32) -> Lab {
    let rgb = Srgb::new(color.r(), color.g(), color.b());
    let rgb: Srgb<f32> = rgb.into_format();
    Lab::from_color_unclamped(rgb)
}

fn from_lab(lab: Lab, a: u8) -> Color32 {
    let rgb2 = Srgb::from_color_unclamped(lab);
    let rgb1: Srgb<u8> = rgb2.into_format();
    Color32::from_rgba_premultiplied(rgb1.red, rgb1.green, rgb1.blue, a)
}
