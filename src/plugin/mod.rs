use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use eframe::egui::{
    Align, Color32, ColorImage, Context, Layout, RichText, Sense, TextureHandle, TextureOptions,
    Ui, Vec2,
};
use eframe::epaint::ahash::HashMap;
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use tracing::debug;
use zip::ZipArchive;

use splinter_animation::{AnimationManager, Lerp};
use splinter_icon::icon;

use crate::colors;
use crate::icons::draw_icon;

pub mod manager;

const PLUGIN_HEIGHT: f32 = 32.0;
pub type PluginId = String;

pub struct Plugin {
    pub id: PluginId,
    pub version: String,
    pub name: String,

    pub icon: Option<TextureHandle>,

    /// The stability makes the plugin be less often split,
    /// Plugins that are stable are libraries which are often present in a mod configuration and are known to be quite stable.
    pub stability: u32,
    pub forced_status: Option<bool>,


    pub status: Status,
    pub depends_on: Vec<String>,
}

impl Plugin {
    pub fn new(path: PathBuf, ctx: &Context) -> Plugin {
        debug!("Loading mod at {path:?}");
        let file = File::open(path.as_path()).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut file = archive.by_name("fabric.mod.json").unwrap();

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let json: FabricModJson = serde_json::from_slice(&data).unwrap();
        drop(file);
        let icon = json
            .icon
            .map(|v| Self::load_icon(&mut archive, &v, ctx))
            .unwrap_or(None);

        Plugin {
            id: json.id,
            version: json.version,
            name: json.name,
            icon,
            stability: 0,
            forced_status: None,
            status: Status::Enabled,
            depends_on: json.depends.unwrap_or_default().into_keys().collect(),
        }
    }

    fn load_icon(
        archive: &mut ZipArchive<File>,
        icon: &str,
        ctx: &Context,
    ) -> Option<TextureHandle> {
        let mut file = archive.by_name(icon).ok()?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).ok()?;
        let mut image = image::load_from_memory(&data).ok()?;
        if image.width() > 32 || image.height() > 32 {
            image = image.resize_exact(
                PLUGIN_HEIGHT as u32,
                PLUGIN_HEIGHT as u32,
                FilterType::Lanczos3,
            );
        }

        Some(
            ctx.load_texture(
                icon,
                ColorImage {
                    size: [image.width() as usize, image.height() as usize],
                    pixels: image
                        .to_rgba8()
                        .pixels()
                        .map(|v| Color32::from_rgba_premultiplied(v.0[0], v.0[1], v.0[2], v.0[3]))
                        .collect(),
                },
                TextureOptions::NEAREST,
            ),
        )
    }

    pub fn ui(&mut self, ui: &mut Ui, animation: &AnimationManager) {
        let mut vec2 = ui.available_size_before_wrap();
        vec2.y = PLUGIN_HEIGHT;
        let enabled = self.enabled();
        ui.allocate_ui_with_layout(vec2, Layout::left_to_right(Align::Center), |ui| {
            let response = ui.interact(ui.max_rect(), ui.next_auto_id(), Sense::click_and_drag());
            if response.clicked() {
                self.forced_status = match self.forced_status {
                    None => Some(false),
                    Some(false) => Some(true),
                    Some(true) => None,
                };
            }



            let mut enable_animation = animation.get(response.id);
            if enabled {
                enable_animation.redirect(1.0);
            } else {
                enable_animation.redirect(0.0);
            }
            let enabled = enable_animation.get();
            let mut stroke = (0.0, Color32::TRANSPARENT);
            if response.hovered() {
                stroke = (2.0, colors::HIGHLIGHT);
            }
            let bg = colors::BG_1.lerp(&colors::BG_2, enabled);
            let fg = colors::FG_4.lerp(&colors::FG_2, enabled);

            ui.set_min_size(ui.available_size_before_wrap());
            ui.painter().rect(ui.max_rect(), 8.0, bg, stroke);

            if let Some(icon) = self.icon.as_ref() {
                let response = ui.image(icon.id(), Vec2::new(PLUGIN_HEIGHT, PLUGIN_HEIGHT));
                ui.painter().rect_filled(
                    response.rect,
                    0.0,
                    Color32::from_rgba_premultiplied(0, 0, 0, ((1.0 - enabled) * 128.0) as u8),
                )
            } else {
                ui.add_space(PLUGIN_HEIGHT);
            }

            ui.add_space(8.0);
            ui.label(RichText::new(&self.name).color(fg).size(16.0));

            ui.allocate_ui_with_layout(
                ui.available_size_before_wrap(),
                Layout::right_to_left(Align::Center),
                |ui| {
                    ui.add_space(8.0);

                    {
                        let force_transition = animation
                            .get(response.id.with("lock"))
                            .redirect(self.forced_status.is_some() as u8 as f32)
                            .get();

                        // if force_transition == 0.0 {
                        let lock_enabled = self.forced_status.unwrap_or(true);
                        ui.add_space((16.0 + 6.0) * force_transition);
                        let color = fg.gamma_multiply(force_transition);
                        let text_response = ui.label(
                            RichText::new(format!(
                                "Force-{}",
                                if lock_enabled { "enabled" } else { "disabled" }
                            ))
                            .color(color)
                            .size(16.0),
                        );
                        draw_icon(
                            ui.painter(),
                            if force_transition != 1.0 {
                                icon!("lock_open")
                            } else {
                                icon!("lock")
                            },
                            text_response.rect.right_center() + Vec2::new(8.0 + 4.0, 0.0),
                            16.0,
                            color,
                        );
                        // }
                    }
                },
            );
        });
        ui.add_space(8.0);
    }

    pub fn enabled(&self) -> bool {
        self.forced_status.unwrap_or(self.status.enabled())
    }

    pub fn should_split(&self) -> bool {
        self.forced_status.is_none() && self.status.enabled()
    }
}

#[derive(Serialize, Deserialize)]
pub struct FabricModJson {
    id: String,
    version: String,
    name: String,
    icon: Option<String>,
    depends: Option<HashMap<String, String>>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Status {
    Enabled ,
    Disabled,
    NotTheProblem,
}

impl Status {
    pub fn enabled(&self) -> bool {
        match self {
            Status::Disabled | Status::NotTheProblem => false,
            Status::Enabled => true,
        }
    }
}
