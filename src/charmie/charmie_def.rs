use std::collections::HashSet;
use std::sync::OnceLock;

use bevy::utils::HashMap;
use crossterm::style::{Color, ContentStyle, Stylize};
use itertools::{EitherOrBoth, Itertools};
use serde::{Deserialize, Serialize, Serializer};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{CharacterMapImage, CharmieRow, CharmieSegment};

static COLOR_NAMES: OnceLock<HashMap<String, Color>> = OnceLock::new();

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
enum ColorDef {
    Named(String),
    Ansi(u8),
    Rgb(u8, u8, u8),
    // Rgba -> ???
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct CharmieDef {
    text: Option<String>,
    fg: Option<String>,
    bg: Option<String>,
    attr: Option<String>,
    values: Option<Values>,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
struct Values {
    #[serde(serialize_with = "char_map_serialize")]
    colors: Option<HashMap<char, ColorDef>>,
    attr: Option<HashMap<char, String>>, // Option<HashMap<char, Vec<String>>> ?
    gap: Option<char>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct CharmieFrameDef {
    text: Option<String>,
    fg: Option<String>,
    bg: Option<String>,
    attr: Option<String>,
    values: Option<Values>,
    timing: f32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct CharmieAnimationDef {
    frame: Vec<CharmieFrameDef>,
    values: Option<Values>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct CharmieActorDef {
    ani: HashMap<String, CharmieAnimationDef>,
    values: Option<Values>,
}

fn char_map_serialize<S, T: Serialize>(
    field: &Option<HashMap<char, T>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(map) = field {
        s.collect_map(map.iter().map(|(ch, t)| (ch.to_string(), t)))
    } else {
        s.serialize_none()
    }
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

impl TryFrom<&ColorDef> for Color {
    type Error = ();
    fn try_from(value: &ColorDef) -> Result<Self, Self::Error> {
        match value {
            ColorDef::Ansi(ansi) => Ok(Color::AnsiValue(*ansi)),
            ColorDef::Rgb(r, g, b) => Ok(Color::Rgb {
                r: *r,
                g: *g,
                b: *b,
            }),
            ColorDef::Named(name) => COLOR_NAMES
                .get_or_init(|| {
                    [
                        ("black", Color::Black),
                        ("dark red", Color::DarkRed),
                        ("darkred", Color::DarkRed),
                        ("dark green", Color::DarkGreen),
                        ("darkgreen", Color::DarkGreen),
                        ("dark yellow", Color::DarkYellow),
                        ("darkyellow", Color::DarkYellow),
                        ("dark blue", Color::DarkBlue),
                        ("darkblue", Color::DarkBlue),
                        ("navy", Color::DarkBlue),
                        ("dark magenta", Color::DarkMagenta),
                        ("darkmagenta", Color::DarkMagenta),
                        ("purple", Color::DarkMagenta),
                        ("dark cyan", Color::DarkCyan),
                        ("darkcyan", Color::DarkCyan),
                        ("teal", Color::DarkCyan),
                        ("grey", Color::Grey),
                        ("gray", Color::Grey),
                        ("dark grey", Color::DarkGrey),
                        ("darkgrey", Color::DarkGrey),
                        ("dark gray", Color::DarkGrey),
                        ("darkgray", Color::DarkGrey),
                        ("red", Color::Red),
                        ("green", Color::Green),
                        ("lime", Color::Green),
                        ("yellow", Color::Yellow),
                        ("blue", Color::Blue),
                        ("magenta", Color::Magenta),
                        ("cyan", Color::Cyan),
                        ("aqua", Color::Cyan),
                        ("white", Color::White),
                    ]
                    .into_iter()
                    .map(|(k, v)| (k.to_owned(), v))
                    .collect()
                })
                .get(&name.to_lowercase())
                .copied()
                .ok_or(()),
        }
    }
}

impl TryFrom<ColorDef> for Color {
    type Error = ();
    fn try_from(value: ColorDef) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<Color> for ColorDef {
    fn from(value: Color) -> Self {
        match value {
            Color::Reset => panic!("Reset not a valid color for Charmie"),
            Color::Rgb { r, g, b } => ColorDef::Rgb(r, g, b),
            Color::AnsiValue(ansi) => ColorDef::Ansi(ansi),
            Color::Black => "black".into(),
            Color::DarkRed => "dark red".into(),
            Color::DarkGreen => "dark green".into(),
            Color::DarkYellow => "dark yellow".into(),
            Color::DarkBlue => "dark blue".into(),
            Color::DarkMagenta => "dark magenta".into(),
            Color::DarkCyan => "dark cyan".into(),
            Color::Grey => "grey".into(),
            Color::DarkGrey => "dark grey".into(),
            Color::Red => "red".into(),
            Color::Green => "green".into(),
            Color::Yellow => "yellow".into(),
            Color::Blue => "blue".into(),
            Color::Magenta => "magenta".into(),
            Color::Cyan => "cyan".into(),
            Color::White => "white".into(),
        }
    }
}

impl From<&str> for ColorDef {
    fn from(value: &str) -> ColorDef {
        ColorDef::Named(value.to_owned())
    }
}

impl From<CharacterMapImage> for CharmieDef {
    fn from(charmi: CharacterMapImage) -> Self {
        let mut used_chars = HashSet::new();
        let mut colors = HashSet::new();
        for row in charmi.rows.iter() {
            for segment in row.segments.iter() {
                match segment {
                    CharmieSegment::Empty { .. } => {},
                    CharmieSegment::Effect { style, .. } => {
                        if let Some(color) = style.foreground_color {
                            colors.insert(color);
                        }
                        if let Some(color) = style.background_color {
                            colors.insert(color);
                        }
                    },
                    CharmieSegment::HalfChar {
                        style,
                        replace_char,
                        ..
                    } => {
                        if let Some(replace_char) = replace_char {
                            used_chars.insert(*replace_char);
                        }
                        if let Some(color) = style.foreground_color {
                            colors.insert(color);
                        }
                        if let Some(color) = style.background_color {
                            colors.insert(color);
                        }
                    },
                    CharmieSegment::Textual { text, style } => {
                        for ch in text.chars() {
                            used_chars.insert(ch);
                        }
                        if let Some(color) = style.foreground_color {
                            colors.insert(color);
                        }
                        if let Some(color) = style.background_color {
                            colors.insert(color);
                        }
                    },
                }
            }
        }
        let gap_char: char = gap_char_iter()
            .filter(|ch| !used_chars.contains(ch))
            .next()
            .expect("there should be enough valid characters");
        let color_chars: HashMap<Color, char> = colors.into_iter().zip(color_char_iter()).collect();
        let mut text = String::new();
        let mut fg = String::new();
        let mut bg = String::new();
        for row in charmi.rows.iter() {
            for segment in row.segments.iter() {
                match segment {
                    CharmieSegment::Empty { len } => {
                        for _ in 0..*len {
                            text.push(gap_char);
                            fg.push(' ');
                            bg.push(' ');
                        }
                    },
                    CharmieSegment::Effect { len, style } => {
                        let bg_char = style
                            .background_color
                            .as_ref()
                            .map(|color| color_chars[color])
                            .unwrap_or(' ');
                        let fg_char = style
                            .foreground_color
                            .as_ref()
                            .map(|color| color_chars[color])
                            .unwrap_or(' ');
                        for _ in 0..*len {
                            text.push(gap_char);
                            fg.push(fg_char);
                            bg.push(bg_char);
                        }
                    },
                    CharmieSegment::HalfChar {
                        style,
                        replace_char,
                        ..
                    } => {
                        let bg_char = style
                            .background_color
                            .as_ref()
                            .map(|color| color_chars[color])
                            .unwrap_or(' ');
                        let fg_char = style
                            .foreground_color
                            .as_ref()
                            .map(|color| color_chars[color])
                            .unwrap_or(' ');
                        text.push(replace_char.unwrap_or(' '));
                        fg.push(fg_char);
                        bg.push(bg_char);
                    },
                    CharmieSegment::Textual {
                        text: seg_text,
                        style,
                    } => {
                        let bg_char = style
                            .background_color
                            .as_ref()
                            .map(|color| color_chars[color])
                            .unwrap_or(' ');
                        let fg_char = style
                            .foreground_color
                            .as_ref()
                            .map(|color| color_chars[color])
                            .unwrap_or(' ');
                        for ch in seg_text.chars() {
                            text.push(ch);
                            for _ in 0..ch.width().unwrap_or_default() {
                                fg.push(fg_char);
                                bg.push(bg_char);
                            }
                        }
                    },
                }
            }
            text = text.trim_end_matches(gap_char).to_string();
            fg = fg.trim_end_matches(' ').to_string();
            bg = bg.trim_end_matches(' ').to_string();
            text.push('\n');
            fg.push('\n');
            bg.push('\n');
        }
        text = text.trim_end_matches('\n').to_string();
        fg = fg.trim_end_matches('\n').to_string();
        bg = bg.trim_end_matches('\n').to_string();

        let text = if text.is_empty() {
            None
        } else {
            text.push('\n');
            Some(text)
        };
        let fg = if fg.is_empty() {
            None
        } else {
            fg.push('\n');
            Some(fg)
        };
        let bg = if bg.is_empty() {
            None
        } else {
            bg.push('\n');
            Some(bg)
        };

        let gap_char = if gap_char == ' ' {
            None
        } else {
            Some(gap_char)
        };
        let colors: HashMap<char, ColorDef> = color_chars
            .into_iter()
            .map(|(color, chr)| (chr, color.into()))
            .collect();
        let colors = if colors.is_empty() {
            None
        } else {
            Some(colors)
        };
        let values = if gap_char.is_none() && colors.is_none() {
            None
        } else {
            Some(Values {
                gap: gap_char,
                colors,
                attr: None,
            })
        };

        CharmieDef {
            text,
            fg,
            bg,
            values,
            attr: None,
        }
    }
}

impl From<CharmieDef> for CharacterMapImage {
    fn from(value: CharmieDef) -> Self {
        let values = value.values.unwrap_or_default();
        let gap_ch = values.gap.unwrap_or(' ');
        let color_map: HashMap<char, Color> = values
            .colors
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(k, v)| Some((k, v.try_into().ok()?)))
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
    char_map: HashMap<char, Color>,
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

fn gap_char_iter() -> impl Iterator<Item = char> {
    // Not allowed = '\' or '"'
    " -_=~*+,./;!#$%&':?@^`|{}[]<>()0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        .chars()
        .chain('\u{80}'..)
        .filter(|ch| ch.width() == Some(1))
}

fn color_char_iter() -> impl Iterator<Item = char> {
    // Not allowed = ' ', '\' or '"'
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!#$%&'()*+,-./0123456789:;<=>?@[]^_`{|}~"
        .chars()
        .chain('\u{80}'..)
        .filter(|ch| ch.width() == Some(1))
}

#[cfg(test)]
mod test {
    use test_log::test;

    use super::*;

    fn test_character_map_image() -> CharacterMapImage {
        let mut charmi: CharacterMapImage = CharacterMapImage::new();
        let orange = Color::AnsiValue(208);
        let white = Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        charmi.push_row(
            CharmieRow::of_gap(2)
                .with_char('y', &ContentStyle::new().yellow().on_blue())
                .with_char('g', &ContentStyle::new().on_green())
                .with_char('b', &ContentStyle::new().blue().on_yellow())
                .with_gap(2),
        );
        charmi.push_row(
            CharmieRow::of_gap(1)
                .with_styled_text("o".stylize().with(orange).on_dark_blue())
                .with_gap(3)
                .with_styled_text("i".stylize().dark_blue().on(orange))
                .with_gap(1),
        );
        charmi.push_row(
            CharmieRow::of_char('r', &ContentStyle::new().red().on_dark_magenta())
                .with_gap(1)
                .with_plain_text("=")
                .with_styled_text("0".stylize().black().on(white))
                .with_effect(1, &ContentStyle::new().black().on(white))
                .with_gap(1)
                .with_styled_text("v".stylize().dark_magenta().on_red()),
        );

        charmi
    }

    #[test]
    fn character_iterators_validity() {
        let first1000: HashSet<char> = gap_char_iter().take(1000).collect();
        // Must contain 1000 unique characters, must contain only valid ones and the first should
        // be space
        assert_eq!(first1000.len(), 1000);
        assert!(first1000.contains(&' '));
        assert!(!first1000.contains(&'\"'));
        assert!(!first1000.contains(&'\\'));
        assert_eq!(Some(' '), gap_char_iter().next());

        let first1000: HashSet<char> = color_char_iter().take(1000).collect();
        // Must contain 1000 unique characters, must contain only valid ones and space is invalid
        assert_eq!(first1000.len(), 1000);
        assert!(!first1000.contains(&' '));
        assert!(!first1000.contains(&'\"'));
        assert!(!first1000.contains(&'\\'));
    }

    #[test]
    fn charmi_to_definition_and_back() {
        let charmi = test_character_map_image();
        let charmi_def: CharmieDef = charmi.clone().into();
        println!("Charmie Def:\n{:?}\n\n", charmi_def);
        let back_charmi = charmi_def.into();
        assert_eq!(charmi, back_charmi);
    }

    #[test]
    fn charmi_to_definition_to_toml_and_back() {
        let charmi = test_character_map_image();
        let charmi_def: CharmieDef = charmi.clone().into();
        let toml_str = toml::to_string(&charmi_def)
            .expect("charmie definition should deserialize successfully");
        println!("TOML for charmi:\n{}", toml_str);
        let back_charmi_def: CharmieDef = toml::from_str(toml_str.as_str())
            .expect("conversion to charmie definition should succeed");
        assert_eq!(charmi_def, back_charmi_def);

        let back_charmi = charmi_def.into();
        assert_eq!(charmi, back_charmi);
    }

    #[test]
    fn load_test_charmi_file() {
        let mut test_charmi = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_charmi.push("assets/test.charmi");
        let result_str = std::fs::read_to_string(test_charmi).expect("test file should exist");
        log::debug!("CHARMI STR: {:?}", result_str);

        let charmie_def: CharmieDef =
            toml::from_str(result_str.as_str()).expect("test definition should parse successfully");

        let charmi: CharacterMapImage = charmie_def.into();
        let expected: CharacterMapImage = test_character_map_image();
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
