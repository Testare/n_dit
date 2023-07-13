use bevy::utils::HashMap;
use crossterm::style::{ContentStyle, Stylize};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{CharacterMapImage, CharmieRow};

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Debug, Default, Deserialize, Serialize)]
struct Values {
    colors: Option<HashMap<char, ColorDef>>,
    attr: Option<HashMap<char, String>>, // Option<HashMap<char, Vec<String>>> ?
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
    values: Option<Values>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CharmieActorDef {
    ani: HashMap<String, CharmieAnimationDef>,
    values: Option<Values>,
}

impl std::ops::Add<&Values> for &Values {
    type Output = Values;
    fn add(self, rhs: &Values) -> Self::Output {
        Values {
            colors: match (rhs.colors.as_ref(), self.colors.as_ref()) {
                (Some(rhs_colors), Some(lhs_colors)) => {
                    let mut map = HashMap::new();
                    map.extend(lhs_colors.clone());
                    map.extend(rhs_colors.clone());
                    Some(map)
                },
                (Some(colors), None) | (None, Some(colors)) => Some(colors.clone()),
                (None, None) => None,
            },
            attr: match (rhs.attr.as_ref(), self.attr.as_ref()) {
                (Some(rhs_attr), Some(lhs_attr)) => {
                    let mut map = HashMap::new();
                    map.extend(lhs_attr.clone());
                    map.extend(rhs_attr.clone());
                    Some(map)
                },
                (Some(attr), None) | (None, Some(attr)) => Some(attr.clone()),
                (None, None) => None,
            },
            gap: rhs.gap.as_ref().or(self.gap.as_ref()).cloned(),
        }
    }
}

impl From<ColorDef> for crossterm::style::Color {
    fn from(value: ColorDef) -> Self {
        match value {
            ColorDef::Ansi(ansi) => crossterm::style::Color::AnsiValue(ansi),
            ColorDef::Rgb(r, g, b) => crossterm::style::Color::Rgb { r, g, b },
            ColorDef::Named(name) => crossterm::style::Color::Red, // TODO better
        }
    }
}

impl From<CharmieDef> for CharacterMapImage {
    fn from(value: CharmieDef) -> Self {
        let values = value.values.unwrap_or_default();
        let gap_ch = values.gap.unwrap_or(' ');
        let color_map: HashMap<char, crossterm::style::Color> = values
            .colors
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        let text_lines: Vec<String> = value
            .text
            .map(|text| text.lines().map(|s| s.to_owned()).collect())
            .unwrap_or_default();
        let fg_lines: Vec<Vec<char>> = value
            .fg
            .map(|text| str_to_char_array(text.as_str()))
            .unwrap_or_default();
        let bg_lines: Vec<Vec<char>> = value
            .bg
            .map(|text| str_to_char_array(text.as_str()))
            .unwrap_or_default();
        let height = text_lines.len().max(fg_lines.len()).max(bg_lines.len());
        let width = text_lines
            .iter()
            .map(String::len)
            .max()
            .max(fg_lines.iter().map(Vec::len).max())
            .max(bg_lines.iter().map(Vec::len).max())
            .unwrap_or_default();
        let mut charmi = CharacterMapImage::new();
        for y in 0..height {
            let mut row = CharmieRow::new();
            let mut x = 0;
            let fg_line = fg_lines.get(y);
            let bg_line = bg_lines.get(y);
            let text_line = text_lines.get(y).map(|s| s.as_str()).unwrap_or_default();
            for ch in text_line.chars().chain(std::iter::repeat(gap_ch)) {
                let mut style = ContentStyle::default().stylize();
                let fg = fg_line.and_then(|line| {
                    let color = color_map.get(line.get(x)?)?;
                    style = style.with(*color);
                    Some(color)
                });
                let bg = bg_line.and_then(|line| {
                    let color = color_map.get(line.get(x)?)?;
                    style = style.on(*color);
                    Some(color)
                });
                if ch == gap_ch {
                    if bg.is_some() || fg.is_some() {
                        row.add_effect(1, &style);
                    } else {
                        row.add_gap(1);
                    }
                } else {
                    row.add_char(ch, &style);
                }
                x += ch.width().unwrap_or_default();
                if x == width {
                    break;
                }
            }
            charmi.push_row(row);
        }
        charmi
    }
}

fn str_to_char_array(input: &str) -> Vec<Vec<char>> {
    input.lines().map(|line| line.chars().collect()).collect()
}

#[cfg(test)]
mod test {
    use test_log::test;

    use super::*;

    #[test]
    fn load_test_charmi_file() {
        let mut test_charmi = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_charmi.push("assets/test.charmi");
        let result_str = std::fs::read_to_string(test_charmi).expect("test file should exist");
        log::debug!("CHARMI STR: {:?}", result_str);

        let charmie_def: CharmieDef =
            toml::from_str(result_str.as_str()).expect("test definition should parse successfully");

        let charmi: CharacterMapImage = charmie_def.into();
        let mut expected: CharacterMapImage = CharacterMapImage::new();
        let orange = crossterm::style::Color::AnsiValue(208);
        let white = crossterm::style::Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        expected.push_row(
            CharmieRow::of_gap(2)
                .with_styled_text("ygb".stylize().red().on_red())
                .with_gap(2),
        );
        expected.push_row(
            CharmieRow::of_gap(1)
                .with_styled_text("o".stylize().with(orange).on_red())
                .with_gap(3)
                .with_styled_text("i".stylize().red().on(orange))
                .with_gap(1),
        );
        expected.push_row(
            CharmieRow::of_char('r', &ContentStyle::new().red().on_red())
                .with_gap(1)
                .with_styled_text("=0".stylize().red().on(white))
                .with_effect(1, &ContentStyle::new().red().on(white))
                .with_gap(1)
                .with_styled_text("v".stylize().red().on_red()),
        );
        assert_eq!(charmi, expected)
    }

    #[test]
    fn load_test_charmi_actor_file() {
        let mut test_charmi = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_charmi.push("assets/test.charmia");
        let result_str = std::fs::read_to_string(test_charmi);
        log::debug!("CHARMIE STR: {:?}", result_str);

        let charmie_def: Result<CharmieActorDef, _> =
            toml::from_str(result_str.expect("text file to exist").as_str());

        log::debug!("CHARMIE DEF: {:?}", charmie_def);
        charmie_def.expect("test definition should parse successfully");
    }
}
