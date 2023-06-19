use bevy::utils::HashMap;
use crossterm::style::Color;
use game_core::prelude::*;

use crate::term::configuration::UiFormat;

#[derive(Deref, DerefMut, Resource)]
pub struct GlyphRegistry {
    registry: HashMap<String, (String, UiFormat)>,
}

impl Default for GlyphRegistry {
    fn default() -> Self {
        GlyphRegistry {
            registry: [
                // Alternatives to consider: <>, @@, {}
                (
                    "env:access_point",
                    (
                        "@@",
                        UiFormat::new(Some(Color::Black), Some(Color::Green), None),
                    ),
                ),
                ("curio:bug", ("b ", UiFormat::fgv(132, 252, 0))),
                ("curio:hack", ("hk", UiFormat::fgv(0, 199, 252))),
                ("curio:sling", (">-", UiFormat::fgv(0, 217, 165))),
                (
                    "curio:data_doctor_pro",
                    ("+ ", UiFormat::fgbgv(255, 0, 0, 0, 0, 200)),
                ),
                ("curio:death", ("死", UiFormat::fg(Color::Red))),
                ("curio:bit_man", ("01", UiFormat::fgv(182, 252, 0))),
                // Considered alternatives "🃁 ", "♠♥", "==", "++", "&]", "□]"
                // Looks good in this font, but not as good in other fonts
                ("pickup:card", ("🂠 ", UiFormat::fg(Color::Yellow))),
                ("pickup:mon", ("$$", UiFormat::fg(Color::Yellow))),
            ]
            .into_iter()
            .map(|(name, (glyph, format))| (name.to_owned(), (glyph.to_owned(), format)))
            .collect(),
        }
    }
}
