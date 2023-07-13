use bevy::utils::HashMap;
use crossterm::style::{ContentStyle, Stylize};
use itertools::{EitherOrBoth, Itertools};
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
        let styles = style_iters(value.fg.as_ref(), value.bg.as_ref(), color_map);

        let height = text_lines.len().max(styles.len());
        let width = text_lines
            .iter()
            .map(String::len)
            .max()
            .max(styles.iter().map(Vec::len).max())
            .unwrap_or_default();
        let mut charmi = CharacterMapImage::new();
        let mut style_row_iter = styles.into_iter();
        for y in 0..height {
            let mut row = CharmieRow::new();
            let mut x = 0;

            let mut style_iter = style_row_iter
                .next()
                .into_iter()
                .flat_map(|v| v.into_iter());
            let text_line = text_lines.get(y).map(|s| s.as_str()).unwrap_or_default();
            for ch in text_line.chars().chain(std::iter::repeat(gap_ch)) {
                let style = style_iter.next().flatten();
                if ch == gap_ch {
                    if let Some(style) = style {
                        row.add_effect(1, &style);
                    } else {
                        row.add_gap(1);
                    }
                } else {
                    row.add_char(ch, &style.unwrap_or_default());
                }
                let chr_width = ch.width().unwrap_or_default();
                if chr_width > 1 {
                    // drop next style section
                    style_iter.next();
                }
                x += chr_width;
                if x == width {
                    break;
                }
            }
            charmi.push_row(row);
        }
        charmi
    }
}

// Helper method for converting fg and bg to an array of arrays of styles
fn style_iters(
    fg: Option<&String>,
    bg: Option<&String>,
    char_map: HashMap<char, crossterm::style::Color>,
) -> Vec<Vec<Option<ContentStyle>>> {
    let fg = fg.into_iter().flat_map(|s| s.lines()).map(|line| {
        // TODO return Result<> if character not in char map and not space
        line.trim_end().chars().map(|c| char_map.get(&c).copied())
    });
    let bg = bg
        .into_iter()
        .flat_map(|s| s.lines())
        .map(|line| line.trim_end().chars().map(|c| char_map.get(&c).copied()));
    fg.zip_longest(bg)
        .map(|lines| {
            let (left, right) = match lines {
                EitherOrBoth::Left(left) => (Some(left), None),
                EitherOrBoth::Right(right) => (None, Some(right)),
                EitherOrBoth::Both(left, right) => (Some(left), Some(right)),
            };
            left.into_iter()
                .flatten()
                .zip_longest(right.into_iter().flatten())
                .map(|cell| {
                    let (left, right) = match cell {
                        EitherOrBoth::Left(left) => (left, None),
                        EitherOrBoth::Right(right) => (None, right),
                        EitherOrBoth::Both(left, right) => (left, right),
                    };
                    let mut style = None;
                    if let Some(fg_color) = left {
                        style = Some(ContentStyle::new().with(fg_color));
                    }
                    if let Some(bg_color) = right {
                        let unwrapped_style = style.unwrap_or_else(|| ContentStyle::new());
                        style = Some(unwrapped_style.on(bg_color))
                    }
                    style
                })
                .collect()
        })
        .collect()
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
        println!(
            "EXPECTED\n{}\n\nACTUAL\n{}",
            expected.to_string(),
            charmi.to_string()
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
