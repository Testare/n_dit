use super::model::curio_action::CurioAction;
use std::fs::{read_dir, read_to_string};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Configuration {
    pub assets_folder: String
}

pub fn load_action_dictionaries(config: Configuration) -> HashMap<String, Arc<CurioAction>> {
    if let Ok(files) = get_all_assets_of_name(config, "ad") {
        return files.iter()
            .filter_map(|file_contents|serde_json::from_str::<HashMap<String, Arc<CurioAction>>>(file_contents).ok())
            .fold(HashMap::<String, Arc<CurioAction>>::new(), |mut master_dictionary, new_dict| {
                master_dictionary.extend(new_dict);
                master_dictionary
            });
    } else {
        HashMap::new()
    }
}

// TODO Better error handling
// Pehraps look into using "glob" crate?
fn get_all_assets_of_name(config: Configuration, extension: &str) -> std::io::Result<Vec<String>> {
    Ok(read_dir(config.assets_folder).unwrap().flat_map(|dir| {
        match dir {
            Ok(dir) => {
                if dir.file_type().unwrap().is_dir() {
                    read_dir(dir.path()).unwrap().filter_map(|file| {
                        let file = file.unwrap();
                        if file.file_type().unwrap().is_file() {
                            let path = file.path();
                            let subextension = path
                                .file_stem()
                                .map(Path::new)
                                .and_then(Path::extension)
                                .and_then(|os_str|os_str.to_str());
                            if subextension == Some(extension) {
                                read_to_string(file.path()).ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }).collect()
                } else {
                    Vec::new()
                }
            }
            _ => { Vec::new() }
        }
    }).collect())
}
