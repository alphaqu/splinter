use std::fs::read_dir;
use std::path::PathBuf;

use eframe::egui::{Align, Align2, Color32, FontId, FontSelection, Label, Layout, RichText, ScrollArea, Sense, Style, Ui, Vec2, Widget};
use eframe::egui::text::LayoutJob;
use splinter_event::EventTracker;
use splinter_icon::icon;

use crate::{ApplicationState, ModpackEvent};
use crate::ui::color;
use crate::ui::icon::Icon;

pub struct HomeView {
    suggested_instances: Vec<PathBuf>,
    tracker: EventTracker,
}

impl HomeView {
    pub fn new() -> HomeView {
        let mut suggested_instances = vec![];

        if let Some(path) = dirs::data_dir() {
            let buf = path.join("multimc/instances");
            if let Ok(value) = read_dir(buf) {
                for entry in value.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if path.join(".minecraft/mods").exists() {
                            suggested_instances.push(path);
                        }
                    }
                }
            }
        }

        // Sort by last used
        suggested_instances.sort_by(|v0, v1| {
            let v0 = v0.metadata().ok().and_then(|v| v.modified().ok());
            let v1 = v1.metadata().ok().and_then(|v| v.modified().ok());
            v1.cmp(&v0)
        });

        // Clean up paths
        let home = dirs::home_dir();
        if let Some(home) = home {
            for path in &mut suggested_instances {
                if let Ok(new) = path.strip_prefix(&home) {
                    *path = PathBuf::from("~").join(new);
                }
            }
        }

        HomeView {
            suggested_instances,
            tracker: EventTracker::new(),
        }
    }

    pub fn ui(&mut self, state: &mut ApplicationState, ui: &mut Ui) {
        let mut commander = self.tracker.tick(&mut state.events);
        ScrollArea::vertical().show(ui, |ui| {
            ui.set_min_size(ui.available_size_before_wrap());
            ui.allocate_ui_at_rect(ui.available_rect_before_wrap().shrink(32.0), |ui| {
                ui.add_space(12.0);
                ui.label(
                    RichText::new("Welcome to Splinter!")
                        .color(color::TEXT)
                        .strong()
                        .size(40.0),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Binary searching your problems away")
                        .color(color::SUBTEXT0)
                        .strong()
                        .size(20.0),
                );
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    if !self.suggested_instances.is_empty() {
                        ui.vertical(|ui| {
                            ui.add_space(32.0);
                            ui.allocate_ui_with_layout(
                                Vec2::new(0.0, 32.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    Icon::new(icon!("history"), 24.0, color::TEXT).ui(ui);
                                    ui.add_space(6.0);
                                    ui.label(
                                        RichText::new("Recover session".to_string())
                                            .color(color::TEXT)
                                            .strong()
                                            .size(24.0),
                                    );
                                },
                            );

                            ui.add_space(8.0);
                            for path in &self.suggested_instances {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(ui.available_rect_before_wrap().width() / 2.0, 24.0),
                                    Layout::left_to_right(Align::Center),
                                    |ui| {
                                        ui.set_min_size(ui.available_size_before_wrap());
                                        let rect = ui.min_rect();
                                        let response = ui.interact(
                                            rect,
                                            ui.next_auto_id(),
                                            Sense::click_and_drag(),
                                        );

                                        if response.clicked() {
                                            commander.dispatch(ModpackEvent::Load(path.clone()));
                                        }
                                        let style = Style::default();
                                        let mut layout_job = LayoutJob::default();
                                        let mut text = RichText::new(format!("{path:?}"))
                                            .color(color::BLUE)
                                            .strong()
                                            .size(16.0);

                                        if response.hovered() {
                                            text = text.color(color::SKY);
                                            text = text.underline();
                                        }
                                        text.append_to(&mut layout_job, &style, FontSelection::Default, Align::Center);
                                        
                                        let painter = ui.painter();
                                        let galley = painter.layout_job(layout_job);
                                        painter.galley(rect.left_top(), galley, Color32::RED);
                                    },
                                );
                                ui.add_space(6.0);
                            }
                        });
                    }
                });
            });
        });
    }
}
