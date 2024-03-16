use std::borrow::Cow;
use std::ops::Deref;

use charmi::CharacterMapImage;
use game_core::card::{Action, ActionTarget, Actions};
use game_core::common::daddy::Daddy;
use game_core::node::{IsTapped, NodeOp, NodePiece, OnTeam, Team, TeamPhase};
use game_core::op::CoreOps;
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;

use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::base_ui::{HoverPoint, Tooltip};
use crate::configuration::DrawConfiguration;
use crate::input_event::{MouseEventListener, MouseEventTty, MouseEventTtyKind};
use crate::key_map::NamedInput;
use crate::layout::{CalculatedSizeTty, StyleTty, UiFocus};
use crate::linkage::base_ui_game_core;
use crate::main_ui::UiOps;
use crate::node_ui::node_context_actions::NodeContextActions;
use crate::node_ui::node_ui_op::FocusTarget;
use crate::node_ui::{NodeUi, NodeUiOp, NodeUiQItem, SelectedAction, SelectedNodePiece};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};
use crate::{KeyMap, Submap};

#[derive(Component, Default, Debug)]
pub struct MenuUiActions;

impl Plugin for MenuUiActions {
    fn build(&self, app: &mut App) {
        app.init_resource::<Daddy<MenuUiActionsCA>>()
            .add_systems(
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
            )
            .add_systems(
                Update,
                (sys_create_action_ca, sys_actions_menu_adjust_ca_hover).chain(),
            );
    }
}

#[derive(Component, Default, Debug, Deref, Reflect)]
pub struct MenuUiActionsCA(Vec<Entity>);

impl MenuUiActions {
    pub fn kb_action_menu(
        mut ev_keys: EventReader<KeyEvent>,
        ast_actions: Res<Assets<Action>>,
        mut res_core_ops: ResMut<CoreOps>,
        mut res_ui_ops: ResMut<UiOps>,
        players: Query<
            (
                Entity,
                &UiFocus,
                &KeyMap,
                &SelectedNodePiece,
                &SelectedAction,
            ),
            With<Player>,
        >,
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
        mut res_ui_ops: ResMut<UiOps>,
        mut ev_mouse: EventReader<MouseEventTty>,
        ui_actions: Query<&ForPlayer, With<MenuUiActions>>,
    ) {
        for layout_event in ev_mouse.read() {
            if !matches!(layout_event.event_kind(), MouseEventTtyKind::Down(_)) {
                continue;
            }
            if let Some(&ForPlayer(player_id)) = layout_event
                .top_entity()
                .and_then(|top_entity| ui_actions.get(top_entity).ok())
            {
                // TODO Perhaps add guardrails to do this only if selected actions is something
                res_ui_ops.request(player_id, NodeUiOp::ChangeFocus(FocusTarget::ActionMenu));
            };
        }
    }

    fn sys_adjust_style_action_menu(
        node_pieces: Query<&Actions, With<NodePiece>>,
        players: Query<&SelectedNodePiece, With<Player>>,
        mut ui: Query<(&mut StyleTty, &ForPlayer), With<MenuUiActions>>,
    ) {
        use taffy::prelude::*;
        for (mut style, ForPlayer(player)) in ui.iter_mut() {
            if let Ok(selected_entity) = players.get(*player) {
                let new_height = selected_entity
                    .of(&node_pieces)
                    .map(|actions| (actions.len() + 1) as f32)
                    .unwrap_or(0.0);

                if Dimension::Length(new_height) != style.min_size.height {
                    style.min_size.height = length(new_height);
                    style.display = if new_height == 0.0 {
                        style.size.height = length(new_height);
                        taffy::style::Display::None
                    } else {
                        // Give a little extra for padding if we can
                        style.size.height = length(new_height + 1.0);
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
        players: Query<(&SelectedNodePiece, &SelectedAction, &UiFocus), With<Player>>,
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

impl NodeUi for MenuUiActions {
    const NAME: &'static str = "Actions Menu";
    type UiBundleExtras = (MouseEventListener, HoverPoint, Tooltip, MenuUiActionsCA);
    type UiPlugin = Self;

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

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        (
            MouseEventListener,
            HoverPoint::default(),
            Tooltip::new("Select action (Double click to use during turn)"),
            MenuUiActionsCA::default(),
        )
    }
}

fn sys_create_action_ca(
    res_daddy: Res<Daddy<MenuUiActionsCA>>,
    mut commands: Commands,
    res_ast_actions: Res<Assets<Action>>,
    q_node_piece: Query<(Ref<Actions>, AsDerefCopied<OnTeam>), With<NodePiece>>,
    mut q_actions_ui: Query<(&ForPlayer, &mut MenuUiActionsCA), With<MenuUiActions>>,
    q_players: Query<(Ref<SelectedNodePiece>, &OnTeam), With<Player>>,
    q_team: Query<&TeamPhase, With<Team>>,
) {
    for (&ForPlayer(player_id), mut actions_menu_ca) in q_actions_ui.iter_mut() {
        get_assert!(player_id, q_players, |(
            selected_entity,
            &OnTeam(team_id),
        )| {
            let selected_entity_changed = selected_entity.is_changed();
            if let Some((actions, piece_team_id)) =
                selected_entity.and_then(|id| q_node_piece.get(id).ok())
            {
                if !actions.is_changed() && !selected_entity_changed {
                    return None;
                }
                // We don't need to check team_phase for change since actions always change during phase change
                let team_phase = get_assert!(team_id, q_team)?;
                for id in actions_menu_ca.0.drain(..) {
                    commands.entity(id).despawn();
                }
                commands.entity(**res_daddy).with_children(|daddy| {
                    for (i, action) in actions.iter().enumerate() {
                        let action_asset_id = action.id();
                        if let Some(action) = res_ast_actions.get(action_asset_id) {
                            let action_id = action.id();
                            let is_targetted_action = *action.target() != ActionTarget::None;
                            let ca_name =
                                if team_id == piece_team_id && *team_phase == TeamPhase::Play {
                                    format!("Perform [{action_id}]")
                                } else {
                                    format!("Display [{action_id}]")
                                };
                            let context_action = if is_targetted_action {
                                base_ui_game_core::context_action_from_op::<UiOps, _>(
                                    ca_name.as_str(),
                                    NodeUiOp::SetSelectedAction(Some(i)),
                                )
                            } else {
                                base_ui_game_core::context_action_from_op::<CoreOps, _>(
                                    ca_name.as_str(),
                                    NodeOp::PerformCurioAction {
                                        curio: **selected_entity,
                                        action_id: Cow::from(action_id.to_string()),
                                        target: default(),
                                    },
                                )
                            };
                            let id = daddy
                                .spawn((
                                    Name::new(format!(
                                        "Actions Menu CA[{i:2}]: [{action_id}] {}",
                                        if is_targetted_action {
                                            "(Target)"
                                        } else {
                                            "(NoTarget)"
                                        }
                                    )),
                                    context_action,
                                ))
                                .id();
                            actions_menu_ca.0.push(id);
                        } else {
                            log::error!("Error: Unloaded action [{action_asset_id}]");
                            let id = daddy.spawn((
                                Name::new("Error loading action CA"),
                                ContextAction::new("<LOG ERROR>", move |_id, _world| {
                                    log::error!("LOG ERROR CA: Action was not pre-loaded for node [{action_asset_id}]");
                                }),
                            )).id();
                            actions_menu_ca.0.push(id);
                        }
                    }
                });
            }
            Some(())
        });
    }
}

fn sys_actions_menu_adjust_ca_hover(
    res_node_ca: Res<NodeContextActions>,
    q_player: Query<Ref<SelectedAction>, With<Player>>,
    mut q_actions_ui: Query<
        (
            Entity,
            &ForPlayer,
            AsDeref<MenuUiActionsCA>,
            AsDerefCopied<HoverPoint>,
            &mut ContextActions,
        ),
        (With<MenuUiActions>,),
    >,
    q_actions_ui_changed: Query<
        (),
        (
            Or<(Changed<MenuUiActionsCA>, Changed<HoverPoint>)>,
            With<MenuUiActions>,
        ),
    >,
) {
    for (
        ui_id,
        &ForPlayer(player_id),
        menu_ui_actions_ca,
        hover_point,
        mut menu_ui_context_actions,
    ) in q_actions_ui.iter_mut()
    {
        let selected_action = get_assert!(player_id, q_player);
        let ui_changed = q_actions_ui_changed.contains(ui_id);
        let changes_occurred = ui_changed
            || selected_action
                .as_ref()
                .map(|sa| sa.is_changed())
                .unwrap_or(false);
        if !changes_occurred {
            continue;
        }
        let selected_action = selected_action.and_then(|sa| *sa.deref().deref());
        let deselect_ca = selected_action.and_then(|_| {
            let hover_y = hover_point?.y;
            if hover_y == 0 || hover_y as usize > menu_ui_actions_ca.len() {
                return None;
            }
            Some(res_node_ca.clear_selected_action())
        });
        let select_ca = hover_point.and_then(|pt| {
            let index = pt.y.checked_sub(1)? as usize;
            if Some(index) == selected_action {
                return None;
            }
            let ca = *menu_ui_actions_ca.get(index)?;
            Some(ca)
        });
        *menu_ui_context_actions.actions_mut() = select_ca.into_iter().chain(deselect_ca).collect();
    }
}
