use std::sync::Arc;

use eframe::egui::{
	FontData, FontDefinitions, FontFamily, FontTweak, Id, Ui,
};

use splinter_animation::AnimationManager;

pub mod color;
pub mod icon;
pub mod progress;

#[derive(Debug)]
pub struct NotificationEvent {
    pub title: String,
    pub description: String,
    pub ty: Severity,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

pub fn animation(ui: &mut Ui) -> AnimationManager {
    ui.data_mut(|d| d.get_persisted::<AnimationManager>(Id::NULL))
        .unwrap()
}

#[derive(Copy, Clone, Debug)]
pub enum ProgressStatus {
    Indeterminate,
    Determinate(f32),
}

pub fn load_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::empty();
    add_font(
        &mut fonts,
        FontData::from_static(include_bytes!("../../assets/Icons.ttf")).tweak(FontTweak {
            scale: 1.0,
            y_offset_factor: 0.0,
            y_offset: 0.0,
            baseline_offset_factor: 0.0,
        }),
        "Icons",
    );
    add_font(
        &mut fonts,
        FontData::from_static(include_bytes!("../../assets/Mukta-Regular.ttf")).tweak(FontTweak {
            scale: 1.0,
            y_offset_factor: 0.0,
            y_offset: 0.0,
            baseline_offset_factor: -0.04,
        }),
        "Roboto-Regular",
    );

    add_font(
        &mut fonts,
        FontData::from_static(include_bytes!("../../assets/Mukta-SemiBold.ttf")).tweak(FontTweak {
            scale: 1.0,
            y_offset_factor: 0.0,
            y_offset: 0.0,
            baseline_offset_factor: -0.01,
        }),
        "Roboto-Bold",
    );

    fonts.families.insert(
        FontFamily::Proportional,
        vec!["Roboto-Regular".to_string(), "Icons".to_string()],
    );
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
