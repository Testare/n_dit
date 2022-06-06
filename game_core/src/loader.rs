use std::fs::{read_dir, read_to_string};
pub struct Configuration {
    assets_folder: String
}

pub fn load_action_dictionaries(config: Configuration) {
    
    
        // std::fs::read_to_string("./assets/nightfall/action_dictionary.json").unwrap();
}

fn get_all_assets_of_name(config: Configuration, filename: String) -> std::io::Result<Vec<String>> {
    Ok(read_dir(config.assets_folder)?.flat_map(|dir| {
        if dir?.file_type()?.is_dir() {
            read_dir(dir?.path())?.filter_map(|file| {
                if file?.ends_with("ad.json") && file?.file_type()?.is_dir() {
                    Some(read_to_string(file.path()))
                } else {
                    None
                }
            })
        }
    }).collect())
}