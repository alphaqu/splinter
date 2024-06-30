use std::path::PathBuf;

use eframe::egui::{Color32, Context, Frame, Id, Margin, Rounding, Style, Vec2, ViewportBuilder, Visuals};
use eframe::{egui, App, NativeOptions};
use tracing::level_filters::LevelFilter;

use splinter_animation::config::AnimationConfig;
use splinter_animation::{AnimationManager};
use splinter_event::{EventSystem, EventTracker};

use crate::data::PluginStatus;
use crate::ui::{animation, color, load_fonts};
use crate::view::home::HomeView;
use crate::view::search::SearchView;
use crate::view::Header;
use crate::ApplicationView::{Home, Search};

mod data;
mod ui;
pub mod view;

#[derive(Debug)]
pub struct LoadPathEvent(pub PathBuf);
#[derive(Debug)]
pub struct PackStatusEvent {
    pub is_loaded: bool,
}

#[derive(Debug)]
pub enum PackOperationEvent {
    Undo,
    Redo,
    Split,
    Invert,
}

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    eframe::run_native(
        "Splinter",
        NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size(Vec2::new(1000.0, 600.0))
                .with_fullscreen(false)
                .with_maximized(false),
            ..NativeOptions::default()
        },
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;
            ctx.data_mut(|v| {
                v.insert_persisted(
                    Id::NULL,
                    AnimationManager::new(AnimationConfig {
                        animation_speed: 0.3,
                    }),
                );
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
    events: EventSystem,
}

pub enum ModpackStatus {
    Empty,
    Active {
        path: PathBuf,
        is_loaded: bool,
        can_undo: bool,
        can_redo: bool,
    },
}

#[derive(Debug)]
pub enum ModpackEvent {
    Load(PathBuf),
    Exit,
}

pub enum ApplicationView {
    Search(SearchView),
    Home(HomeView),
}

pub struct Application {
    header: Header,
    view: ApplicationView,
    state: ApplicationState,

    tracker: EventTracker,
}

impl App for Application {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
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
            });
    }
}
