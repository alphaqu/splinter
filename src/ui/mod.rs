pub mod color;
pub mod icon;
pub mod progress;
use std::sync::Arc;

use eframe::egui::{Color32, Context, FontData, FontDefinitions, FontFamily, FontTweak, Id, Rounding, Style, Ui, Vec2, Visuals};

use splinter_animation::config::AnimationConfig;
use splinter_animation::AnimationManager;

#[derive(Debug)]
pub struct NotificationEvent {
	pub title: String,
	pub description: String,
	pub ty: Severity,
}

#[derive(Debug)]
pub enum Severity {
	Info,
	Warning,
	Error,
}

pub struct UiSystem {
	animation: AnimationManager,
	progress: Option<ProgressStatus>,
}

impl UiSystem {
	pub fn new(ctx: &Context) -> UiSystem {
		let mut style = Style::default();
		let mut visuals = Visuals::dark();
		visuals.override_text_color = Some(color::TEXT);
		visuals.panel_fill = color::CRUST;
		visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 30);
		visuals.widgets.inactive.rounding = Rounding::same(8.0);
		style.visuals = visuals;
		style.spacing.item_spacing = Vec2::splat(0.0);
		ctx.set_style(style);
		ctx.set_fonts(load_fonts());
		UiSystem {
			animation: AnimationManager::new(AnimationConfig {
				animation_speed: 0.3,
			}),
			progress: None,
		}
	}

	pub fn tick(&mut self, ctx: &Context) {
		self.animation.tick(ctx);
	}

	pub fn animation(&self) -> &AnimationManager {
		&self.animation
	}

	pub fn notify(&mut self, notification: NotificationEvent) {
		todo!()
	}

	pub fn set_progress(&mut self, progress: Option<ProgressStatus>) {
		self.progress = progress;
	}

	pub fn end_tick(&mut self, ctx: &Context) {
		self.animation.end_tick(ctx);
	}
}


pub fn animation(ui: &mut Ui) -> AnimationManager {
	ui.data_mut(|d| {
		d.get_persisted::<AnimationManager>(Id::null())
	}).unwrap()
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

	fonts
		.families
		.insert(FontFamily::Proportional, vec!["Roboto-Regular".to_string(), "Icons".to_string()]);
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
