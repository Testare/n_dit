use bevy::ecs::system::SystemParam;
use charmi::CharacterMapImage;
use game_core::node::{InNode, Node};
use game_core::player::Player;
use game_core::prelude::*;

use super::{NodePieceQItem, SimpleSubmenu};
use crate::layout::CalculatedSizeTty;

#[derive(Component, Debug, Default)]
pub struct MenuUiStats;

#[derive(SystemParam)]
pub struct MenuUiStatsDataParam<'w, 's> {
    player_node: Query<'w, 's, &'static InNode, With<Player>>,
    node_grids: Query<'w, 's, &'static EntityGrid, With<Node>>,
}

impl SimpleSubmenu for MenuUiStats {
    const NAME: &'static str = "Menu Stats";
    type RenderSystemParam = MenuUiStatsDataParam<'static, 'static>;
    type UiBundleExtras = ();

    fn height(selected: &NodePieceQItem<'_>) -> Option<usize> {
        let stats_to_display = if selected.max_size.is_some() { 1 } else { 0 }
            + if selected.speed.is_some() { 1 } else { 0 };
        if stats_to_display > 0 {
            Some(stats_to_display + 1)
        } else {
            None
        }
    }

    fn render(
        player: Entity,
        selected: &NodePieceQItem,
        size: &CalculatedSizeTty,
        node_ui_data: &MenuUiStatsDataParam,
    ) -> Option<CharacterMapImage> {
        if selected.max_size.is_some() || selected.speed.is_some() {
            let mut stats = vec![format!("{0:─<1$}", "─Stats", size.width())];
            if let Some(max_size) = selected.max_size {
                let InNode(node_id) = node_ui_data.player_node.get(player).ok()?;
                let grid = node_ui_data.node_grids.get(*node_id).ok()?;
                let size = grid.len_of(selected.entity);
                stats.push(format!("Size:  {}/{}", size, **max_size));
            }
            if let Some(speed) = selected.speed {
                let moves_taken = selected
                    .moves_taken
                    .map(|moves_taken| **moves_taken)
                    .unwrap_or(0);
                stats.push(format!("Moves: {}/{}", moves_taken, **speed));
            }
            Some(stats.into_iter().collect())
        } else {
            None
        }
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {}
}
