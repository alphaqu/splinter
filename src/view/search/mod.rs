use std::path::PathBuf;

use eframe::egui::{Context, Ui};

use crate::ApplicationState;
use crate::data::Modpack;

pub struct SearchView {
    pub modpack: Modpack,
}
impl SearchView {
    pub fn new(path: PathBuf, ctx: &Context) -> Option<SearchView> {
        Some(SearchView {
            modpack: Modpack::new(path, ctx)?,
        })
    }
    pub fn ui(&mut self, state: &mut ApplicationState, ui: &mut Ui) {
        ui.add_space(12.0);
        self.modpack.ui(ui, state);
    }
}
