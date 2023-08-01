use game_core::node::Node;
use game_core::player::ForPlayer;

use super::{NodeCursor, NodeUiQ, ShowNode};
use crate::animation::AnimationPlayer;
use crate::layout::{StyleTty, UiFocusBundle, UiFocusCycleOrder};
use crate::node_ui::grid_ui::{GridUi, GridUiAnimation};
use crate::node_ui::menu_ui::{
    MenuUiActions, MenuUiCardSelection, MenuUiDescription, MenuUiLabel, MenuUiStats,
};
use crate::node_ui::{
    AvailableActionTargets, AvailableMoves, NodeUi, SelectedAction, SelectedEntity,
};
use crate::prelude::*;
use crate::render::TerminalRendering;
use crate::TerminalWindow;

pub fn create_node_ui(
    mut commands: Commands,
    mut show_node: EventReader<ShowNode>,
    mut terminal_window: ResMut<TerminalWindow>,
    node_qs: Query<NodeUiQ, With<Node>>,
) {
    use taffy::prelude::*;
    if let Some(ShowNode { player, node }) = show_node.iter().next() {
        if let Ok(node_q) = node_qs.get(*node) {
            let render_root = commands
                .spawn((
                    StyleTty(taffy::prelude::Style {
                        size: Size {
                            width: Dimension::Points(100.),
                            height: Dimension::Points(100.),
                        },
                        flex_direction: FlexDirection::Column,
                        ..default()
                    }),
                    Name::new("Node UI Root"),
                    crate::layout::LayoutRoot,
                    TerminalRendering::default(),
                ))
                .with_children(|root| {
                    root.spawn(super::titlebar_ui::TitleBarUi::bundle(*player, &node_q));
                    root.spawn((
                        StyleTty(taffy::prelude::Style {
                            size: Size {
                                width: Dimension::Auto,
                                height: Dimension::Auto,
                            },
                            flex_grow: 1.0,
                            flex_shrink: 0.0,
                            ..default()
                        }),
                        Name::new("Node Content Pane"),
                    ))
                    .with_children(|content_pane| {
                        content_pane
                            .spawn((
                                StyleTty(taffy::prelude::Style {
                                    size: Size {
                                        width: Dimension::Points(14.),
                                        height: Dimension::Auto,
                                    },
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                }),
                                Name::new("Menu Bar"),
                            ))
                            .with_children(|menu_bar| {
                                menu_bar.spawn(MenuUiLabel::bundle(*player, &node_q));
                                menu_bar
                                    .spawn(MenuUiCardSelection::bundle(*player, &node_q))
                                    .insert(UiFocusCycleOrder(2));
                                menu_bar.spawn(MenuUiStats::bundle(*player, &node_q));
                                menu_bar
                                    .spawn(MenuUiActions::bundle(*player, &node_q))
                                    .insert(UiFocusCycleOrder(1));
                                menu_bar.spawn(MenuUiDescription::bundle(*player, &node_q));
                            });
                        content_pane
                            .spawn(GridUi::bundle(*player, &node_q))
                            .insert(UiFocusCycleOrder(0));
                    });
                    root.spawn(super::MessageBarUi::bundle(*player, &node_q));
                })
                .id();

            // This could probably live in GridUiPlugin when node loading is formalized
            commands.spawn((
                Name::new("GridAnimationPlayer"),
                GridUiAnimation,
                ForPlayer(*player),
                AnimationPlayer::default(),
                TerminalRendering::default(),
            ));

            commands.entity(*player).insert((
                NodeCursor::default(),
                SelectedEntity(node_q.grid.item_at(default())),
                SelectedAction(None),
                AvailableActionTargets::default(),
                UiFocusBundle::default(),
                AvailableMoves::default(),
            ));
            terminal_window.set_render_target(Some(render_root));
        }
    }
}
