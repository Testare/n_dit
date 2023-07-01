use bevy::ecs::system::SystemParam;
use game_core::card::{Action, Actions};
use game_core::node::NodePiece;
use game_core::player::{ForPlayer, Player};

use super::{NodePieceQItem, SimpleSubmenu};
use crate::term::input_event::{MouseButton, MouseEventKind};
use crate::term::layout::{CalculatedSizeTty, LayoutEvent, LayoutMouseTarget, UiFocusOnClick};
use crate::term::node_ui::{SelectedAction, SelectedEntity};
use crate::term::prelude::*;

#[derive(Component, Default, Debug)]
pub struct MenuUiActions;

impl MenuUiActions {
    pub fn handle_layout_events(
        mut layout_events: EventReader<LayoutEvent>,
        actions_of_piece: Query<&Actions, With<NodePiece>>,
        mut players: Query<(&mut SelectedAction, &SelectedEntity), With<Player>>,
        ui_actions: Query<&ForPlayer, With<MenuUiActions>>,
    ) {
        for layout_event in layout_events.iter() {
            if let Ok(ForPlayer(player)) = ui_actions.get(layout_event.entity()) {
                if let Ok((mut selected_action, selected_entity)) = players.get_mut(*player) {
                    let actions = selected_entity.of(&actions_of_piece);

                    // TODO If curio is active and that action has no range, do it immediately. Perhaps if the button is "right", just show it
                    match layout_event.event_kind() {
                        MouseEventKind::Down(MouseButton::Left) => {
                            if actions.is_some()
                                && layout_event.pos().y > 0
                                && layout_event.pos().y <= actions.unwrap().len() as u32
                            {
                                let clicked_action = Some((layout_event.pos().y - 1) as usize);
                                if **selected_action != clicked_action {
                                    **selected_action = Some((layout_event.pos().y - 1) as usize);
                                } else {
                                    **selected_action = None;
                                }
                            }
                        },
                        _ => {},
                    }
                } else {
                    log::error!("Entity missing required components to render player UI");
                }
            }
        }
    }
}

#[derive(SystemParam)]
pub struct MenuUiActionsParam<'w, 's> {
    players: Query<'w, 's, &'static SelectedAction, With<Player>>,
    actions: Query<'w, 's, &'static Action>,
}

impl SimpleSubmenu for MenuUiActions {
    const NAME: &'static str = "Actions Menu";
    type UiBundleExtras = (LayoutMouseTarget, UiFocusOnClick);

    type RenderSystemParam = MenuUiActionsParam<'static, 'static>;

    fn layout_event_system() -> Option<bevy::app::SystemAppConfig> {
        Some(Self::handle_layout_events.into_app_config())
    }

    fn height(selected: &NodePieceQItem<'_>) -> Option<usize> {
        let actions = selected.actions.as_deref()?;
        Some(actions.len() + 1)
    }

    fn render(
        player: Entity,
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        param: &MenuUiActionsParam,
    ) -> Option<Vec<String>> {
        let actions = selected.actions.as_deref()?;
        let mut menu = vec![format!("{0:-<1$}", "-Actions", size.width())];
        for action in actions.iter() {
            if let Some(action) = get_assert!(*action, param.actions) {
                menu.push(action.name.clone());
            }
        }
        if let Some(action_idx) = **(param.players.get(player).ok()?) {
            menu[action_idx + 1] = format!("▶{}", menu[action_idx + 1]);
        }
        Some(menu)
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        (LayoutMouseTarget, UiFocusOnClick)
    }
}
