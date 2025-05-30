use charmi::CharmiImage;
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
    ) -> Option<CharmiImage> {
        let display_id = selected.piece.display_id();
        let glyph = (**glyph_registry).get(display_id).unwrap_or_default();

        let is_tapped = selected
            .is_tapped
            .map(|is_tapped| **is_tapped)
            .unwrap_or(false);
        let mut charmi = CharmiImage::build_dynamic();
        charmi
            .add_text("[")
            .style(glyph.style())
            .add_text(&glyph.glyph())
            .no_style()
            .add_text("]");
        if is_tapped {
            charmi.add_text(" (tapped)");
        }
        charmi.next_line();
        if selected.access_point.is_some() {
            charmi.add_line("Access Point");
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
            charmi.style(&glyph.style()).add_line(&name).no_style();
        } else {
            charmi.next_line();
        }
        Some(charmi.build())
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {}
}
