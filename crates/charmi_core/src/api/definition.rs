use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize, Serializer};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::color::COLOR_CODE_TO_NAME;
use crate::{CharCell, CharmiActor, CharmiAnimation, CharmiAnimationFrame, CharmiImage};

static _PREFERRED_CHARACTERS: LazyLock<HashMap<u32, char>> = LazyLock::new(|| {
    [
        (0, 'V'),
        (1, 'R'),
        (2, 'G'),
        (3, 'Y'),
        (4, 'B'),
        (5, 'P'),
        (6, 'C'),
        (7, 'W'),
        (8, 'v'),
        (9, 'r'),
        (10, 'g'),
        (11, 'y'),
        (12, 'b'),
        (13, 'p'),
        (14, 'c'),
        (15, 'w'),
    ]
    .into_iter()
    .collect()
});

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ColorDef {
    Named(String), // TODO BEFOREMERGE replace String with Cow<'static, str>
    Ansi(u8),
    Rgb(u8, u8, u8),
    // Argb -> ???
}

impl ColorDef {
    pub fn recognized_color(name: &str) -> bool {
        super::color::COLOR_NAME_TO_CODE.contains_key(name)
    }
}

impl From<String> for ColorDef {
    fn from(value: String) -> Self {
        Self::Named(value)
    }
}

impl From<(u8,)> for ColorDef {
    fn from(value: (u8,)) -> Self {
        Self::Ansi(value.0)
    }
}

impl From<u8> for ColorDef {
    fn from(value: u8) -> Self {
        Self::Ansi(value)
    }
}

impl From<(u8, u8, u8)> for ColorDef {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::Rgb(r, g, b)
    }
}

#[derive(Debug, Default, Eq, Deserialize, Serialize, PartialEq)]
pub struct CharmiDef {
    pub text: Option<String>,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub attr: Option<String>,
    // TODO actually support this
    pub split: Option<String>,
    pub values: Option<Values>,
}

#[derive(Clone, Debug, Default, Eq, Deserialize, Serialize, PartialEq)]
pub struct Values {
    #[serde(serialize_with = "char_map_serialize")]
    pub colors: Option<HashMap<char, ColorDef>>,
    pub attr: Option<HashMap<char, String>>,
    // TODO BEFOREMERGE add "split" Option<String>
    pub split: Option<String>,
    pub gap: Option<char>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CharmiFrameDef {
    #[serde(flatten)]
    pub charmi: CharmiDef,
    pub timing: f32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CharmiAnimationDef {
    #[serde(rename = "f")]
    pub frames: Vec<CharmiFrameDef>,
    pub values: Option<Values>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CharmiActorDef {
    #[serde(rename = "a")]
    pub animations: HashMap<String, CharmiAnimationDef>,
    pub values: Option<Values>,
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
            split: rhs.split.as_ref().or(self.split.as_ref()).cloned(),
        }
    }
}

impl CharmiDef {
    fn with_additional_values(mut self, values: &Option<Values>) -> Self {
        self.values = match (self.values.as_ref(), values) {
            (Some(prev_values), Some(new_values)) => Some(new_values + prev_values),
            (Some(values), None) | (None, Some(values)) => Some(values.clone()),
            (None, None) => None,
        };
        self
    }
}

impl CharmiAnimationDef {
    fn with_additional_values(mut self, values: &Option<Values>) -> Self {
        self.values = match (self.values.as_ref(), values) {
            (Some(prev_values), Some(new_values)) => Some(new_values + prev_values),
            (Some(values), None) | (None, Some(values)) => Some(values.clone()),
            (None, None) => None,
        };
        self
    }
}

impl TryFrom<&ColorDef> for u32 {
    type Error = ();
    fn try_from(value: &ColorDef) -> Result<Self, Self::Error> {
        match value {
            ColorDef::Ansi(ansi) => Ok(*ansi as u32),
            ColorDef::Rgb(r, g, b) => {
                Ok(CharmiImage::TRUE_COLOR | (*r as u32) << 16 | (*g as u32) << 8 | *b as u32)
            },
            ColorDef::Named(name) => super::color::COLOR_NAME_TO_CODE
                .get(&name.to_lowercase())
                .copied()
                .ok_or(()),
        }
    }
}

impl From<u32> for ColorDef {
    fn from(value: u32) -> Self {
        match value {
            CharmiImage::NO_COLOR | 0..16 => {
                ColorDef::Named(COLOR_CODE_TO_NAME[&value].to_string())
            },
            x if x < 0x100 => ColorDef::Ansi(x as u8),
            x => ColorDef::Rgb(x as u8, x as u8, x as u8),
        }
    }
}

impl From<&str> for ColorDef {
    fn from(value: &str) -> ColorDef {
        ColorDef::Named(value.to_owned())
    }
}

impl From<&CharmiDef> for CharmiImage {
    fn from(value: &CharmiDef) -> Self {
        let values = value.values.clone().unwrap_or_default();
        let gap_ch = values.gap.unwrap_or(' ');
        let color_map: HashMap<char, u32> = values
            .colors
            .unwrap_or_default()
            .iter()
            .filter_map(|(k, v)| {
                let v: Option<u32> = v.try_into().ok();
                if v.is_none() {
                    log::warn!("Unknown color {k:?} -> {v:?}")
                }
                Some((*k, v?))
            })
            .chain(std::iter::once((' ', CharmiImage::NO_COLOR)))
            .collect();
        let mut text_lines: Vec<&str> = value
            .text
            .as_ref()
            .map(|text| text.lines().collect())
            .unwrap_or_default();
        let mut fg_lines: Vec<&str> = value
            .fg
            .as_ref()
            .map(|text| text.lines().collect())
            .unwrap_or_default();
        let mut bg_lines: Vec<&str> = value
            .bg
            .as_ref()
            .map(|text| text.lines().collect())
            .unwrap_or_default();

        let height = text_lines.len().max(bg_lines.len()).max(fg_lines.len());

        fg_lines.resize_with(height, Default::default);
        text_lines.resize_with(height, Default::default);
        bg_lines.resize_with(height, Default::default);
        let width = text_lines
            .iter()
            .chain(fg_lines.iter())
            .chain(bg_lines.iter())
            .map(|s| s.width()) //UNICODE WIDTH
            .max()
            .unwrap_or_default();
        let mut cell_vec = Vec::with_capacity(width * height);

        for y in 0..height {
            let end_of_line = (y + 1) * width;
            let mut chars = text_lines[y].chars();
            let mut fgs = fg_lines[y].chars();
            let mut bgs = bg_lines[y].chars();
            while cell_vec.len() < end_of_line {
                let ch = chars.next().unwrap_or(gap_ch);
                let chv = if ch == gap_ch { 0 } else { ch as u32 };
                let chw = ch.width().unwrap_or(0);
                if chw == 0 {
                    continue;
                }
                // TODO make this a private function
                let fg = fgs
                    .next()
                    .and_then(|color_ch| color_map.get(&color_ch).copied())
                    .unwrap_or(CharmiImage::NO_COLOR);
                let bg = bgs
                    .next()
                    .and_then(|color_ch| color_map.get(&color_ch).copied()) // Note: unmapped characters can be used to fill gaps
                    .unwrap_or(CharmiImage::NO_COLOR);
                let attr = 0;
                let splitchar = ' ';
                // TODO add splitting support
                // TODO add attr support
                cell_vec.push(CharCell::new(chv, fg, bg, attr));
                if chw == 2 {
                    fgs.next();
                    bgs.next();
                    cell_vec.push(CharCell::new(
                        CharmiImage::SUPPRESSED_CHAR,
                        splitchar as u32,
                        ' ' as u32,
                        attr,
                    ))
                }
            }
        }
        unsafe {
            // SAFETY: We handle the suppressed char ourselves
            CharmiImage::from_raw_cells(width as u32, height as u32, cell_vec.into())
        }
    }
}

impl From<&CharmiImage> for CharmiDef {
    fn from(value: &CharmiImage) -> Self {
        fn clean_up_line(text: &mut String, trailing_ch: char) {
            text.truncate(
                text.rfind(|ch| ch != trailing_ch)
                    .map(|i| i + 1)
                    .unwrap_or(0),
            );
            text.push('\n');
        }

        fn clean_up_text(mut text: String) -> Option<String> {
            text.truncate(text.rfind(|ch| ch != '\n').map(|i| i + 1).unwrap_or(0));

            if text.is_empty() {
                None
            } else {
                text.push('\n');
                Some(text)
            }
        }
        let mut used_chars = HashSet::new();
        let mut colors = HashSet::new();
        for cell in value.cells.iter() {
            if let Ok(ch) = <char>::try_from(cell.ch) {
                used_chars.insert(ch);
                colors.insert(cell.fg);
                colors.insert(cell.bg);
            } else {
                used_chars
                    .insert(<char>::try_from(cell.fg).expect("split char should be valid char"));
                used_chars
                    .insert(<char>::try_from(cell.bg).expect("split char should be valid char"));
            }
        }
        colors.remove(&CharmiImage::NO_COLOR);
        let gap_ch = gap_char_iter()
            .find(|ch| !used_chars.contains(ch))
            .expect("gap char iter shouldn't run out");
        // TODO BEFOREMERGE map colors to preferred characters first
        let mut color_map: HashMap<u32, char> = colors.into_iter().zip(color_char_iter()).collect();
        let colors = (!color_map.is_empty()).then(|| {
            color_map
                .iter()
                .map(|(colornum, ch)| (*ch, (*colornum).into()))
                .collect()
        });
        color_map.insert(CharmiImage::NO_COLOR, ' ');
        let values: Values = Values {
            colors,
            gap: (gap_ch != ' ').then_some(gap_ch),
            split: None,
            attr: None,
        };
        let mut splitcharsl = String::new();
        let mut textl = String::new();
        let mut fgl = String::new();
        let mut bgl = String::new();
        for line in value.cells.chunks(value.width as usize) {
            for cell in line.iter() {
                if let Ok(ch) = <char>::try_from(cell.ch) {
                    if ch == '\0' {
                        textl.push(gap_ch);
                    } else {
                        textl.push(ch);
                    }
                    bgl.push(color_map[&cell.bg]);
                    fgl.push(color_map[&cell.fg]);
                    splitcharsl.push(' ');
                } else {
                    // ch is assumed to be SUPPRESSED_CHAR
                    let splitchar_left =
                        <char>::try_from(cell.bg).expect("split chars should still be valid char");
                    let splitchar_right =
                        <char>::try_from(cell.fg).expect("split chars should still be valid char");
                    bgl.push(' ');
                    fgl.push(' ');
                    splitcharsl.pop();
                    splitcharsl.push(splitchar_left);
                    splitcharsl.push(splitchar_right);
                }
            }
            clean_up_line(&mut textl, gap_ch);
            clean_up_line(&mut bgl, ' ');
            clean_up_line(&mut fgl, ' ');
            clean_up_line(&mut splitcharsl, ' ');
        }
        let text = clean_up_text(textl);
        let fg = clean_up_text(fgl);
        let bg = clean_up_text(bgl);
        let _splitchars = clean_up_text(splitcharsl);

        // NOTE: Conversion isn't necessarily lossless with regard to size: TOML might convert back to a smaller image. However,
        // if IMAGE is converted to TOML and then converted back as DE_IMAGE, DE_IMAGE.resize(IMAGE.width(), IMAGE>height()) should
        // be the same as IMAGE.
        CharmiDef {
            text,
            fg,
            bg,
            values: (values.colors.is_some() || values.gap.is_some()).then_some(values),
            attr: None,
            split: None,
        }
    }
}

impl From<CharmiAnimationDef> for CharmiAnimation {
    fn from(value: CharmiAnimationDef) -> Self {
        let CharmiAnimationDef { values, frames } = value;
        frames
            .into_iter()
            .map(|frame| {
                (
                    frame.timing,
                    CharmiImage::from(&frame.charmi.with_additional_values(&values)),
                )
            })
            .collect()
    }
}
impl From<CharmiAnimation> for CharmiAnimationDef {
    fn from(value: CharmiAnimation) -> Self {
        let CharmiAnimation { frames, timings } = value;

        let mut last_timing = 0.0;
        let frames = frames
            .into_iter()
            .zip(timings)
            .map(|(frame, timing)| {
                let CharmiAnimationFrame { charmi } = frame;
                let frame = CharmiFrameDef {
                    timing: timing - last_timing,
                    charmi: (&charmi).into(),
                };
                last_timing = timing;
                frame
            })
            .collect();

        CharmiAnimationDef {
            frames,
            values: None,
        }
    }
}

impl From<CharmiActorDef> for CharmiActor {
    fn from(value: CharmiActorDef) -> Self {
        let CharmiActorDef { animations, values } = value;
        animations
            .into_iter()
            .map(|(name, animation)| {
                (
                    name,
                    CharmiAnimation::from(animation.with_additional_values(&values)),
                )
            })
            .collect()
    }
}

impl From<CharmiActor> for CharmiActorDef {
    fn from(value: CharmiActor) -> Self {
        let CharmiActor { animations } = value;
        let animations = animations
            .into_iter()
            .map(|(name, animation)| (name, CharmiAnimationDef::from(animation)))
            .collect();
        CharmiActorDef {
            animations,
            values: None,
        }
    }
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

    mod utils {
        use super::*;
        // use crate::charmi_actor::{CharmiActor, CharmiAnimation};

        /// Should match `tests/data/test.charmia`
        pub fn test_charmi_actor() -> CharmiActor {
            [("spin", test_charmi_animation())].into_iter().collect()
        }

        /// Should match the "spin" animation in `tests/data/test.charmia`
        pub fn test_charmi_animation() -> CharmiAnimation {
            [
                (
                    50.0,
                    CharmiImage::build_dynamic()
                        .skip_cells(1)
                        .fg("red")
                        .bg("dark red")
                        .add_text("o")
                        .build(),
                ),
                (
                    50.0,
                    CharmiImage::build_dynamic()
                        .skip_cells(2)
                        .fg("yellow")
                        .bg("dark yellow")
                        .add_text("o")
                        .build(),
                ),
                (
                    50.0,
                    CharmiImage::build_dynamic()
                        .next_line()
                        .skip_cells(3)
                        .fg("green")
                        .bg("dark green")
                        .add_text("o")
                        .build(),
                ),
                (
                    50.0,
                    CharmiImage::build_dynamic()
                        .set_cursor(2, 2)
                        .fg("blue")
                        .bg("dark blue")
                        .add_text("o")
                        .build(),
                ),
                (
                    50.0,
                    CharmiImage::build_dynamic()
                        .set_cursor(1, 2)
                        .fg("magenta")
                        .bg("dark magenta")
                        .add_text("o")
                        .build(),
                ),
                (
                    50.0,
                    CharmiImage::build_dynamic()
                        .next_line()
                        .fg("white")
                        .bg("black")
                        .add_text("o")
                        .build(),
                ),
            ]
            .into_iter()
            .collect()
        }

        /// Should match `tests/data/test.charmi`
        pub fn test_character_map_image() -> CharmiImage {
            let orange = 208u32;
            let white = (255, 255, 255);
            #[rustfmt::skip]
            return CharmiImage::build_dynamic()
                .skip_cells(2)
                .fg("yellow").bg("blue").add_text("y")
                .no_fg().bg("green").add_text("g")
                .fg("blue").bg("yellow").add_text("b")
                .next_line()

                .skip_cells(1)
                .fg(orange).bg("dark blue").add_text("o")
                .add_gap(3)
                .fg("dark blue").bg(orange).add_text("i")
                .next_line()

                .fg("red").bg("dark magenta").add_text("r")
                .skip_cells(1)
                .no_fg().no_bg().add_text("=")
                .fg("black").bg(white).add_text("0")
                .add_style_effect(1)
                .skip_cells(1)
                .fg("dark magenta").bg("red").add_text("v")
                .build();
        }
    }

    #[test]
    fn from_simple_def() {
        // let def = Ch
        let def = CharmiDef {
            text: None,
            bg: Some("bagels".into()),
            fg: None,
            attr: None,
            values: None,
            split: None,
        };
        _ = CharmiImage::from(&def);
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
        let charmi = utils::test_character_map_image();
        println!("Test Charmi {charmi:?}");
        let charmi_def: CharmiDef = (&charmi).into();
        println!("Charmi Def:\n{:?}\n\n", charmi_def);
        let back_charmi = (&charmi_def).into();
        assert_eq!(charmi, back_charmi);
    }

    #[test]
    fn charmi_animation_to_definition_and_back() {
        let animation = utils::test_charmi_animation();
        let charmi_def: CharmiAnimationDef = animation.clone().into();
        println!("Charmi Animation Def:\n{:?}\n\n", charmi_def);
        let back = charmi_def.into();
        assert_eq!(animation, back);
    }

    #[test]
    fn charmi_actor_to_definition_and_back() {
        let actor = utils::test_charmi_actor();
        let charmi_def: CharmiActorDef = actor.clone().into();
        println!("Charmi Actor Def:\n{:?}\n\n", charmi_def);
        let back = charmi_def.into();
        assert_eq!(actor, back);
    }

    #[test]
    fn charmi_to_definition_to_toml_and_back() {
        let charmi = utils::test_character_map_image();
        let charmi_def: CharmiDef = (&charmi).into();
        let toml_str = toml::to_string(&charmi_def)
            .expect("charmi definition should deserialize successfully");
        println!("TOML for charmi:\n{}", toml_str);
        let back_charmi_def: CharmiDef = toml::from_str(toml_str.as_str())
            .expect("conversion to charmi definition should succeed");
        assert_eq!(charmi_def, back_charmi_def);

        let back_charmi = (&charmi_def).into();
        assert_eq!(charmi, back_charmi);
    }

    #[test]
    fn charmi_animation_to_definition_to_toml_and_back() {
        let charmi = utils::test_charmi_animation();
        let charmi_def: CharmiAnimationDef = charmi.clone().into();
        let toml_str = toml::to_string(&charmi_def)
            .expect("charmi definition should deserialize successfully");
        println!("TOML for charmi:\n{}", toml_str);
        let back_charmi_def: CharmiAnimationDef = toml::from_str(toml_str.as_str())
            .expect("conversion to charmi definition should succeed");
        assert_eq!(charmi_def, back_charmi_def);

        let back_charmi = charmi_def.into();
        assert_eq!(charmi, back_charmi);
    }

    #[test]
    fn charmi_actor_to_definition_to_toml_and_back() {
        let charmi = utils::test_charmi_actor();
        let charmi_def: CharmiActorDef = charmi.clone().into();
        let toml_str = toml::to_string(&charmi_def)
            .expect("charmi definition should deserialize successfully");
        println!("TOML for charmi:\n{}", toml_str);
        let back_charmi_def: CharmiActorDef = toml::from_str(toml_str.as_str())
            .expect("conversion to charmi definition should succeed");
        assert_eq!(charmi_def, back_charmi_def);

        let back_charmi = charmi_def.into();
        assert_eq!(charmi, back_charmi);
    }

    #[test]
    fn load_test_charmi_file() {
        let mut test_charmi = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_charmi.push("tests/data/test.charmi");
        let result_str = std::fs::read_to_string(test_charmi).expect("test file should exist");
        log::debug!("CHARMI STR: {:?}", result_str);

        let charmi_def: CharmiDef =
            toml::from_str(result_str.as_str()).expect("test definition should parse successfully");

        let charmi = CharmiImage::from(&charmi_def);
        let expected: CharmiImage = utils::test_character_map_image();
        assert_eq!(expected, charmi);
    }

    #[test]
    fn load_test_charmia_file() {
        let mut test_charmi = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_charmi.push("tests/data/test.charmia");
        let result_str = std::fs::read_to_string(test_charmi).expect("text file to exist");

        let charmi_def: CharmiActorDef =
            toml::from_str(result_str.as_str()).expect("test definition should parse successfully");

        let charmia = CharmiActor::from(charmi_def);
        let expected = utils::test_charmi_actor();
        assert_eq!(expected, charmia);
    }
}
