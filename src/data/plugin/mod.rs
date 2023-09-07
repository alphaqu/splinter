use std::fs::{File, rename};
use std::io::Read;
use std::path::PathBuf;

use eframe::egui::{
	Align, Color32, ColorImage, Context, Layout, RichText, Sense, TextureHandle, TextureOptions,
	Ui, Vec2,
};
use image::imageops::FilterType;
use tracing::{debug, error, info, warn};
use zip::ZipArchive;

pub use metadata::PluginMetadata;
use splinter_animation::Lerp;
use splinter_icon::icon;
pub use status::PluginStatus;

use crate::ui::{animation, color};
use crate::ui::icon::Icon;

mod metadata;
mod status;

pub const PLUGIN_HEIGHT: f32 = 32.0;

pub struct Plugin {
    pub metadata: PluginMetadata,
    path: PathBuf,
    icon: Option<TextureHandle>,

    /// The stability makes the plugin be less often split,
    /// Plugins that are stable are libraries which are often present in a mod configuration and are known to be quite stable.
    pub stability: u32,
    pub forced_status: Option<bool>,

    pub status: PluginStatus,
    file_status: FileStatus,
}


#[derive(Copy, Clone)]
enum FileStatus {
    Enabled,
    ForceDisabled,
    Disabled
}
impl Plugin {
    pub fn new(path: PathBuf, ctx: &Context) -> Option<Plugin> {
        let extension = path.extension()?.to_str()?;
        warn!("{extension:?}");

        let status = match extension {
            "jar" => FileStatus::Enabled,
            "disabled" => FileStatus::ForceDisabled,
            "tempdisabled" => {
               if let Err(error) =  rename(&path, path.with_extension("")) {
                   error!("Failed to un-disable {error:?}");
                   FileStatus::Disabled
               } else {
                   FileStatus::Enabled
               }
            },
            _ => {
                info!("Unknown file extension \"{extension}\" in mods folder");
                return None;
            }
        };
        debug!("Loading mod at {path:?}");
        let file = File::open(path.as_path()).ok()?;
        let mut archive = ZipArchive::new(file).unwrap();

        let metadata = if let Some(metadata) = PluginMetadata::new(&mut archive) {
            metadata
        } else {
            warn!("Plugin at {path:?} does not have metadata");
            //state.notify(Notification {
            //    title: format!("Could not read plugin \"{path:?}\"",),
            //    description: "Plugin does not have a detectable metadata".to_string(),
            //    ty: Severity::Warning,
            //});
            return None;
        };

        Some(Plugin {
            icon: metadata
                .icon
                .as_ref()
                .and_then(|icon| Self::load_icon(&mut archive, icon, ctx)),
            metadata,
            stability: 0,
            forced_status: if matches!(status, FileStatus::ForceDisabled) { Some(false)} else  { None},
            status: match status {
                FileStatus::Enabled => PluginStatus::Enabled,
                FileStatus::Disabled |  FileStatus::ForceDisabled => PluginStatus::Disabled,
            },
            file_status: status,
            path,
        })
    }

    pub fn push_changes(&mut self) {
        let file = match self.file_status {
            FileStatus::Enabled => {
                let mut path = self.path.clone();
                let mut is_jar = false;
                while let Some(extension) =  path.extension() {
                    let extension = extension.to_str().unwrap();
                    if extension == "jar" {
                        is_jar = true;
                        break;
                    }
                    path = path.with_extension("");
                }

                if !is_jar {
                    warn!("Plugin at {:?} is no longer a jar.", self.path);
                    return;
                }

                path
            }
            FileStatus::Disabled | FileStatus::ForceDisabled => {
                self.path.with_extension("")
            }
        };

        let new_file = match self.forced_status {
            None => {
                match self.status {
                    PluginStatus::Enabled => {
                        file
                    }
                    PluginStatus::NotTheProblem | PluginStatus::Disabled => {
                        file.with_extension("jar.tempdisabled")
                    }
                }
            }
            Some(value) => {
                if value {
                    file
                } else {
                    file.with_extension("jar.disabled")
                }
            }
        };

        if new_file != self.path {
            match rename(&self.path, &new_file) {
                Ok(_) => {
                    self.path = new_file;
                }
                Err(err) => {
                    error!("Failed to rename file to {err}");
                }
            };
        }

    }

    pub fn should_split(&self) -> bool {
        self.forced_status.is_none() && self.status.enabled()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let mut vec2 = ui.available_size_before_wrap();
        vec2.y = PLUGIN_HEIGHT;
        let enabled = self.forced_status.unwrap_or(self.status.enabled());
        ui.allocate_ui_with_layout(vec2, Layout::left_to_right(Align::Center), |ui| {
            ui.add_space(4.0);
            let rect = ui.available_rect_before_wrap();

            let response = ui.interact(
                rect,
                ui.next_auto_id(),
                Sense::click_and_drag(),
            );
            if response.clicked() {
                // Cycle the forced status.
                self.forced_status = match self.forced_status {
                    None => Some(false),
                    Some(false) => Some(true),
                    Some(true) => None,
                };
                self.push_changes();
                // TODO check what mods are going to be broken.
            }

            let animation = animation(ui);
            let enabled = animation
                .get(response.id)
                .redirect(enabled as u8 as f32)
                .get();

            let mut stroke = (0.0, Color32::TRANSPARENT);
            if response.hovered() {
                stroke = (2.0, color::PANEL);
            }

            let bg = color::CRUST.lerp(&color::MANTLE, enabled * 0.8 + 0.2);
            let fg = color::SUBTEXT0.lerp(&color::TEXT, enabled);
            ui.set_min_size(rect.size());
            ui.painter().rect(rect, 8.0, bg, stroke);

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
            ui.label(RichText::new(&self.metadata.name).color(fg).size(18.0));

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

                        let lock_enabled = self.forced_status.unwrap_or(true);
                        ui.add_space(6.0 * force_transition);
                        let color = fg.gamma_multiply(force_transition);

                        ui.add(Icon::new(
                            if force_transition != 1.0 {
                                icon!("lock_open")
                            } else {
                                icon!("lock")
                            },
                            16.0,
                            color,
                        ));
                        ui.add_space(6.0 * force_transition);
                        ui.label(
                            RichText::new(format!(
                                "Force-{}",
                                if lock_enabled { "enabled" } else { "disabled" }
                            ))
                            .color(color)
                            .size(18.0),
                        );
                    }
                },
            );
        });
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
}
