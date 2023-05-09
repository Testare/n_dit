
use crate::error::LoadingError;
use crate::assets::{AssetDictionary, Asset};
use std::fs::{read_dir, read_to_string};
use std::ffi::OsString;
use std::path::Path;

pub struct Configuration {
    pub assets_folder: String,
}

pub fn load_asset_dictionary<T: Asset + std::fmt::Debug>(config: &Configuration) -> Result<AssetDictionary<T>, LoadingError> {
    let files = get_all_assets_of_name(config, T::SUB_EXTENSION)?;
    Ok(files.iter()
            .filter_map(|(filename, file_contents)| {
                let dict_result = AssetDictionary::from_json(file_contents);
                match dict_result {
                    Ok(dict)=>Some(dict),
                    Err(err)=> {
                    log::warn!("Unable to load asset [{:?}] : [{}]", filename, err);
                        None
                    }
                }
            })
            .reduce(
                |mut master_dictionary, new_dict| {
                    master_dictionary.extend(new_dict);
                    master_dictionary
                }
            ).unwrap_or_default())
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
