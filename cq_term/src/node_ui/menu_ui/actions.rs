use charmi::{CharacterMapImage, CharmieRow};
use crossterm::event::KeyModifiers;
use crossterm::style::{ContentStyle, Stylize};
use game_core::card::{Action, ActionRange, Actions};
use game_core::node::{IsTapped, NodeOp, NodePiece};
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
use taffy::style::Dimension;

use crate::input_event::{MouseButton, MouseEventKind};
use crate::key_map::NamedInput;
use crate::layout::{
    CalculatedSizeTty, LayoutEvent, LayoutMouseTarget, StyleTty, UiFocus, UiFocusOnClick,
};
use crate::node_ui::{NodeUi, NodeUiQItem, SelectedAction, SelectedEntity};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};
use crate::{KeyMap, Submap};

#[derive(Component, Default, Debug)]
pub struct MenuUiActions;

impl MenuUiActions {
    pub fn kb_action_menu(
        mut players: Query<
            (
                Entity,
                &UiFocus,
                &KeyMap,
                &SelectedEntity,
                &mut SelectedAction,
            ),
            With<Player>,
        >,
        node_pieces: Query<(&Actions, Option<&IsTapped>), With<NodePiece>>,
        mut ev_keys: EventReader<KeyEvent>,
        action_menu_uis: Query<(), With<MenuUiActions>>,
        mut ev_node_op: EventWriter<Op<NodeOp>>,
        rangeless_actions: Query<(), (Without<ActionRange>, With<Action>)>,
    ) {
        for KeyEvent { code, modifiers } in ev_keys.iter() {
            for (player_id, focus, key_map, selected_entity, mut selected_action) in
                players.iter_mut()
            {
                if (**focus)
                    .map(|focused_ui| !action_menu_uis.contains(focused_ui))
                    .unwrap_or(true)
                {
                    continue;
                }

                key_map
                    .named_input_for_key(Submap::Node, *code, *modifiers)
                    .and_then(|named_input| {
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
                                        **selected_action = next_action;
                                    }
                                },
                                NamedInput::MenuFocusNext | NamedInput::MenuFocusPrev => {
                                    **selected_action = None;
                                },
                                NamedInput::Activate => {
                                    if is_tapped.map(|is_tapped| **is_tapped).unwrap_or(true) {
                                        **selected_action = None;
                                    } else if let Some(action) =
                                        actions.get(selected_action.unwrap_or_default())
                                    {
                                        if rangeless_actions.contains(*action) {
                                            ev_node_op.send(Op::new(
                                                player_id,
                                                NodeOp::PerformCurioAction {
                                                    action: *action,
                                                    curio: **selected_entity,
                                                    target: default(),
                                                },
                                            ))
                                        }
                                    }
                                },
                                _ => {},
                            }
                        }
                        Some(())
                    });
            }
        }
    }

    pub fn mouse_action_menu(
        mut ev_node_op: EventWriter<Op<NodeOp>>,
        mut layout_events: EventReader<LayoutEvent>,
        node_pieces: Query<(&Actions, Option<&IsTapped>), With<NodePiece>>,
        mut players: Query<(&mut SelectedAction, &SelectedEntity), With<Player>>,
        ui_actions: Query<&ForPlayer, With<MenuUiActions>>,
        rangeless_actions: Query<(), (Without<ActionRange>, With<Action>)>,
    ) {
        for layout_event in layout_events.iter() {
            if let Ok(ForPlayer(player_id)) = ui_actions.get(layout_event.entity()) {
                get_assert_mut!(*player_id, players, |(
                    mut selected_action,
                    selected_entity,
                )| {
                    if let Some((actions, is_tapped)) = selected_entity.of(&node_pieces) {
                        // TODO If curio is active and that action has no range, do it immediately. Perhaps if the button is "right", just show it
                        match layout_event.event_kind() {
                            MouseEventKind::Down(MouseButton::Left) => {
                                if layout_event.pos().y > 0
                                    && layout_event.pos().y <= actions.len() as u32
                                {
                                    let clicked_action_idx = (layout_event.pos().y - 1) as usize;
                                    let clicked_action = Some(clicked_action_idx);

                                    if **selected_action != clicked_action {
                                        **selected_action =
                                            Some((layout_event.pos().y - 1) as usize);
                                    } else if layout_event.double_click()
                                        || !layout_event
                                            .modifiers()
                                            .intersection(KeyModifiers::SHIFT | KeyModifiers::ALT)
                                            .is_empty()
                                    {
                                        let action_id = actions[clicked_action_idx];
                                        if matches!(is_tapped, Some(IsTapped(false)))
                                            && rangeless_actions.contains(action_id)
                                        {
                                            **selected_action = None;
                                            ev_node_op.send(Op::new(
                                                *player_id,
                                                NodeOp::PerformCurioAction {
                                                    action: action_id,
                                                    curio: **selected_entity,
                                                    target: default(),
                                                },
                                            ))
                                        }
                                    } else {
                                        **selected_action = None;
                                    }
                                }
                            },
                            _ => {},
                        }
                    }
                    Some(())
                });
            }
        }
    }

    pub fn sys_on_focus_action_menu(
        mut players: Query<(&UiFocus, &mut SelectedAction), (Changed<UiFocus>, With<Player>)>,
        action_menus: Query<(Entity, &ForPlayer), With<MenuUiActions>>,
    ) {
        for (action_menu, ForPlayer(player)) in action_menus.iter() {
            if let Ok((ui_focus, mut selected_action)) = players.get_mut(*player) {
                if **ui_focus == Some(action_menu) && selected_action.is_none() {
                    **selected_action = Some(0);
                }
            }
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
        node_pieces: Query<&Actions, With<NodePiece>>,
        players: Query<(&SelectedEntity, &SelectedAction, &UiFocus), With<Player>>,
        mut ui: Query<
            (
                Entity,
                &CalculatedSizeTty,
                &ForPlayer,
                &mut TerminalRendering,
            ),
            With<MenuUiActions>,
        >,
        actions: Query<&Action>,
    ) {
        // let render_param = render_param.into_inner();
        for (id, size, ForPlayer(player), mut tr) in ui.iter_mut() {
            if let Ok((selected_entity, selected_action, focus)) = players.get(*player) {
                let rendering = selected_entity
                    .of(&node_pieces)
                    .and_then(|piece_actions| {
                        let title_style = if Some(id) == **focus {
                            // TODO replace with configurable "MenuUiTitleFocused"
                            ContentStyle::new().reverse()
                        } else {
                            // TODO replace with configurable "MenuUiTitleUnfocused"
                            ContentStyle::new()
                        };
                        let mut menu = CharacterMapImage::new();
                        let menu_title = format!("{0:-<1$}", "-Actions", size.width());
                        menu.push_row(CharmieRow::of_text(menu_title, &title_style));

                        for (idx, action) in piece_actions.iter().enumerate() {
                            if let Some(action) = get_assert!(*action, actions) {
                                if Some(idx) == **selected_action {
                                    menu.push_row(format!("▶{}", action.name).into());
                                } else {
                                    menu.push_row(action.name.as_str().into());
                                }
                            }
                        }
                        Some(menu)
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
                Self::sys_on_focus_action_menu
                    .before(Self::kb_action_menu)
                    .in_set(NDitCoreSet::ProcessInputs),
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
    type UiBundleExtras = (LayoutMouseTarget, UiFocusOnClick);
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
        (LayoutMouseTarget, UiFocusOnClick)
    }
}
