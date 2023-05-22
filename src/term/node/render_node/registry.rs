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
                ("mon", "$$"),
                ("access_point", "@@"),
                ("curio:hack", "hk"),
                // Considered alternatives "🃁 ", "♠♥", "==", "++", "&]", "□]"
                ("pickup:card", "🂠 "),
            ]
            .into_iter()
            .map(|(name, glyph)| (name.to_owned(), glyph.to_owned()))
            .collect(),
        }
    }
}
