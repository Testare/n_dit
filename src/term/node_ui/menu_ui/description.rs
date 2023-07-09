use game_core::card::{Action, Actions, Description};
use game_core::node::NodePiece;
use game_core::player::{ForPlayer, Player};
use game_core::prelude::*;
use taffy::style::Dimension;

use super::{NodePieceQ, NodeUi, SelectedAction, SelectedEntity};
use crate::term::layout::{CalculatedSizeTty, FitToSize, StyleTty};
use crate::term::node_ui::NodeUiQItem;
use crate::term::render::{RenderTtySet, TerminalRendering};

#[derive(Component, Debug, Default)]
pub struct MenuUiDescription;

impl MenuUiDescription {
    fn style_update_system(
        node_pieces: Query<(Option<&Description>, Option<&Actions>), With<NodePiece>>,
        players: Query<(&SelectedEntity, &SelectedAction), With<Player>>,
        mut ui: Query<(&mut StyleTty, &CalculatedSizeTty, &ForPlayer), With<MenuUiDescription>>,
        mut last_nonzero_width: Local<usize>,
        action_descs: Query<&Description, With<Action>>,
    ) {
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
                                Some(action_descs.get(*action_id).ok()?.as_str())
                            })
                            .or_else(|| Some(piece_desc?.as_str()))?;
                        Some(textwrap::wrap(desc_str, *last_nonzero_width).len() as f32 + 1.0)
                    })
                    .unwrap_or(0.0);
                if Dimension::Points(new_height) != style.min_size.height {
                    style.min_size.height = Dimension::Points(new_height);
                    style.display = if new_height == 0.0 {
                        style.size.height = Dimension::Points(new_height);
                        taffy::style::Display::None
                    } else {
                        // Give a little extra for padding if we can
                        style.size.height = Dimension::Points(new_height);
                        taffy::style::Display::Flex
                    };
                }
            }
        }
    }

    fn render_system(
        mut commands: Commands,
        node_pieces: Query<NodePieceQ>,
        players: Query<(&SelectedEntity, &SelectedAction), With<Player>>,
        mut ui: Query<
            (
                Entity,
                &CalculatedSizeTty,
                &ForPlayer,
                &mut TerminalRendering,
            ),
            With<MenuUiDescription>,
        >,
        action_descs: Query<&Description, With<Action>>,
    ) {
        for (id, size, ForPlayer(player), mut tr) in ui.iter_mut() {
            if let Ok((selected_entity, selected_action)) = players.get(*player) {
                let rendering = selected_entity
                    .of(&node_pieces)
                    .and_then(|selected| {
                        let desc_str = selected_action
                            .and_then(|selected_action| {
                                let action_id = selected.actions?.get(selected_action)?;
                                Some(action_descs.get(*action_id).ok()?.as_str())
                            })
                            .or_else(|| Some(selected.description?.as_str()))?;
                        let wrapped_desc = textwrap::wrap(desc_str, size.width());
                        let mut menu = vec![format!("{0:-<1$}", "-Desc", size.width())];
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
        app.add_systems((
            Self::style_update_system.in_set(RenderTtySet::PreCalculateLayout),
            Self::render_system.in_set(RenderTtySet::PostCalculateLayout),
        ));
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
                height: Dimension::Points(0.0),
            },
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        ()
    }
}
