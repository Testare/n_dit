use std::sync::OnceLock;

use charmi::style::CharmiStyle;
use charmi::ColorDef;
use game_core::registry::Registry;
use serde::{Deserialize, Serialize};

static DEFAULT_GLYPH: OnceLock<NodeGlyph> = OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeGlyph {
    PlainGlyph(String),
    ColoredGlyph(String, ColorDef),
    NameAndBothColors(String, ColorDef, ColorDef),
}

impl Default for NodeGlyph {
    fn default() -> Self {
        NodeGlyph::PlainGlyph("??".to_string())
    }
}

impl Default for &NodeGlyph {
    fn default() -> Self {
        DEFAULT_GLYPH.get_or_init(NodeGlyph::default)
    }
}

impl NodeGlyph {
    pub fn glyph(&self) -> String {
        match self {
            NodeGlyph::PlainGlyph(glyph) => glyph.clone(),
            NodeGlyph::ColoredGlyph(glyph, _) => glyph.clone(),
            NodeGlyph::NameAndBothColors(glyph, _, _) => glyph.clone(),
        }
    }

    pub fn style(&self) -> CharmiStyle {
        match self {
            NodeGlyph::PlainGlyph(_) => CharmiStyle::default(),
            NodeGlyph::ColoredGlyph(_, fg) => CharmiStyle::default().fg(fg),
            NodeGlyph::NameAndBothColors(_, fg, bg) => CharmiStyle::default().fg(fg).bg(bg),
        }
    }
}

impl Registry for NodeGlyph {
    const REGISTRY_NAME: &'static str = "term:node_glyphs";
    type Value = NodeGlyph;
}
