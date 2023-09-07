use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::PathBuf;

use crossbeam::channel::{Receiver, unbounded};
use eframe::egui::Context;
use tracing::debug;

use splinter_event::{EventSystem, EventTracker};

use crate::data::{PluginList, PluginMetadata};
use crate::data::Plugin;
use crate::ui::{NotificationEvent, ProgressStatus, Severity};
use crate::view::ProgressEvent;

pub struct ModpackLoader {
    total_plugins: usize,
    receiver: Receiver<Option<Plugin>>,
    tracker: EventTracker
}

impl ModpackLoader {
    pub fn new(paths: Vec<PathBuf>, ctx: &Context) -> ModpackLoader {
        let (sender, receiver) = unbounded();
        let total_plugins = paths.len();
        for path in paths {
            let sender = sender.clone();
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                sender.send(Plugin::new(path, &ctx)).unwrap();
            });
        }

        ModpackLoader {
            total_plugins,
            receiver,
            tracker: EventTracker::new(),
        }
    }

    pub fn tick(&mut self, plugins: &mut PluginList, events: &mut EventSystem) -> Result<(), ()> {
        let mut commander = self.tracker.tick(events);
        while let Ok(plugin) = self.receiver.try_recv() {
            if let Some(plugin) = plugin {
                let idx = plugins.list.len();

                let mut add_id = |id: String| {
                    debug!("Adding id binding {id} to {}", plugin.metadata.id);
                    if let Some(old) = plugins.lookup.insert(id.clone(), idx) {
                        commander.dispatch(NotificationEvent {
                            title: "Duplicate ids".to_string(),
                            description: format!(
                                "Mod \"{}\" and \"{}\" have the same id \"{}\"",
                                plugin.metadata.name, plugins.list[old].metadata.name, id
                            ),
                            ty: Severity::Warning,
                        });
                    }
                };

                add_id(plugin.metadata.id.clone());
                for id in &plugin.metadata.provides {
                    add_id(id.clone());
                }

                plugins.list.push(plugin);
            } else {
                self.total_plugins -= 1;
            }
        }

        if plugins.list.len() >= self.total_plugins {
            for (id, plugin) in plugins.list.iter().enumerate() {
                Self::add_modules(id, &plugin.metadata, &mut plugins.lookup);
            }

            commander.dispatch(ProgressEvent(None));
            Err(())
        } else {
            commander.dispatch(ProgressEvent(Some(ProgressStatus::Determinate(
                plugins.list.len() as f32 / self.total_plugins as f32,
            ))));
            Ok(())
        }
    }

    fn add_modules(id: usize, plugin: &PluginMetadata, lookup: &mut HashMap<String, usize>) {
        for metadata in &plugin.contains {
            Self::add_modules(id, metadata, lookup);
            match lookup.entry(metadata.id.clone()) {
                Entry::Occupied(_) => {}
                Entry::Vacant(entry) => {
                    entry.insert(id);
                }
            }
        }
    }
}
