use std::os::linux::raw::stat;
use std::path::PathBuf;
use dirs::home_dir;
use eframe::egui::{Align, Align2, Color32, Direction, FontFamily, FontId, Id, Layout, Mesh, Pos2, Rect, Response, Sense, Spinner, Ui, Vec2, Widget};
use eframe::epaint::Vertex;
use rfd::FileDialog;
use tracing::info;

use splinter_animation::{AnimationImpl,Lerp};
use splinter_event::{ EventSystem, EventTracker};
use splinter_icon::icon;
use crate::{ApplicationState, ModpackEvent, ModpackStatus};
use crate::data::{Modpack, ModpackOperationEvent};
use crate::ui::icon::Icon;
use crate::ui::progress::{Progress, ProgressSpinner, ProgressSystem};
use crate::ui::{animation, color, progress, ProgressStatus, UiSystem};

#[derive(Debug)]
pub struct ProgressEvent(pub Option<ProgressStatus>);

pub struct Header {
    events: EventTracker,
    progress: Option<ProgressStatus>,
}

impl Header {
    pub fn new() -> Header {
        Header  {
            events: EventTracker::new(),
            progress: None,
        }
    }
    pub fn ui(&mut self, state: &mut ApplicationState, ui: &mut Ui) {
        let mut commander = self.events.tick(&mut state.events);
        for event in commander.consume::<ProgressEvent>() {
            self.progress = event.0;
        }

        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let modpack = &mut state.modpack_status;
                let is_ready = match modpack {
                    ModpackStatus::Empty => false,
                    ModpackStatus::Active { is_loaded, .. } => {
                        *is_loaded
                    }
                };

                HeaderEntry::button(
                    ui,
                    is_ready,
                    icon!("cancel"),
                    color::RED.gamma_multiply(0.2),
                    color::RED,
                    "If your issue does no longer persist, \nyou may press this to swap the enabled mods.",
                    || {
                        commander.dispatch(ModpackOperationEvent::Invert);
                    },
                );
                ui.add_space(4.0);

                HeaderEntry::button(
                    ui,
                    is_ready,
                    icon!("bug_report"),
                    color::GREEN.gamma_multiply(0.2),
                    color::GREEN,
                    "If your issue does persist, \npressing this will disable half of the mods.",

                    || {
                        commander.dispatch(ModpackOperationEvent::Split);
                    },
                );
                ui.add_space(4.0);

                HeaderEntry::button(
                    ui,
                    match modpack {
                        ModpackStatus::Empty => false,
                        ModpackStatus::Active { is_loaded, can_redo, .. } => {
                            *is_loaded && *can_redo
                        }
                    },
                    icon!("redo"),
                    color::MANTLE,
                    color::SUBTEXT1,
                    "Redoes your last operation",
                    || {
                        commander.dispatch(ModpackOperationEvent::Redo);
                    },
                );
                ui.add_space(4.0);

                HeaderEntry::button(
                    ui,
                    match modpack {
                        ModpackStatus::Empty => false,
                        ModpackStatus::Active { is_loaded, can_undo, .. } => {
                            *is_loaded && *can_undo
                        }
                    },
                    icon!("undo"),
                    color::MANTLE,
                    color::SUBTEXT1,
                    "Undoes your last operation",
                    || {
                        commander.dispatch(ModpackOperationEvent::Undo);
                    },
                );
                ui.add_space(4.0);

                let path = match modpack {
                    ModpackStatus::Empty => None,
                    ModpackStatus::Active { path, .. } => {
                        Some(path.clone())
                    }
                };
                let ctx = ui.ctx().clone();
                HeaderEntry::path(
                    ui,
                    self.progress,
                    icon!("folder_open"),
                    if let Some(path) = &path {
                        path.to_str().unwrap_or("")
                    } else {
                        "Navigate to..."
                    }
                        .to_string(),
                    || {
                        let option = FileDialog::new()
                            .set_directory(
                                path.unwrap_or(home_dir().unwrap_or(PathBuf::from("/"))),
                            )
                            .pick_folder();
                        println!("{option:?}");
                        if let Some(path) = option
                        {
                            self.progress = Some(ProgressStatus::Indeterminate);
                            commander.dispatch(ModpackEvent::Load(path));
                        }
                    },
                );
                ui.add_space(4.0);
                HeaderEntry::button(
                    ui,
                    match modpack {
                        ModpackStatus::Empty => false,
                        ModpackStatus::Active { .. } => {
                            true
                        }
                    },
                    icon!("home"),
                    color::MANTLE,
                    color::SUBTEXT1,
                    "Exit the current session.",
                    || {
                        commander.dispatch(ModpackEvent::Exit);
                    },
                );
            });
        });
    }
}

pub const HEADER_HEIGHT: f32 = 48.0;
pub const HEADER_PADDING: f32 = 8.0;

pub struct HeaderEntry;

impl HeaderEntry {
    //pub fn ui(
    //         ui: &mut Ui,
    //         button: Option<bool>,
    //         system: &UiSystem,
    //         icon: u32,
    //         color: Color32,
    //         func: impl FnOnce(&mut Ui, Color32, Response),
    //     ) {
    //         let desired_size = if button.is_some() {
    //             Vec2::new(HEADER_HEIGHT, HEADER_HEIGHT)
    //         } else {
    //             ui.available_size_before_wrap()
    //         };
    //         ui.allocate_ui_with_layout(
    //             desired_size,
    //             if button.is_some() {
    //                 Layout::centered_and_justified(Direction::BottomUp)
    //             } else {
    //                 Layout::left_to_right(Align::Center)
    //             },
    //             |ui| {
    //                 if let Some(status) = button {
    //                     ui.set_enabled(status);
    //                 }
    //                 ui.set_max_size(desired_size);
    //                 ui.set_clip_rect(ui.max_rect());
    //                 let id = ui.next_auto_id();
    //                 let response = ui.interact(
    //                     ui.max_rect(),
    //                     id.with("header-entry"),
    //                     Sense::click_and_drag(),
    //                 );
    //
    //                 let rect = ui.max_rect();
    //                 let painter = ui.painter();
    //
    //                 let bg = color::CRUST.lerp(
    //                     &color,
    //                     system
    //                         .animation()
    //                         .get(response.id)
    //                         .redirect_with_speed(
    //                             if !ui.is_enabled() {
    //                                 0.10
    //                             } else if response.hovered() {
    //                                 0.35
    //                             } else {
    //                                 0.2
    //                             },
    //                             0.2,
    //                         )
    //                         .get(),
    //                 );
    //                 painter.rect_filled(rect, 8.0, bg);
    //
    //                 if button.is_none() {
    //                     ui.add_space(12.0);
    //                 }
    //                 Icon::new(icon, 24.0, color).ui(ui);
    //                 if button.is_none() {
    //                     ui.add_space(8.0);
    //                 }
    //                 func(ui, bg, response);
    //             },
    //         );
    //     }

    pub fn path(
        ui: &mut Ui,
        progress: Option<ProgressStatus>,
        icon: u32,
        text: String,
        func: impl FnOnce(),
    ) {
        let mut desired_size = ui.available_size_before_wrap();
        desired_size.x -= HEADER_HEIGHT + 4.0;
        ui.allocate_ui_with_layout(desired_size, Layout::left_to_right(Align::Center), |ui| {
            let animation = animation(ui);

            ui.set_max_size(desired_size);
            ui.set_clip_rect(ui.max_rect());
            let id = ui.next_auto_id();
            let response = ui.interact(
                ui.max_rect(),
                id.with("header-entry"),
                Sense::click_and_drag(),
            );

            let rect = ui.max_rect();
            let painter = ui.painter();

            let bg = color::MANTLE.lerp(
                &color::PANEL,
                animation
                    .get(response.id)
                    .redirect_with_speed(if response.hovered() { 0.8 } else { 0.0 }, 0.2)
                    .get(),
            );
            painter.rect_filled(rect, 8.0, bg);
            ui.add_space(16.0);
          //  Icon::new(icon, 24.0, color::SUBTEXT1).ui(ui);
           // ui.add_space(10.0);
            let painter = ui.painter();
            let galley = painter.layout_no_wrap(
                text.to_string(),
                FontId::new(20.0, FontFamily::Proportional),
                color::SUBTEXT1,
            );
            let mut rect = ui.available_rect_before_wrap();
            // padding
            rect.max.x -= 40.0;
            let overflowing = galley.size().x > rect.size().x;
            let text_pos = if overflowing {
                Align2::RIGHT_CENTER
                    .anchor_rect(Rect::from_min_size(rect.right_center(), galley.size()))
                    .min
            } else {
                Align2::LEFT_CENTER
                    .anchor_rect(Rect::from_min_size(rect.left_center(), galley.size()))
                    .min
            };
            let overflowing = animation
                .get_or(response.id.with("overflow"), || AnimationImpl::simple(1.0))
                .redirect_with_speed(overflowing as u8 as f32, 0.5)
                .get();
            painter.with_clip_rect(rect).galley(text_pos, galley);

            //   if overflowing != 0.0 {
            let size = rect.size();
            let rect = Rect::from_min_size(rect.min, Vec2::new(50.0f32.min(size.x), size.y));
            // Gradient
            let mut mesh = Mesh::default();
            let idx = mesh.vertices.len() as u32;
            mesh.add_triangle(idx + 0, idx + 1, idx + 2);
            mesh.add_triangle(idx + 2, idx + 1, idx + 3);

            mesh.vertices.push(Vertex {
                pos: rect.left_top(),
                uv: Pos2::ZERO,
                color: bg.gamma_multiply(overflowing),
            });
            mesh.vertices.push(Vertex {
                pos: rect.right_top(),
                uv: Pos2::ZERO,
                color: bg.gamma_multiply(0.0),
            });
            mesh.vertices.push(Vertex {
                pos: rect.left_bottom(),
                uv: Pos2::ZERO,
                color: bg.gamma_multiply(overflowing),
            });
            mesh.vertices.push(Vertex {
                pos: rect.right_bottom(),
                uv: Pos2::ZERO,
                color: bg.gamma_multiply(0.0),
            });
            painter.add(mesh);

            ui.allocate_ui_with_layout(ui.available_size_before_wrap(), Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(12.0);
                let (progress, enabled) = match progress {
                    Some(progress) => {
                        (match progress {
                            ProgressStatus::Determinate(progress) => {
                                Some(progress)
                            }

                            ProgressStatus::Indeterminate => {
                                None
                            }
                        }, true)
                    }
                    None => {
                        (None, false)
                    }
                };

                let enabled = animation.get(Id::new("progress")).redirect(enabled as u8 as f32).get();
                ProgressSpinner {
                    radius: 24.0,
                    width: 3.0 * enabled,
                    rounded: true,
                    progress: Progress {
                        progress,
                        speed: 1.0,
                        color: color::SUBTEXT1,
                        track_color: Color32::TRANSPARENT,
                    },
                }.ui(ui);
            });

            if response.clicked() {
                func();
            }
        });
    }
    pub fn button(
        ui: &mut Ui,
        enabled: bool,
        icon: u32,
        bg: Color32,
        fg: Color32,
        tooltip: &str,
        func: impl FnOnce(),
    ) {
        let animation = animation(ui);
        let desired_size = Vec2::new(HEADER_HEIGHT, HEADER_HEIGHT);
        ui.allocate_ui_with_layout(
            desired_size,
            Layout::centered_and_justified(Direction::BottomUp),
            |ui| {
                ui.set_enabled(enabled);
                ui.set_max_size(desired_size);
                ui.set_clip_rect(ui.max_rect());
                let id = ui.next_auto_id();
                let response = ui.interact(
                    ui.max_rect(),
                    id.with("header-entry"),
                    Sense::click_and_drag(),
                );
                response.clone().on_hover_text(tooltip);

                let rect = ui.max_rect();
                let painter = ui.painter();

                let bg = if !enabled {
                    bg.lerp(&color::BACKGROUND, 0.75)
                } else if response.hovered() {
                    bg.lerp(&color::PANEL, 0.8)
                } else {
                    bg
                };
                /*let bg = color::CRUST.lerp(
                    &bg,
                    animation
                        .get(response.id)
                        .redirect_with_speed(
                            if enabled {
                                1.0
                            } else if !enabled {

                            } else if response.hovered() {
                                0.25
                            } else {
                                0.15
                            },
                            0.2,
                        )
                        .get(),
                );*/
                painter.rect_filled(rect, 8.0, bg);
                Icon::new(
                    icon,
                    24.0,
                    if !enabled {
                        fg.lerp(&color::TEXT, 0.5)
                    } else if response.hovered() {
                        fg
                    } else {
                        fg.lerp(&color::BRIGHT, 0.2)
                    },
                )
                .ui(ui);
                if response.clicked() {
                    func();
                }
            },
        );
    }
}
