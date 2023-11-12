use std::fs::read_dir;
use std::path::PathBuf;

use eframe::egui::{Align, Color32, Context, Frame, Id, Layout, Margin, RichText, Rounding, ScrollArea, Sense, Style, Vec2, Visuals, Widget};
use eframe::{egui, App, NativeOptions};
use rfd::FileDialog;
use tracing::level_filters::LevelFilter;
use tracing::warn;

use splinter_animation::{AnimationManager, Lerp};
use splinter_animation::config::AnimationConfig;
use splinter_event::{EventSystem, EventTracker};
use splinter_icon::icon;
use crate::ApplicationView::{Home, Search};

use crate::data::{Modpack, PluginStatus};
use crate::ui::{animation, color, load_fonts, NotificationEvent, ProgressStatus, UiSystem};
use crate::ui::icon::Icon;
use crate::view::{Header, HeaderEntry};
use crate::view::home::HomeView;
use crate::view::search::SearchView;

pub mod view;
mod data;
mod ui;

#[derive(Debug)]
pub struct LoadPathEvent(pub PathBuf);
#[derive(Debug)]
pub struct PackStatusEvent {
    pub is_loaded: bool
}

#[derive(Debug)]
pub enum PackOperationEvent {
    Undo,
    Redo,
    Split,
    Invert
}

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
            let ctx = &cc.egui_ctx;
            ctx.data_mut(|v| {
                v.insert_persisted(Id::null(), AnimationManager::new(AnimationConfig {
                    animation_speed: 0.3,
                }));
            });

            let mut style = Style::default();
            let mut visuals = Visuals::dark();
            visuals.override_text_color = Some(color::TEXT);
            visuals.panel_fill = Color32::from_rgb(30, 30, 30);
            visuals.window_fill = Color32::from_rgb(30, 30, 30);

            visuals.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 30);
            visuals.widgets.inactive.rounding = Rounding::same(8.0);
            style.visuals = visuals;
            style.spacing.item_spacing = Vec2::splat(0.0);
            ctx.set_style(style);
            ctx.set_fonts(load_fonts());

            Box::new(Application {
                header: Header::new(),
                view: Home(HomeView::new()),
                state: ApplicationState {
                    modpack_status: ModpackStatus::Empty,
                    events: EventSystem::new(),
                },
                tracker: EventTracker::new(),
            })
        }),
    )
    .unwrap();
}

pub struct ApplicationState {
    modpack_status: ModpackStatus,
    events: EventSystem
}

pub enum ModpackStatus {
    Empty,
    Active {
        path: PathBuf,
        is_loaded: bool,
        can_undo: bool,
        can_redo: bool,
    }
}

#[derive(Debug)]
pub enum ModpackEvent {
    Load(PathBuf),
    Exit
}


pub enum ApplicationView {
    Search(SearchView),
    Home(HomeView)
}

pub struct Application {
    header: Header,
    view: ApplicationView,
    state: ApplicationState,

    tracker: EventTracker,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let commander = self.tracker.tick(&mut self.state.events);
        for event in commander.consume::<ModpackEvent>() {
            match event {
                ModpackEvent::Load(path) => {
                    if let Some(value) = SearchView::new(path.clone(), ctx) {
                        self.view = Search(value);
                    }
                }
                ModpackEvent::Exit => {
                    // We want to re-enable the plugins
                    match &mut self.view {
                        Search(search) => {
                            for plugin in search.modpack.plugins_mut().iter_mut() {
                                if plugin.forced_status.is_none() {
                                    plugin.status = PluginStatus::Enabled;
                                    plugin.push_changes();
                                }
                            }
                        }
                        Home(_) => {}
                    }
                    self.state.modpack_status = ModpackStatus::Empty;
                    self.view = Home(HomeView::new());
                }
            }
        }

        egui::CentralPanel::default()
            .frame(
                Frame::central_panel(&ctx.style())
                    .inner_margin(Margin::same(8.0))
                    .outer_margin(Margin::same(0.0))
                    .fill(color::BACKGROUND),
            )
            .show(ctx, |ui| {
                animation(ui).tick(ui.ctx());
                self.header.ui(&mut self.state, ui);
                match &mut self.view {
                    Search(view) => view.ui(&mut self.state, ui),
                    Home(view) => view.ui(&mut self.state, ui),
                }
                animation(ui).end_tick(ui.ctx());

                //ui.horizontal(|ui| {
                //                     ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                //                         let is_ready = self
                //                             .modpack
                //                             .as_ref()
                //                             .map(|v| !v.is_loading())
                //                             .unwrap_or(false);
                //
                //                         HeaderEntry::button(
                //                             ui,
                //                             is_ready,
                //                             icon!("cancel"),
                //                             color::RED,
                //                             || {
                //                                 if let Some(value) = &mut self.modpack {
                //                                     value.invert();
                //                                 };
                //                             },
                //                         );
                //                         ui.add_space(4.0);
                //
                //                         HeaderEntry::button(
                //                             ui,
                //                             is_ready,
                //                             icon!("bug_report"),
                //                             color::GREEN,
                //                             || {
                //                                 if let Some(value) = &mut self.modpack {
                //                                     value.split();
                //                                 };
                //                             },
                //                         );
                //                         ui.add_space(4.0);
                //
                //                         HeaderEntry::button(
                //                             ui,
                //                             self.modpack
                //                                 .as_ref()
                //                                 .map(|v| !v.is_loading() && v.can_redo())
                //                                 .unwrap_or(false),
                //                             icon!("redo"),
                //                             color::SUBTEXT1,
                //                             || {
                //                                 if let Some(value) = &mut self.modpack {
                //                                     value.redo();
                //                                 };
                //                             },
                //                         );
                //                         ui.add_space(4.0);
                //
                //                         HeaderEntry::button(
                //                             ui,
                //                             self.modpack
                //                                 .as_ref()
                //                                 .map(|v| !v.is_loading() && v.can_undo())
                //                                 .unwrap_or(false),
                //                             icon!("undo"),
                //                             color::SUBTEXT1,
                //                             || {
                //                                 if let Some(value) = &mut self.modpack {
                //                                     value.undo();
                //                                 };
                //                             },
                //                         );
                //                         ui.add_space(4.0);
                //
                //                         let ctx = ui.ctx().clone();
                //                         HeaderEntry::path(
                //                             ui,
                //                             &mut self.ui,
                //                             icon!("folder_open"),
                //                             color::SUBTEXT1,
                //                             if let Some(path) = &self.path {
                //                                 path.to_str().unwrap_or("")
                //                             } else {
                //                                 "Open mods directory..."
                //                             }
                //                             .to_string(),
                //                             |ui| {
                //                                 if let Some(path) = FileDialog::new()
                //                                     .set_directory(
                //                                         self.path.as_ref().unwrap_or(&PathBuf::from("/")),
                //                                     )
                //                                     .pick_folder()
                //                                 {
                //                                     ui.set_progress(Some(ProgressStatus::Indeterminate));
                //
                //                                     self.path = Some(path.clone());
                //                                     let modpack = Modpack::new(path, &ctx);
                //                                     if modpack.is_none() {
                //                                         warn!("Modpack is none");
                //                                     }
                //                                     self.modpack = modpack;
                //                                 }
                //                             },
                //                         );
                //                     });
                //                 });
                //
                //                 if let Some(modpack) = &mut self.modpack {
                //                     ui.add_space(12.0);
                //                     modpack.ui(ui, &mut self.ui);
                //                 } else {
                //                     ScrollArea::vertical().show(ui, |ui| {
                //                         ui.set_min_size(ui.available_size_before_wrap());
                //                         ui.allocate_ui_at_rect(
                //                             ui.available_rect_before_wrap().shrink(32.0),
                //                             |ui| {
                //                                 ui.add_space(12.0);
                //                                 ui.label(
                //                                     RichText::new("Welcome to Splinter!")
                //                                         .color(color::TEXT)
                //                                         .strong()
                //                                         .size(40.0),
                //                                 );
                //                                 ui.add_space(8.0);
                //                                 ui.label(
                //                                     RichText::new("Binary searching your problems away")
                //                                         .color(color::SUBTEXT0)
                //                                         .strong()
                //                                         .size(20.0),
                //                                 );
                //                                 ui.add_space(4.0);
                //                                 if false {
                //                                     ui.add_space(32.0);
                //                                     ui.allocate_ui_with_layout(
                //                                         Vec2::new(0.0, 32.0),
                //                                         Layout::left_to_right(Align::Center),
                //                                         |ui| {
                //                                             Icon::new(icon!("history"), 24.0, color::SUBTEXT1).ui(ui);
                //                                             ui.add_space(6.0);
                //                                             ui.label(
                //                                                 RichText::new(format!("Recover session"))
                //                                                     .color(color::SUBTEXT1)
                //                                                     .strong()
                //                                                     .size(24.0),
                //                                             );
                //                                         },
                //                                     );
                //                                 }
                //
                //                                 if !self.suggested_instances.is_empty() {
                //                                     ui.add_space(32.0);
                //                                     ui.allocate_ui_with_layout(
                //                                         Vec2::new(0.0, 32.0),
                //                                         Layout::left_to_right(Align::Center),
                //                                         |ui| {
                //                                             Icon::new(icon!("sort"), 24.0, color::TEXT).ui(ui);
                //                                             ui.add_space(6.0);
                //                                             ui.label(
                //                                                 RichText::new(format!("Suggested instances"))
                //                                                     .color(color::TEXT)
                //                                                     .strong()
                //                                                     .size(24.0),
                //                                             );
                //                                         },
                //                                     );
                //
                //                                     ui.add_space(8.0);
                //                                     for path in &self.suggested_instances {
                //                                         ui.allocate_ui_with_layout(
                //                                             Vec2::new(ui.available_rect_before_wrap().width(), 24.0),
                //                                             Layout::left_to_right(Align::Center),
                //                                             |ui| {
                //                                                 ui.set_min_size(ui.available_size_before_wrap());
                //                                                 let response = ui.interact(ui.min_rect(), ui.next_auto_id(), Sense::click_and_drag());
                //
                //                                                 if response.hovered() {
                //
                //                                                 }
                //                                                 //ui.painter().rect_filled(
                //                                                 //    ui.max_rect(),
                //                                                 //    8.0,
                //                                                 //    color::BASE,
                //                                                 //);
                //                                                 let mut text = RichText::new(format!("{path:?}"))
                //                                                     .color( color::BLUE)
                //                                                     .strong()
                //                                                     .size(16.0);
                //
                //                                                 if response.hovered() {
                //                                                     text = text.color(color::SKY);
                //                                                     text = text.underline();
                //                                                 }
                //                                                 ui.label(
                //                                                     text,
                //                                                 );
                //                                             },
                //                                         );
                //                                         ui.add_space(6.0);
                //                                     }
                //                                 }
                //                             },
                //                         );
                //                     });
                //                 }
            });

    }
}
