mod node_definition;
mod sprite_definition;

pub use node_definition::{NodeDef, node_from_def};
pub use sprite_definition::{CardDef, SpriteDef, CardInstanceDefAlternative};

use crate::assets::{AssetDictionary, Asset};
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::ffi::OsString;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub enum LoadingError {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    DecodeError(base64::DecodeError),
    MissingAsset(&'static str, String),
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

impl From<base64::DecodeError> for LoadingError {
    fn from(err: base64::DecodeError) -> Self {
        LoadingError::DecodeError(err)
    }
}

pub struct Configuration {
    pub assets_folder: String,
}

pub fn load_asset_dictionary<T: Asset + std::fmt::Debug>(config: &Configuration) -> Result<AssetDictionary<T>, LoadingError> {
    let files = get_all_assets_of_name(config, T::SUB_EXTENSION)?;
    Ok(files.iter()
            .filter_map(|(filename, file_contents)| {
                let serde_result = serde_json::from_str::<HashMap<String, T::UnnamedAsset>>(file_contents);
                match serde_result {
                    Ok(dict)=>Some(dict.into_iter()
                        .map(|(key, unnamed)| {
                            let named = T::with_name(unnamed, key.as_str());
                            (key, Arc::new(named))
                        })
                        .collect::<HashMap<String, Arc<T>>>()),
                    Err(err)=> {
                        log::warn!("Unable to load asset [{:?}] : [{}]", filename, err);
                        None
                    }
                }
            })
            .fold(
                HashMap::<String, Arc<T>>::new(),
                |mut master_dictionary, new_dict| {
                    master_dictionary.extend(new_dict);
                    master_dictionary
                },
            ).into())
}

fn get_all_assets_of_name(config: &Configuration, extension: &str) -> std::io::Result<Vec<(OsString, String)>> {
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
                                    match read_to_string(file.path()) {
                                        Ok(contents) => Some((path.into_os_string(), contents)),
                                        Err(err) => {
                                            log::warn!("Unable to read asset file [{:?}], error: [{}]", path.into_os_string(), err);
                                            None
                                        }
                                    }
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
