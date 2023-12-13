use charmi::CharacterMapImage;
use crossterm::event::KeyModifiers;
use game_core::card::{Action, Actions};
use game_core::node::{IsTapped, NodeOp, NodePiece};
use game_core::op::CoreOps;
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
use taffy::style::Dimension;

use crate::base_ui::{HoverPoint, Tooltip};
use crate::configuration::DrawConfiguration;
use crate::input_event::{MouseButton, MouseEventListener, MouseEventTty, MouseEventTtyKind};
use crate::key_map::NamedInput;
use crate::layout::{CalculatedSizeTty, StyleTty, UiFocus};
use crate::node_ui::node_ui_op::{FocusTarget, UiOps};
use crate::node_ui::{NodeUi, NodeUiOp, NodeUiQItem, SelectedAction, SelectedEntity};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};
use crate::{KeyMap, Submap};

#[derive(Component, Default, Debug)]
pub struct MenuUiActions;

impl MenuUiActions {
    pub fn kb_action_menu(
        mut ev_keys: EventReader<KeyEvent>,
        ast_actions: Res<Assets<Action>>,
        mut res_core_ops: ResMut<CoreOps>,
        mut res_ui_ops: ResMut<UiOps>,
        players: Query<(Entity, &UiFocus, &KeyMap, &SelectedEntity, &SelectedAction), With<Player>>,
        node_pieces: Query<(&Actions, Option<&IsTapped>), With<NodePiece>>,
        action_menu_uis: Query<(), With<MenuUiActions>>,
    ) {
        for KeyEvent { code, modifiers } in ev_keys.read() {
            for (player_id, focus, key_map, selected_entity, selected_action) in players.iter() {
                if (**focus)
                    .map(|focused_ui| !action_menu_uis.contains(focused_ui))
                    .unwrap_or(true)
                {
                    continue;
                }

                if let Some(named_input) =
                    key_map.named_input_for_key(Submap::Node, *code, *modifiers)
                {
                    if let Some((actions, is_tapped)) = selected_entity.of(&node_pieces) {
                        match named_input {
                            NamedInput::Direction(dir) => {
                                let actions_bound = actions.len();
                                let current_action = selected_action.unwrap_or(0);
                                let next_action = Some(
                                    (current_action
                                        + match dir {
                                            Compass::North => actions_bound - 1,
                                            Compass::South => 1,
                                            _ => 0,
                                        })
                                        % actions_bound,
                                );
                                if **selected_action != next_action {
                                    res_ui_ops.request(
                                        player_id,
                                        NodeUiOp::SetSelectedAction(next_action),
                                    );
                                }
                            },
                            NamedInput::MenuFocusNext | NamedInput::MenuFocusPrev => {
                                res_ui_ops.request(player_id, NodeUiOp::SetSelectedAction(None));
                            },
                            NamedInput::Activate => {
                                if is_tapped.map(|is_tapped| **is_tapped).unwrap_or(true) {
                                    res_ui_ops
                                        .request(player_id, NodeUiOp::SetSelectedAction(None));
                                } else if let Some(action) = actions
                                    .get(selected_action.unwrap_or_default())
                                    .and_then(|handle| ast_actions.get(handle))
                                {
                                    if action.range().is_none() {
                                        res_core_ops.request(
                                            player_id,
                                            NodeOp::PerformCurioAction {
                                                action_id: action.id_cow(),
                                                curio: **selected_entity,
                                                target: default(),
                                            },
                                        );
                                    }
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
    }

    pub fn mouse_action_menu(
        ast_actions: Res<Assets<Action>>,
        mut res_core_ops: ResMut<CoreOps>,
        mut res_ui_ops: ResMut<UiOps>,
        mut ev_mouse: EventReader<MouseEventTty>,
        node_pieces: Query<(&Actions, Option<&IsTapped>), With<NodePiece>>,
        players: Query<(&SelectedEntity, AsDerefCopied<SelectedAction>), With<Player>>,
        ui_actions: Query<&ForPlayer, With<MenuUiActions>>,
    ) {
        for layout_event in ev_mouse.read() {
            ui_actions
                .get(layout_event.entity())
                .ok()
                .and_then(|ForPlayer(player_id)| {
                    let (selected_entity, selected_action) = get_assert!(*player_id, players)?;
                    let (actions, is_tapped) = selected_entity.of(&node_pieces)?;

                    // TODO If curio is active and that action has no range, do it immediately. Perhaps if the button is "right", just show it
                    match layout_event.event_kind() {
                        MouseEventTtyKind::Down(MouseButton::Left)
                        | MouseEventTtyKind::DoubleClick => {
                            let clicked_action = if layout_event.relative_pos().y > 0
                                && layout_event.relative_pos().y <= actions.len() as u32
                            {
                                Some((layout_event.relative_pos().y - 1) as usize)
                            } else {
                                None
                            };
                            let double_click = layout_event.double_click()
                                || !layout_event
                                    .modifiers()
                                    .intersection(KeyModifiers::SHIFT | KeyModifiers::ALT)
                                    .is_empty();

                            let same_action_clicked = clicked_action == selected_action;

                            if clicked_action.is_none() {
                                res_ui_ops.request(
                                    *player_id,
                                    NodeUiOp::ChangeFocus(FocusTarget::ActionMenu),
                                );
                            } else if double_click {
                                res_ui_ops
                                    .request(*player_id, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                                let action = ast_actions.get(
                                    &actions[clicked_action
                                        .expect("clicked_action should be checked by this point")],
                                )?;
                                if matches!(is_tapped, Some(IsTapped(false))) // Should only match if the game has started
                                    && action.range().is_none()
                                {
                                    res_core_ops.request(
                                        *player_id,
                                        NodeOp::PerformCurioAction {
                                            action_id: action.id_cow(),
                                            curio: **selected_entity,
                                            target: default(),
                                        },
                                    );
                                } else {
                                    res_ui_ops.request(
                                        *player_id,
                                        NodeUiOp::SetSelectedAction(clicked_action),
                                    );
                                }
                            } else if same_action_clicked {
                                res_ui_ops
                                    .request(*player_id, NodeUiOp::ChangeFocus(FocusTarget::Grid));
                                res_ui_ops.request(*player_id, NodeUiOp::SetSelectedAction(None));
                            } else {
                                res_ui_ops.request(
                                    *player_id,
                                    NodeUiOp::ChangeFocus(FocusTarget::ActionMenu),
                                );
                                res_ui_ops.request(
                                    *player_id,
                                    NodeUiOp::SetSelectedAction(clicked_action),
                                );
                            }
                        },
                        _ => {},
                    }
                    Some(())
                });
        }
    }

    fn sys_adjust_style_action_menu(
        node_pieces: Query<&Actions, With<NodePiece>>,
        players: Query<&SelectedEntity, With<Player>>,
        mut ui: Query<(&mut StyleTty, &ForPlayer), With<MenuUiActions>>,
    ) {
        for (mut style, ForPlayer(player)) in ui.iter_mut() {
            if let Ok(selected_entity) = players.get(*player) {
                let new_height = selected_entity
                    .of(&node_pieces)
                    .map(|actions| (actions.len() + 1) as f32)
                    .unwrap_or(0.0);

                if Dimension::Points(new_height) != style.min_size.height {
                    style.min_size.height = Dimension::Points(new_height);
                    style.display = if new_height == 0.0 {
                        style.size.height = Dimension::Points(new_height);
                        taffy::style::Display::None
                    } else {
                        // Give a little extra for padding if we can
                        style.size.height = Dimension::Points(new_height + 1.0);
                        taffy::style::Display::Flex
                    };
                }
            }
        }
    }

    fn sys_render_action_menu(
        res_draw_config: Res<DrawConfiguration>,
        ast_actions: Res<Assets<Action>>,
        node_pieces: Query<&Actions, With<NodePiece>>,
        players: Query<(&SelectedEntity, &SelectedAction, &UiFocus), With<Player>>,
        mut ui: Query<
            (
                Entity,
                &CalculatedSizeTty,
                &ForPlayer,
                AsDeref<HoverPoint>,
                &mut TerminalRendering,
            ),
            With<MenuUiActions>,
        >,
    ) {
        // let render_param = render_param.into_inner();
        for (id, size, ForPlayer(player), hover_point, mut tr) in ui.iter_mut() {
            if let Ok((selected_entity, selected_action, focus)) = players.get(*player) {
                let rendering = selected_entity
                    .of(&node_pieces)
                    .map(|piece_actions| {
                        let title_style = if Some(id) == **focus {
                            res_draw_config.color_scheme().menu_title_hover()
                        } else {
                            res_draw_config.color_scheme().menu_title()
                        };
                        let hover_index = hover_point
                            .as_ref()
                            .and_then(|pt| (pt.y as usize).checked_sub(1));
                        let mut menu = CharacterMapImage::new();
                        let menu_title = format!("{0:─<1$}", "─Actions", size.width());
                        menu.new_row().add_text(menu_title, &title_style);

                        for (idx, action) in piece_actions.iter().enumerate() {
                            if let Some(action) = ast_actions.get(action) {
                                let style = (Some(idx) == hover_index)
                                    .then(|| res_draw_config.color_scheme().menu_hover())
                                    .unwrap_or_default();
                                let action_text = if Some(idx) == **selected_action {
                                    format!("▶{}", action.id())
                                } else {
                                    action.id().to_string()
                                };
                                menu.new_row().add_text(action_text, &style);
                            }
                        }
                        menu
                    })
                    .unwrap_or_default();
                tr.update_charmie(rendering);
            }
        }
    }
}

impl Plugin for MenuUiActions {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                Self::kb_action_menu.in_set(NDitCoreSet::ProcessInputs),
                Self::mouse_action_menu.in_set(NDitCoreSet::ProcessInputs),
            ),
        )
        .add_systems(
            RENDER_TTY_SCHEDULE,
            (
                Self::sys_adjust_style_action_menu.in_set(RenderTtySet::AdjustLayoutStyle),
                Self::sys_render_action_menu.in_set(RenderTtySet::PostCalculateLayout),
            ),
        );
    }
}

impl NodeUi for MenuUiActions {
    const NAME: &'static str = "Actions Menu";
    type UiBundleExtras = (MouseEventListener, HoverPoint, Tooltip);
    type UiPlugin = Self;

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
        (
            MouseEventListener,
            HoverPoint::default(),
            Tooltip::new("Select action"),
        )
    }
}
