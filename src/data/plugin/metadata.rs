use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Read};
use eframe::epaint::ahash::{HashSet, HashSetExt};
use image::DynamicImage;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use zip::ZipArchive;

pub type PluginId = String;

pub struct PluginMetadata {
    pub id: PluginId,
    // Some mods like fabric api provide multiple ids to be backwards compatible
    pub provides: Vec<PluginId>,
    // This is a list of mods which are bundled within this plugin.
    pub contains: Vec<PluginMetadata>,
    pub version: String,
    pub name: String,
    pub icon: Option<String>,
    pub depends_on: Vec<String>,
}

impl PluginMetadata {
    pub fn new<R: Read + io::Seek>(zip: &mut ZipArchive<R>) -> Option<PluginMetadata> {
        if let Some(mut metadata) = FabricMetadata::new(zip) {
            let mut depends_on = HashSet::new();
            Self::add_module_depends(&metadata, &mut depends_on);
            metadata.depends_on = depends_on.into_iter().collect();
            return Some(metadata);
        }

        None
    }

    fn add_module_depends(metadata: &PluginMetadata, depends_on: &mut HashSet<String>) {
        for on in &metadata.depends_on {
            depends_on.insert(on.clone());
        }

        for plugin in &metadata.contains {
            Self::add_module_depends(plugin, depends_on);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct FabricMetadata {
    id: String,
    provides: Option<Vec<String>>,
    version: String,
    name: String,
    icon: Option<String>,
    depends: Option<HashMap<String, String>>,
    jars: Option<Vec<FabricJarEntry>>
}


#[derive(Serialize, Deserialize)]
struct FabricJarEntry {
    file: String
}

impl FabricMetadata {
    pub fn new<R: Read + io::Seek >(zip: &mut ZipArchive<R>) -> Option<PluginMetadata> {
        let mut file = zip.by_name("fabric.mod.json").ok()?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).ok()?;
        let mut json: FabricMetadata = serde_json::from_slice(&data).ok()?;
        drop(file);

        let mut contains = Vec::new();
        if let Some(jars) = json.jars {
            for jar in jars {
                debug!("Loading inner mod {}", jar.file);
                let mut file = zip.by_name(&jar.file).unwrap();
                let mut file_data = Vec::new();
                file.read_to_end(&mut file_data).unwrap();

                let mut reader = Cursor::new(file_data.as_slice());
                let mut archive = ZipArchive::new(&mut reader).unwrap();
                if let Some(metadata) = PluginMetadata::new(&mut archive) {
                    contains.push(metadata);
                } else {
                    warn!("Failed to read inner jar {}", jar.file);
                }
            }
        }

        let mut depends_on = HashSet::new();
        if let Some(value) = &json.depends {
            for (name, _) in value {
                depends_on.insert(name.clone());
            }
        }

        Some(PluginMetadata {
            id: json.id,
            provides: json.provides.unwrap_or_default(),
            contains,
            version: json.version,
            name: json.name,
            icon: json.icon,
            depends_on: depends_on.into_iter().collect(),
        })
    }
}
