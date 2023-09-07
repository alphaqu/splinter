use std::collections::{HashMap, HashSet};
use std::fs::read_dir;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{Context, FontFamily, FontId, RichText, ScrollArea, Ui};
use rand::prelude::SliceRandom;
use rayon::iter::*;
use tracing::{debug, info, warn};

pub use loader::ModpackLoader;
pub use metadata::ModpackMetadata;
use splinter_event::{EventSystem, EventTracker};
use crate::{ApplicationState, ModpackStatus};
pub use crate::data::Plugin;
pub use crate::data::PluginStatus;
use crate::ui::{color, UiSystem};

mod loader;
mod metadata;


#[derive(Debug)]
pub enum ModpackOperationEvent {
    Undo,
    Redo,
    Split,
    Invert
}

pub struct Modpack {
    path: PathBuf,
    metadata: ModpackMetadata,
    plugins: PluginList,
    loader: Option<ModpackLoader>,

    display_order: Vec<Vec<String>>,
    // This contains the list of mod-ids which splinter is going to ask the user to enable.
    to_ask: Vec<AskingEnable>,

    undo_queue: Vec<State>,
    undo_queue_location: usize,

    tracker: EventTracker,
}

impl Modpack {
    pub fn new(mut path: PathBuf, ctx: &Context) -> Option<Modpack> {
        if let Ok(new_path) = path.strip_prefix("~"){
            path = dirs::home_dir().unwrap().join(new_path);
        }
        if path.ends_with("mods") {
            path = path.parent()?.to_path_buf();
        }
        let mc = path.join(".minecraft");
        if mc.exists() {
            path = mc;
        }

        info!("Loading {path:?}");
        if let Ok(dir) = read_dir(path.join("mods")) {
            return Some(Modpack {
                metadata: ModpackMetadata::new(&path),
                path,
                plugins: PluginList::new(),
                loader: Some(ModpackLoader::new(
                    dir.flatten().map(|v| v.path()).collect(),
                    ctx,
                )),
                display_order: vec![],
                to_ask: vec![],
                undo_queue: vec![],
                undo_queue_location: 0,
                tracker: EventTracker::new(),
            });
        } else {
            warn!("Could not find mods folder")
        }

        None
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn ui(&mut self, ui: &mut Ui, state: &mut ApplicationState) {
        let commander = self.tracker.tick(&mut state.events);
        for event in commander.consume::<ModpackOperationEvent>() {
            match event {
                ModpackOperationEvent::Undo => self.undo(),
                ModpackOperationEvent::Redo => self.redo(),
                ModpackOperationEvent::Split => self.split(),
                ModpackOperationEvent::Invert => self.invert(),
            }
        }
        // Load plugins which are getting loaded.
        if let Some(loader) = &mut self.loader {
            ui.ctx().request_repaint();

            let i = self.plugins.list.len();
            if let Err(()) = loader.tick(&mut self.plugins, &mut state.events) {
                self.loader = None;
                self.save_state();
            }

            if self.plugins.list.len() != i {
                self.update_display_order();
            }
        }

        ScrollArea::vertical().show(ui, |ui| {
            for (i, plugins) in self.display_order.iter().enumerate() {
                let status = PluginStatus::iter()[i];
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(match status {
                            PluginStatus::Disabled => "Disabled Mods",
                            PluginStatus::Enabled => "Enabled Mods",
                            PluginStatus::NotTheProblem => "Not faulty",
                        })
                        .color(color::TEXT)
                            .font(FontId::new(18.0, FontFamily::Name(Arc::from("Roboto-Bold"))))
                    );
                });
                ui.add_space(4.0);
                for id in plugins {
                    if let Some(plugin) = self.plugins.get_mut(id) {
                        plugin.ui(ui);
                        ui.add_space(8.0);
                    }
                }
                ui.add_space(8.0);
            }
        });

        state.modpack_status = ModpackStatus::Active {
            path: self.path.clone(),
            is_loaded: !self.is_loading(),
            can_undo: self.can_undo(),
            can_redo: self.can_redo(),
        };
    }

    /// Just flips enabled <-> disabled.
    pub fn invert(&mut self) {
        if self.is_loading() {
            return;
        }

        for plugin in self.plugins.iter_mut() {
            if plugin.status == PluginStatus::Disabled {
                plugin.status = PluginStatus::Enabled;
            } else if plugin.status == PluginStatus::Enabled {
                plugin.status = PluginStatus::Disabled;
            }
        }

        self.update_display_order();
        self.save_state();
        self.push_changes();
    }

    /// Binary splits the enabled mods. Wont do anything if the modpack is currently loading.
    pub fn split(&mut self) {
        if self.is_loading() {
            return;
        }

        // Move the disabled plugins to the NotTheProblem status,
        // because we confirmed that this configuration still contains the issue and the other mods are irrelevant.
        for plugin in self.plugins.iter_mut() {
            if plugin.status == PluginStatus::Disabled {
                plugin.status = PluginStatus::NotTheProblem;
            }
        }

        let mut to_split = self.splittable_plugins();
        let original_len = to_split.len();
        let to_disable = to_split.len() / 2;
        let mut disabled = 0;

        let mut rng = rand::thread_rng();
        for i in 0..100 {
            debug!(
                "Attempt {i} on splitting, {}/{} remaining.",
                disabled, to_disable
            );
            // We disabled until we either hit the disabled count, or we run out of entries to pop.
            while disabled < to_disable && let Some(id) = to_split.pop() {
				self.plugins.get_mut(&id).unwrap().status = PluginStatus::Disabled;
				debug!("Disabled {id}");
				disabled += 1;
			}

            self.enable_dependencies();
            to_split = self.splittable_plugins();

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

        self.update_ask(AskingKind::SplitDependency);
        self.update_display_order();
        self.save_state();
        self.push_changes();
    }

    /// Gets a list of plugins which are able to be disabled in a split operation.
    fn splittable_plugins(&self) -> Vec<String> {
        let mut to_split = Vec::new();

        for plugin in self.plugins.iter() {
            if plugin.should_split() {
                to_split.push((plugin.metadata.id.clone(), plugin.stability));
            }
        }

        // We sort by the priority.
        to_split.sort_by(|(_, v0), (_, v1)| v1.cmp(v0));
        to_split.into_iter().map(|(id, _)| id).collect()
    }

    /// Goes through the enabled mods and enables their dependencies.
    /// It does this recursively because dependencies may depend on other dependencies.
    fn enable_dependencies(&mut self) {
        let mut scan = true;
        while scan {
            scan = false;
            for id in self.get_dependant_disabled_mods() {
                if let Some(value) = self.plugins.get(&id) {
                    if value.forced_status == Some(false) {
                        // We ask the user to enable this mod at the end of the split operations,
                        // as they have disabled it and it's about to break things.
                        // self.to_ask.push(id);
                        continue;
                    }
                }

                self.plugins.get_mut(&id).unwrap().status = PluginStatus::Enabled;

                // We just enabled a dependency,
                // we need to run this again as there may be new dependencies which need to be enabled.
                scan = true;
            }
        }
    }

    /// Updates the list of mods which we advise the user to enable
    fn update_ask(&mut self, kind: AskingKind) {
        self.to_ask.clear();
        for id in self.get_dependant_disabled_mods() {
            self.to_ask.push(AskingEnable {
                depended_by: self.find_dependants(&id),
                id,
                kind,
            });
        }
    }

    /// Get disabled mods which are depended by other enabled mods.
    ///
    /// Basically we get the mods which the modloader is about to tell the user are not there.
    fn get_dependant_disabled_mods(&self) -> HashSet<String> {
        let mut to_enable = HashSet::new();

        for plugin in self.plugins.iter() {
            if plugin.status == PluginStatus::Enabled {
                for id in &plugin.metadata.depends_on {
                    if let Some(value) = self.plugins.get(id) {
                        if !value.status.enabled() {
                            to_enable.insert(id.clone());
                        }
                    }
                }
            }
        }

        to_enable
    }

    /// Gives a list of mods which are dependant on this id.
    fn find_dependants(&self, id: &str) -> Vec<String> {
        let mut dependants = Vec::new();
        for plugin in self.plugins.iter() {
            if plugin.status == PluginStatus::Enabled {
                for depends_on in &plugin.metadata.depends_on {
                    if depends_on == id {
                        dependants.push(plugin.metadata.id.clone());
                    }
                }
            }
        }

        dependants
    }

    fn update_display_order(&mut self) {
        self.display_order.clear();
        for status in PluginStatus::iter() {
            let mut plugins = Vec::new();
            for plugin in self.plugins.iter() {
                if plugin.status == status {
                    plugins.push(plugin.metadata.id.clone());
                }
            }

            if !plugins.is_empty() {
                plugins.sort_by(|v0, v1| {
                    self.plugins
                        .get(v0)
                        .unwrap()
                        .metadata
                        .id
                        .cmp(&self.plugins.get(v1).unwrap().metadata.id)
                });
                self.display_order.push(plugins);
            }
        }
    }

    pub fn plugins(&self) -> &PluginList {
        &self.plugins
    }

    pub fn plugins_mut(&mut self) -> &mut PluginList {
        &mut self.plugins
    }

    pub fn is_loading(&self) -> bool {
        self.loader.is_some()
    }

    pub fn save_state(&mut self) {
        if self.is_loading() {
            return;
        }

        if self.undo_queue_location < self.undo_queue.len().saturating_sub(1) {
            let to_remove = self.undo_queue.len().saturating_sub(1) - self.undo_queue_location;
            for _ in 0..to_remove {
                self.undo_queue.pop();
            }
        }

        let mut state = HashMap::new();
        for plugin in self.plugins().iter() {
            state.insert(plugin.metadata.id.clone(), plugin.status);
        }

        self.undo_queue_location = self.undo_queue.len();
        self.undo_queue.push(State { plugins: state });
    }

    pub fn can_undo(&self) -> bool {
        self.undo_queue_location > 0
    }

    pub fn undo(&mut self) {
        if self.is_loading() {
            return;
        }
        if self.can_undo() {
            self.undo_queue_location -= 1;
            self.update_state();
            self.push_changes();
        }
    }

    pub fn can_redo(&self) -> bool {
        self.undo_queue_location < self.undo_queue.len().saturating_sub(1)
    }

    pub fn redo(&mut self) {
        if self.is_loading() {
            return;
        }
        if self.can_redo() {
            self.undo_queue_location += 1;
            self.update_state();
            self.push_changes();
        }
    }

    fn push_changes(&mut self) {
        let list = &mut self.plugins;
        for plugin in &mut list.list {
            plugin.push_changes();
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

pub struct PluginList {
    list: Vec<Plugin>,
    lookup: HashMap<String, usize>,
}

impl PluginList {
    pub fn new() -> PluginList {
        PluginList {
            list: vec![],
            lookup: Default::default(),
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.lookup.contains_key(id)
    }

    pub fn get(&self, id: &str) -> Option<&Plugin> {
        self.list.get(*self.lookup.get(id)?)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Plugin> {
        self.list.get_mut(*self.lookup.get(id)?)
    }

    pub fn iter(&self) -> &[Plugin] {
        &self.list
    }

    pub fn iter_mut(&mut self) -> &mut [Plugin] {
        &mut self.list
    }
}


struct State {
    plugins: HashMap<String, PluginStatus>
}

struct AskingEnable {
    id: String,
    depended_by: Vec<String>,
    kind: AskingKind,
}

#[derive(Copy, Clone)]
enum AskingKind {
    SplitDependency,
    MakingForce,
}
