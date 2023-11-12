//use std::collections::HashMap;
// use std::fs::read_to_string;
// use std::path::{Path, PathBuf};
//
// pub struct ModpackMetadata {
//     name: Option<String>,
//     icon: Option<String>,
// }
//
// impl ModpackMetadata {
//     pub fn new(dir: &PathBuf) -> ModpackMetadata {
//         if let Some(value) = Self::new_multimc(dir) {
//             return value;
//         }
//
//         ModpackMetadata {
//             name: None,
//             icon: None,
//         }
//     }
//
//     fn new_multimc(dir: &Path) -> Option<ModpackMetadata> {
//         let parent = dir.parent()?;
//         let string = read_to_string(parent.join("instance.cfg")).ok()?;
//         let mut properties: HashMap<String, String> = string
//             .split('\n')
//             .flat_map(|v| {
//                 v.split_once('=')
//                     .map(|(k, v)| (k.to_string(), v.to_string()))
//             })
//             .collect();
//
//         return Some(ModpackMetadata {
//             name: properties.remove("name"),
//             icon: properties.remove("iconKey"),
//         });
//     }
// }