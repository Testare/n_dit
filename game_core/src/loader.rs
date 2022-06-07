mod node_definition;
mod sprite_definition;

pub use node_definition::NodeDef;
pub use sprite_definition::{CurioDef, SpriteDef};

use super::model::curio_action::CurioAction;
use crate::assets::{AssetDictionary, Asset};
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub enum LoadingError {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

impl From<std::io::Error> for LoadingError {
    fn from(err: std::io::Error) -> Self {
        LoadingError::Io(err)
    }
}

impl From<serde_json::Error> for LoadingError {
    fn from(err: serde_json::Error) -> Self {
        LoadingError::SerdeJson(err)
    }
}

pub struct Configuration {
    pub assets_folder: String,
}

pub fn load_action_dictionaries(config: &Configuration) -> HashMap<String, Arc<CurioAction>> {
    if let Ok(files) = get_all_assets_of_name(config, "actions") {
        return files
            .iter()
            .filter_map(|file_contents| {
                serde_json::from_str::<HashMap<String, Arc<CurioAction>>>(file_contents).ok()
            })
            .fold(
                HashMap::<String, Arc<CurioAction>>::new(),
                |mut master_dictionary, new_dict| {
                    master_dictionary.extend(new_dict);
                    master_dictionary
                },
            );
    } else {
        HashMap::new()
    }
}

pub fn load_asset_dictionary<T: Asset>(config: &Configuration) -> Result<AssetDictionary<T>, LoadingError> {
    let files = get_all_assets_of_name(config, T::SUB_EXTENSION)?;
    Ok(files.iter()
            .filter_map(|file_contents| {
                serde_json::from_str::<HashMap<String, Arc<T>>>(file_contents).ok()
            })
            .fold(
                HashMap::<String, Arc<T>>::new(),
                |mut master_dictionary, new_dict| {
                    master_dictionary.extend(new_dict);
                    master_dictionary
                },
            ).into())
}

fn get_all_assets_of_name(config: &Configuration, extension: &str) -> std::io::Result<Vec<String>> {
    Ok(read_dir(config.assets_folder.as_str())
        .unwrap()
        .flat_map(|dir| match dir {
            Ok(dir) => {
                if dir.file_type().unwrap().is_dir() {
                    read_dir(dir.path())
                        .unwrap()
                        .filter_map(|file| {
                            let file = file.unwrap();
                            if file.file_type().unwrap().is_file() {
                                let path = file.path();
                                let subextension = path
                                    .file_stem()
                                    .map(Path::new)
                                    .and_then(Path::extension)
                                    .and_then(|os_str| os_str.to_str());
                                if subextension == Some(extension) {
                                    read_to_string(file.path()).ok()
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        })
        .collect())
}
