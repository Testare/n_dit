use crossterm::event::{MouseEventKind, MouseButton};
use game_core::Node;

use crate::term::{prelude::*, layout::{LayoutEvent, CalculatedSizeTty}, node_ui::{NodeUiDataParam, SelectedAction}};

use super::{NodePieceQItem, SimpleSubmenu, NodePieceQ};

#[derive(Component, Debug)]
pub struct MenuUiActions;

impl MenuUiActions {

    pub fn handle_layout_events(
        mut layout_events: EventReader<LayoutEvent>,
        node_pieces: Query<NodePieceQ>,
        node_data_p: NodeUiDataParam,
        mut selected_action: Query<&mut SelectedAction, With<Node>>,
    ) {
        node_data_p.node_data().and_then(|node_data| {
            let selected = node_pieces.get((**node_data.selected_entity)?).ok()?;
            let actions = selected.actions?;
            let mut selected_action = selected_action.get_mut((**node_data_p.node_focus)?).ok()?;
            for layout_event in layout_events.iter() {
                match layout_event.event_kind() {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if layout_event.pos().y > 0 && layout_event.pos().y <= actions.len() as u32 {
                            let clicked_action = Some((layout_event.pos().y - 1)  as usize);
                            if **selected_action != clicked_action {
                                **selected_action = Some((layout_event.pos().y - 1) as usize);
                            } else {
                                **selected_action = None;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Some(selected)
        });

    }
}

impl SimpleSubmenu for MenuUiActions {
    type RenderSystemParam = Query<'static, 'static, &'static SelectedAction, With<Node>>;

    fn height(selected: &NodePieceQItem<'_>) -> Option<usize> {
        let actions = selected.actions.as_deref()?;
        Some(actions.len() + 1)
    }

    fn render(
        selected: &NodePieceQItem<'_>,
        size: &CalculatedSizeTty,
        selected_action: Query<&SelectedAction, With<Node>>,
    ) -> Option<Vec<String>> {
        let actions = selected.actions.as_deref()?;
        let mut menu = vec![format!("{0:-<1$}", "-Actions", size.width())];
        for action in actions.iter() {
            menu.push(action.name.clone());
        }
        if let Some(action_idx) = **(selected_action.get_single().ok()?) {
            menu[action_idx+1] = format!("▶{}", menu[action_idx+1]);

        }
        Some(menu)
    }
}