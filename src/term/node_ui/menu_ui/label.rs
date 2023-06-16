use game_core::node::{Curio, Pickup};
use game_core::prelude::*;

use super::super::registry::GlyphRegistry;
use super::{NodePieceQItem, SimpleSubmenu};
use crate::term::layout::CalculatedSizeTty;

#[derive(Component, Debug, Default)]
pub struct MenuUiLabel;

impl SimpleSubmenu for MenuUiLabel {
    const NAME: &'static str = "Menu Label";
    type UiBundleExtras = ();
    type RenderSystemParam = Res<'static, GlyphRegistry>;

    fn height(_: &NodePieceQItem<'_>) -> Option<usize> {
        Some(2)
    }

    fn render(
        _player: Entity,
        selected: &NodePieceQItem,
        _size: &CalculatedSizeTty,
        glyph_registry: &Res<GlyphRegistry>,
    ) -> Option<Vec<String>> {
        let display_id = selected.piece.display_id();
        let glyph = (**glyph_registry)
            .get(display_id)
            .map(|s| s.as_str())
            .unwrap_or("??");

        let is_tapped = selected
            .is_tapped
            .map(|is_tapped| **is_tapped)
            .unwrap_or(false);

        let mut label = vec![format!(
            "[{}]{}",
            glyph,
            if is_tapped { " (tapped)" } else { "" }
        )];
        if selected.access_point.is_some() {
            label.push("Access Point".to_owned());
        } else if let Some(name) = selected
            .curio
            .map(Curio::name)
            .or_else(|| {
                selected.pickup.map(|pickup| match pickup {
                    Pickup::Mon(_) => "Mon",
                    Pickup::Card(_) => "Card: ??",
                    Pickup::Item(_) => "Item: ??",
                })
            })
            .map(str::to_owned)
        {
            label.push(name);
        }
        Some(label)
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        ()
    }
}
