use std::sync::OnceLock;

use charmi::ColorDef;
use crossterm::style::{Color, ContentStyle, StyledContent, Stylize};
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

    pub fn style(&self) -> ContentStyle {
        match self {
            NodeGlyph::PlainGlyph(_) => ContentStyle::new(),
            NodeGlyph::ColoredGlyph(_, fg) => {
                ContentStyle::new().with(fg.try_into().unwrap_or(Color::White))
            },
            NodeGlyph::NameAndBothColors(_, fg, bg) => ContentStyle::new()
                .with(fg.try_into().unwrap_or(Color::White))
                .on(bg.try_into().unwrap_or(Color::Black)),
        }
    }

    pub fn styled_glyph(&self) -> StyledContent<String> {
        match self {
            NodeGlyph::PlainGlyph(glyph) => glyph.clone().stylize(),
            NodeGlyph::ColoredGlyph(glyph, fg) => {
                glyph.clone().with(fg.try_into().unwrap_or(Color::White))
            },
            NodeGlyph::NameAndBothColors(glyph, fg, bg) => glyph
                .clone()
                .with(fg.try_into().unwrap_or(Color::White))
                .on(bg.try_into().unwrap_or(Color::Black)),
        }
    }
}

impl Registry for NodeGlyph {
    const REGISTRY_NAME: &'static str = "term:node_glyphs";
    type Value = NodeGlyph;
}
