use bevy::utils::HashMap;
use game_core::prelude::*;

#[derive(Deref, DerefMut, Resource)]
pub struct GlyphRegistry {
    registry: HashMap<String, String>,
}

impl Default for GlyphRegistry {
    fn default() -> Self {
        GlyphRegistry {
            registry: [
                // Alternatives to consider: <>, @@, {}
                ("env:access_point", "@@"),
                ("curio:hack", "hk"),
                ("curio:sling", ">-"),
                ("curio:data_doctor_pro", "死"),
                ("curio:death", "死"),
                // Considered alternatives "🃁 ", "♠♥", "==", "++", "&]", "□]"
                ("pickup:card", "🂠 "), // Looks good in this font, but not as good in other fonts
                ("pickup:mon", "$$"),
            ]
            .into_iter()
            .map(|(name, glyph)| (name.to_owned(), glyph.to_owned()))
            .collect(),
        }
    }
}
