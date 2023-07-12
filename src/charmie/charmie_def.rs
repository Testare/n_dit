use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum ColorDef {
    Named(String),
    Ansi(u8),
    Rgb(u8, u8, u8),
    // Rgba -> ???
}



#[derive(Debug, Deserialize, Serialize)]
struct CharmieDef {
    text: Option<String>,
    fg: Option<String>,
    bg: Option<String>,
    attr: Option<String>,
    values: Option<Values>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Values {
    colors: Option<HashMap<char, ColorDef>>,
    attr: Option<HashMap<char, String>>,
    gap: Option<char>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CharmieFrameDef {
    text: Option<String>,
    fg: Option<String>,
    bg: Option<String>,
    attr: Option<String>,
    values: Option<Values>,
    timing: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct CharmieAnimationDef {
    frame: Vec<CharmieFrameDef>,
    values: Option<Values>
}

#[derive(Debug, Deserialize, Serialize)]
struct CharmieActorDef {
    ani: HashMap<String, CharmieAnimationDef>,
    values: Option<Values>
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;


    #[test]
    fn load_test_charmi_file() {
        let mut test_charmi = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_charmi.push("assets/test.charmi");
        let result_str = std::fs::read_to_string(test_charmi);
        log::debug!("CHARMI STR: {:?}", result_str);

        let charmie_def: Result<CharmieDef, _> = toml::from_str(result_str.expect("text file to exist").as_str());

        log::debug!("CHARMI DEF: {:?}", charmie_def);
        charmie_def.expect("test definition should parse successfully");
    }

    #[test]
    fn load_test_charmi_actor_file() {
        let mut test_charmi = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_charmi.push("assets/test.charmia");
        let result_str = std::fs::read_to_string(test_charmi);
        log::debug!("CHARMIE STR: {:?}", result_str);

        let charmie_def: Result<CharmieActorDef, _> = toml::from_str(result_str.expect("text file to exist").as_str());

        log::debug!("CHARMIE DEF: {:?}", charmie_def);
        charmie_def.expect("test definition should parse successfully");
    }

}
