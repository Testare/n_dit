use game_core::card::{Action, Actions, Description};
use game_core::node::NodePiece;
use game_core::player::{ForPlayer, Player};
use game_core::prelude::*;

use super::{NodePieceQ, NodeUi, SelectedAction, SelectedNodePiece};
use crate::layout::{CalculatedSizeTty, FitToSize, StyleTty};
use crate::node_ui::NodeUiQItem;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};

#[derive(Component, Debug, Default)]
pub struct MenuUiDescription;

impl MenuUiDescription {
    fn style_update_system(
        ast_actions: Res<Assets<Action>>,
        node_pieces: Query<(Option<&Description>, Option<&Actions>), With<NodePiece>>,
        players: Query<(&SelectedNodePiece, &SelectedAction), With<Player>>,
        mut ui: Query<(&mut StyleTty, &CalculatedSizeTty, &ForPlayer), With<MenuUiDescription>>,
        mut last_nonzero_width: Local<usize>,
    ) {
        use taffy::prelude::*;
        for (mut style, size, ForPlayer(player)) in ui.iter_mut() {
            if let Ok((selected_entity, selected_action)) = players.get(*player) {
                if size.width() != 0 {
                    *last_nonzero_width = size.width();
                }
                let new_height = selected_entity
                    .of(&node_pieces)
                    .and_then(|(piece_desc, actions)| {
                        let desc_str = selected_action
                            .and_then(|selected_action| {
                                let action_id = actions?.get(selected_action)?;
                                Some(ast_actions.get(action_id)?.description())
                            })
                            .or_else(|| Some(piece_desc?.as_str()))?;
                        Some(textwrap::wrap(desc_str, *last_nonzero_width).len() as f32 + 1.0)
                    })
                    .unwrap_or(0.0);
                if Dimension::Length(new_height) != style.min_size.height {
                    style.min_size.height = length(new_height);
                    style.display = if new_height == 0.0 {
                        style.size.height = length(new_height);
                        taffy::style::Display::None
                    } else {
                        // Give a little extra for padding if we can
                        style.size.height = length(new_height);
                        taffy::style::Display::Flex
                    };
                }
            }
        }
    }

    fn render_system(
        ast_actions: Res<Assets<Action>>,
        node_pieces: Query<NodePieceQ>,
        players: Query<(&SelectedNodePiece, &SelectedAction), With<Player>>,
        mut ui: Query<
            (&CalculatedSizeTty, &ForPlayer, &mut TerminalRendering),
            With<MenuUiDescription>,
        >,
    ) {
        for (size, ForPlayer(player), mut tr) in ui.iter_mut() {
            if let Ok((selected_entity, selected_action)) = players.get(*player) {
                let rendering = selected_entity
                    .of(&node_pieces)
                    .and_then(|selected| {
                        let desc_str = selected_action
                            .and_then(|selected_action| {
                                let action_id = selected.actions?.get(selected_action)?;
                                Some(ast_actions.get(action_id)?.description())
                            })
                            .or_else(|| Some(selected.description?.as_str()))?;
                        let wrapped_desc = textwrap::wrap(desc_str, size.width());
                        let mut menu = vec![format!("{0:─<1$}", "─Desc", size.width())];
                        for desc_line in wrapped_desc.into_iter() {
                            menu.push(desc_line.into_owned());
                        }
                        Some(menu)
                    })
                    .unwrap_or_default();
                tr.update(rendering.fit_to_size(size));
            }
        }
    }
}

impl Plugin for MenuUiDescription {
    fn build(&self, app: &mut App) {
        app.add_systems(
            RENDER_TTY_SCHEDULE,
            (
                Self::style_update_system.in_set(RenderTtySet::AdjustLayoutStyle),
                Self::render_system.in_set(RenderTtySet::PostCalculateLayout),
            ),
        );
    }
}

impl NodeUi for MenuUiDescription {
    const NAME: &'static str = "Description Menu";
    type UiPlugin = Self;
    type UiBundleExtras = ();

    fn initial_style(_: &NodeUiQItem) -> StyleTty {
        use taffy::prelude::*;

        StyleTty(taffy::prelude::Style {
            display: Display::None,
            min_size: Size {
                width: Dimension::Auto,
                height: length(0.0),
            },
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {}
}
