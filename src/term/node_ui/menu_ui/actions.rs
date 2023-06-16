use crossterm::event::{MouseButton, MouseEventKind};
use game_core::card::Actions;
use game_core::node::NodePiece;
use game_core::player::{ForPlayer, Player};

use super::{NodePieceQItem, SimpleSubmenu};
use crate::term::layout::{CalculatedSizeTty, LayoutEvent};
use crate::term::node_ui::{SelectedAction, SelectedEntity};
use crate::term::prelude::*;

#[derive(Component, Debug)]
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

impl SimpleSubmenu for MenuUiActions {
    type RenderSystemParam = Query<'static, 'static, &'static SelectedAction, With<Player>>;

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
        selected_action: &Query<&SelectedAction, With<Player>>,
    ) -> Option<Vec<String>> {
        let actions = selected.actions.as_deref()?;
        let mut menu = vec![format!("{0:-<1$}", "-Actions", size.width())];
        for action in actions.iter() {
            menu.push(action.name.clone());
        }
        if let Some(action_idx) = **(selected_action.get(player).ok()?) {
            menu[action_idx + 1] = format!("▶{}", menu[action_idx + 1]);
        }
        Some(menu)
    }
}
