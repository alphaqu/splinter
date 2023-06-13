#![feature(let_chains)]

use std::path::PathBuf;

use eframe::egui::{
    Align, Color32, Context, Direction, Frame, Id, Label, Layout, Margin, RichText, Rounding,
    Sense, Style, Ui, Vec2, Visuals, Widget,
};
use eframe::{egui, App, NativeOptions};
use rfd::FileDialog;
use tracing::level_filters::LevelFilter;

use splinter_animation::config::AnimationConfig;
use splinter_animation::{AnimationManager, Lerp};
use splinter_icon::icon;

use crate::icons::{draw_icon, load_fonts};
use crate::plugin::manager::PluginManager;
use crate::prog::spinner::ProgressSpinner;
use crate::prog::Progress;

mod colors;
mod icons;
mod plugin;
mod prog;
mod progress;

const HEADER_SIZE: f32 = 48.0;
fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    eframe::run_native(
        "Splinter",
        NativeOptions {
            initial_window_size: Some(Vec2::new(1000.0, 600.0)),
            fullscreen: false,
            maximized: false,
            ..NativeOptions::default()
        },
        Box::new(|cc| {

            //cc.egui_ctx.set_pixels_per_point(3.0);
            cc.egui_ctx.set_fonts(load_fonts());
            let mut visuals = Visuals::dark();
            visuals.override_text_color = Some(Color32::WHITE);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 30);
            visuals.widgets.inactive.rounding = Rounding::same(8.0);
            let mut style = Style::default();
            style.visuals = visuals;
            style.spacing.item_spacing = Vec2::splat(0.0);
            cc.egui_ctx.set_style(style);
            let mut application = Application {
                path: Some(PathBuf::from(
                    "/home/alphasucks/.local/share/multimc/instances/affinity-smp/.minecraft/mods/",
                )),
                animation: AnimationManager::new(AnimationConfig {
                    animation_speed: 0.25,
                }),
                plugin: PluginManager::new(),
            };

            Box::new(application)
        }),
    );
}

pub struct Application {
    path: Option<PathBuf>,
    animation: AnimationManager,
    plugin: PluginManager,
}

impl Application {
    fn layout_button(
        ui: &mut Ui,
        animation: &AnimationManager,
        icon: u32,
        fg: Color32,
        bg: Color32,
        func: impl FnOnce(),
    ) {
        let id = ui.next_auto_id();

        ui.allocate_ui_with_layout(
            Vec2::new(HEADER_SIZE, HEADER_SIZE),
            Layout::centered_and_justified(Direction::BottomUp),
            |ui| {
                let response =
                    ui.interact(ui.max_rect(), id.with("button"), Sense::click_and_drag());
                ui.set_min_size(ui.available_size_before_wrap());
                let rect = ui.max_rect();
                let painter = ui.painter();

                painter.rect_filled(
                    rect,
                    8.0,
                    animation
                        .get(response.id)
                        .redirect_with_speed(
                            if !ui.is_enabled() {
                                bg.lerp(&colors::BG_1, 0.4)
                            } else if response.hovered() {
                                bg.lerp(&fg, 0.10)
                            } else {
                                bg
                            },
                            0.2,
                        )
                        .get(),
                );
                draw_icon(painter, icon, rect.center(), 24.0, fg);

                if response.clicked() {
                    func();
                }
            },
        );
    }
}
impl App for Application {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        //ScanTask::run(&mut self.scan, &mut self.plugins, ctx);
        self.animation.tick(ctx);
        egui::CentralPanel::default()
            .frame(
                Frame::central_panel(&ctx.style())
                    .inner_margin(Margin::same(0.0))
                    .outer_margin(Margin::same(8.0))
                    .fill(colors::BG_0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Buttons
                        ui.add_enabled_ui(self.plugin.is_ready(), |ui| {
                            Self::layout_button(
                                ui,
                                &self.animation,
                                icon!("cancel"),
                                Color32::from_rgb(200, 50, 50),
                                Color32::from_rgb(30, 15, 15),
                                || {
                                    self.plugin.invert();
                                    self.plugin.split();
                                },
                            );
                            ui.add_space(4.0);
                            Self::layout_button(
                                ui,
                                &self.animation,
                                icon!("bug_report"),
                                Color32::from_rgb(50, 200, 50),
                                Color32::from_rgb(10, 30, 10),
                                || {
                                    self.plugin.split();
                                },
                            );
                            ui.add_space(4.0);
                            ui.add_enabled_ui(self.plugin.can_redo(), |ui| {
                                Self::layout_button(
                                    ui,
                                    &self.animation,
                                    icon!("redo"),
                                    Color32::from_rgb(200, 200, 200),
                                    Color32::from_rgb(25, 25, 25),
                                    || {
                                        self.plugin.redo();
                                    },
                                );
                                ui.add_space(4.0);
                            });
                            ui.add_enabled_ui(self.plugin.can_undo(), |ui| {
                                Self::layout_button(
                                    ui,
                                    &self.animation,
                                    icon!("undo"),
                                    Color32::from_rgb(200, 200, 200),
                                    Color32::from_rgb(25, 25, 25),
                                    || {
                                        self.plugin.undo();
                                    },
                                );
                                ui.add_space(4.0);
                            });
                        });

                        let mut size = ui.available_size_before_wrap();
                        size.y = HEADER_SIZE;

                        // Main file path bar
                        ui.allocate_ui_with_layout(
                            size,
                            Layout::left_to_right(Align::Center),
                            |ui| {
                                let mut rect = ui.max_rect();
                                ui.set_clip_rect(rect);
                                let painter = ui.painter();
                                let response = ui.interact(
                                    rect,
                                    Id::new("header-path"),
                                    Sense::click_and_drag(),
                                );

                                painter.rect(
                                    rect,
                                    8.0,
                                    Color32::from_rgb(20, 20, 20),
                                    if response.hovered() {
                                        (2.0, colors::HIGHLIGHT)
                                    } else {
                                        (0.0, Color32::TRANSPARENT)
                                    },
                                );

                                ui.add_space(8.0);
                                rect.min.x += 8.0;
                                let icon = rect.left_center() + Vec2::new(16.0, 0.0);

                                let painter = ui.painter();
                                draw_icon(
                                    painter,
                                    icon!("folder_open"),
                                    icon,
                                    24.0,
                                    Color32::from_rgb(200, 200, 200),
                                );
                                ui.add_space(24.0 + 12.0);
                                Label::new(
                                    RichText::new(if let Some(path) = &self.path {
                                        path.to_str().unwrap_or("")
                                    } else {
                                        "Open mods directory..."
                                    })
                                    .color(Color32::from_rgb(200, 200, 200))
                                    .size(20.0),
                                )
                                .wrap(true)
                                .ui(ui);

                                if response.clicked() {
                                    if let Some(path) = FileDialog::new()
                                        .set_directory(
                                            self.path.as_ref().unwrap_or(&PathBuf::from("/")),
                                        )
                                        .pick_folder()
                                    {
                                        self.path = Some(path.clone());
                                        self.plugin.start_scan(path, ctx);
                                    }
                                };

                                // Progress bar
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add_space(16.0);
                                    let mut animation =
                                        self.animation.get::<f32>(Id::new("progress-bar"));

                                    let progress = self.plugin.progress();
                                    if progress.is_some() {
                                        animation.redirect_with_speed(3.0, 2.0);
                                    } else {
                                        animation.redirect_with_speed(0.0, 2.0);
                                    }

                                    ProgressSpinner {
                                        radius: 24.0,
                                        width: animation.get(),
                                        rounded: true,
                                        progress: Progress {
                                            progress,
                                            speed: 1.0,
                                            color: Color32::WHITE,
                                            track_color: Color32::TRANSPARENT,
                                        },
                                    }
                                    .ui(ui);
                                });
                            },
                        );
                    });
                });
                ui.add_space(16.0);
                self.plugin.ui(ui, &self.animation);
            });
        self.animation.end_tick(ctx);
    }
}
