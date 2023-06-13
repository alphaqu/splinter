use std::collections::HashMap;
use std::fs::read_dir;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use crossbeam::channel::{unbounded, Receiver};
use eframe::egui::{Context, Label, RichText, ScrollArea, Ui};
use rand::prelude::SliceRandom;
use rand::{Rng, thread_rng};
use tracing::{debug, info, warn};

use splinter_animation::AnimationManager;
use crate::colors;

use crate::plugin::{Plugin, PluginId, Status};

pub struct PluginManager {
    screen_order: Vec<(Status, Vec<PluginId>)>,

    undo_queue: Vec<UndoState>,
    undo_queue_location: usize,

    plugins: HashMap<String, Plugin>,
    scan_task: Option<ScanTask>,
}

impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager {
            screen_order: Vec::new(),
            undo_queue: vec![],
            undo_queue_location: 0,
            plugins: Default::default(),
            scan_task: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, animation: &AnimationManager) {
        if let Some(task) = self.scan_task.as_mut() {
            ui.ctx().request_repaint();
            let mut update = false;
            while let Ok(plugin) = task.receiver.try_recv() {
                if self.plugins.insert(plugin.id.clone(), plugin).is_some() {
                    panic!("Duplicate id");
                }

                task.scanned += 1;
                update = true;
            }

            if task.scanned == task.count {
                self.scan_task = None;
                self.save_state();
            }

            if update {
                self.update_display_order();
            }
        }

        ScrollArea::vertical().show(ui, |ui| {
            for (status, plugins) in &self.screen_order {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.add(Label::new(
                        RichText::new(match status {
                            Status::Disabled => "Disabled Mods",
                            Status::Enabled => "Enabled Mods",
                            Status::NotTheProblem => "Not the problem!",
                        })
                            .color(colors::FG_2)
                        .size(24.0),
                    ));
                });
                ui.add_space(8.0);

                for id in plugins {
                    if let Some(plugin) = self.plugins.get_mut(id) {
                        plugin.ui(ui, animation);
                    } else {
                        panic!("Plugin does not exist");
                    }
                }
            }
        });
    }

    pub fn split(&mut self) {
        info!("Splitting");

        // Move the disabled plugins to the NotTheProblem status,
        // because we confirmed that this configuration still contains the issue.
        for plugin in self.plugins.values_mut() {
            if plugin.status == Status::Disabled {
                plugin.status = Status::NotTheProblem;
            }
        }

        let mut to_split = self.get_split_list();
        let original_len = to_split.len();
        let to_disable = to_split.len() / 2;
        let mut disabled = 0;

        let mut rng = rand::thread_rng();
        for i in 0..100 {
            info!(
                "Attempt {i} on splitting, {}/{} remaining.",
                disabled, to_disable
            );
            // We disabled until we either hit the disabled count, or we run out of entries to pop.
            while disabled < to_disable && let Some(id) = to_split.pop() {
                self.plugins.get_mut(&id).unwrap().status = Status::Disabled;
                debug!("Disabled {id}");

                disabled += 1;
            }
            self.update_dependencies();
            to_split = self.get_split_list();

            // We shuffle so we dont repeat the same order
            // which would yield the same dependencies being turned on and off.
            to_split.shuffle(&mut rng);

            // We find out how many we actually disabled after the dependencies have been applied.
            disabled = original_len - to_split.len();

            // If we have disabled the target amount, then we are done.
            // Else we continue removing entries and then checking after their dependencies.
            if disabled >= to_disable {
                break;
            }
        }

        self.update_display_order();
        self.save_state();
    }

    pub fn invert(&mut self) {
        for plugin in self.plugins.values_mut() {
            if plugin.status == Status::Enabled {
                plugin.status = Status::NotTheProblem;
            } else if plugin.status == Status::Disabled {
                plugin.status = Status::Enabled;
            }
        }
    }

    fn get_split_list(&self) -> Vec<String> {
        let mut to_split = Vec::new();
        for plugins in self.plugins.values() {
            if plugins.should_split() {
                to_split.push((plugins.id.clone(), plugins.name.clone(), plugins.stability));
            }
        }
        to_split.sort_by(|(_, v0s, v0), (_, v1s, v1)| v1.cmp(v0).then(v1s.cmp(v0s)));
        to_split.into_iter().map(|(id, _, _)| id).collect()
    }

    fn update_display_order(&mut self) {
        let mut screen_order = HashMap::<Status, Vec<PluginId>>::new();

        for plugin in self.plugins.values() {
            screen_order
                .entry(
                    plugin
                        .forced_status
                        .map(|v| if v { Status::Enabled } else { Status::Disabled })
                        .unwrap_or(plugin.status),
                )
                .or_default()
                .push(plugin.id.clone());
        }

        for plugins in screen_order.values_mut() {
            plugins.sort_by(|v0, v1| {
                self.plugins
                    .get(v0)
                    .unwrap()
                    .id
                    .cmp(&self.plugins.get(v1).unwrap().id)
            });
        }

        self.screen_order = Vec::new();
        for (status, plugins) in screen_order {
            self.screen_order.push((status, plugins));
        }
        self.screen_order.sort_by(|(v0, _), (v1, _)| v0.cmp(v1));
    }

    pub fn update_dependencies(&mut self) {
        let mut scan_dependencies = true;
        while scan_dependencies {
            scan_dependencies = false;

            let mut to_enable = Vec::new();
            for plugin in self.plugins.values() {
                if plugin.status == Status::Enabled {
                    for string in &plugin.depends_on {
                        if let Some(value) = self.plugins.get(string) {
                            // TODO check if its force disabled and tell the user that this will prob crash
                            if !value.status.enabled() {
                                to_enable.push(string.clone());
                            }
                        } else {
                            //minecraft lol
                            //panic!("Unmet dependency! {} depends on {}", plugin.id, string)
                        }
                    }
                }
            }

            for id in to_enable {
                debug!("Enabled {id} because its a dependency");
                self.plugins.get_mut(&id).unwrap().status = Status::Enabled;
                scan_dependencies = true;
            }
        }
    }

    pub fn split_count(&self) -> usize {
        let mut enabled = 0;
        for plugin in self.plugins.values() {
            if plugin.should_split() {
                enabled += 1;
            }
        }
        enabled
    }

    pub fn progress(&self) -> Option<f32> {
        self.scan_task
            .as_ref()
            .map(|v| v.scanned as f32 / v.count as f32)
    }

    pub fn start_scan(&mut self, mut folder: PathBuf, ctx: &Context) {
        if folder.ends_with(".minecraft") {
            folder.push("mods");
        }

        if self.scan_task.is_some() {
            return;
        }

        info!("Scanning {folder:?}");

        self.screen_order.clear();
        self.plugins.clear();
        self.undo_queue.clear();
        self.undo_queue_location = 0;
        // We need to be in the mods directory
        if !folder.ends_with("mods") {
            // TODO error message
            return;
        }

        if let Ok(dir) = read_dir(folder) {
            let (sender, receiver) = unbounded();

            let mut count = 0;
            for entry in dir.flatten() {
                let sender = sender.clone();
                let context = ctx.clone();
                count += 1;
                std::thread::spawn(move || {
                    let secs: f32 = thread_rng().gen();
                    let _ = sender.send(Plugin::new(entry.path(), &context));
                });
            }


            self.scan_task = Some(ScanTask {
                scanned: 0,
                count,
                receiver,
            });
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.plugins.is_empty() && self.scan_task.is_none()
    }

    pub fn save_state(&mut self) {
        if !self.is_ready() {
            return;
        }
        if self.undo_queue_location < self.undo_queue.len().saturating_sub(1) {
            let to_remove = self.undo_queue.len().saturating_sub(1) - self.undo_queue_location;
            for _ in 0..to_remove {
                self.undo_queue.pop();
            }
        }

        let mut state = HashMap::new();
        for (id, plugin) in &self.plugins {
            state.insert(id.clone(), plugin.status);
        }

        self.undo_queue_location = self.undo_queue.len();
        self.undo_queue.push(UndoState { plugins: state });
    }

    pub fn can_undo(&self) -> bool {
        self.undo_queue_location > 0
    }

    pub fn undo(&mut self) {
        if !self.is_ready() {
            return;
        }
        if self.can_undo() {
            self.undo_queue_location -= 1;
            self.update_state();
        }
    }

    pub fn can_redo(&self) -> bool {
        self.undo_queue_location < self.undo_queue.len().saturating_sub(1)
    }

    pub fn redo(&mut self) {
        if !self.is_ready() {
            return;
        }
        if self.can_redo() {
            self.undo_queue_location += 1;
            self.update_state();
        }
    }

    fn update_state(&mut self) {
        let state = &self.undo_queue[self.undo_queue_location];
        for (id, status) in &state.plugins {
            if let Some(plugin) = self.plugins.get_mut(id) {
                plugin.status = *status;
            } else {
                warn!("Plugin {id} does not exist, but is referenced in state");
            }
        }
        self.update_display_order();
    }
}

pub struct UndoState {
    plugins: HashMap<String, Status>,
}

pub struct ScanTask {
    scanned: usize,
    count: usize,
    receiver: Receiver<Plugin>,
}
