use charmi::CharacterMapImage;
use game_core::node::{Curio, Pickup};
use game_core::prelude::*;
use game_core::registry::Reg;

use super::{NodePieceQItem, SimpleSubmenu};
use crate::layout::CalculatedSizeTty;
use crate::node_ui::NodeGlyph;

#[derive(Component, Debug, Default)]
pub struct MenuUiLabel;

impl SimpleSubmenu for MenuUiLabel {
    const NAME: &'static str = "Menu Label";
    type UiBundleExtras = ();
    type RenderSystemParam = Res<'static, Reg<NodeGlyph>>;

    fn height(_: &NodePieceQItem<'_>) -> Option<usize> {
        Some(2)
    }

    fn render(
        _player: Entity,
        selected: &NodePieceQItem,
        _size: &CalculatedSizeTty,
        glyph_registry: &Res<Reg<NodeGlyph>>,
    ) -> Option<CharacterMapImage> {
        let display_id = selected.piece.display_id();
        let glyph = (**glyph_registry).get(display_id).unwrap_or_default();

        let is_tapped = selected
            .is_tapped
            .map(|is_tapped| **is_tapped)
            .unwrap_or(false);
        let label = CharacterMapImage::new()
            .with_row(|row| {
                let row = row
                    .with_plain_text("[")
                    .with_styled_text(glyph.styled_glyph())
                    .with_plain_text("]");
                if is_tapped {
                    row.with_plain_text(" (tapped)")
                } else {
                    row
                }
            })
            .with_row(|row| {
                if selected.access_point.is_some() {
                    row.with_plain_text("Access Point")
                } else if let Some(name) = selected
                    .curio
                    .map(Curio::name)
                    .or_else(|| {
                        selected.pickup.map(|pickup| match pickup {
                            Pickup::Mon(_) => "Mon",
                            Pickup::Card(_) => "Card: ??",
                            Pickup::Item(_) => "Item: ??",
                            Pickup::MacGuffin => "Intelligence", // TODO Need to configure these labels
                        })
                    })
                    .map(str::to_owned)
                {
                    row.with_text(name, &glyph.style())
                } else {
                    row
                }
            });

        Some(label)
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {}
}
