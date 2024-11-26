use bevy::ecs::query::Has;

use crate::input_event::MouseEventTtyDisabled;
use crate::layout::CalculatedSizeTty;
use crate::prelude::*;
use crate::render::TerminalRendering;

// TODO Make this a pane
#[derive(Component, Debug)]
pub struct PopupMenu;

pub fn sys_render_popup_menu(
    mut popup_menus: Query<
        (AsDerefCopied<CalculatedSizeTty>, &mut TerminalRendering),
        With<PopupMenu>,
    >,
) {
    for (size, mut tr) in popup_menus.iter_mut() {
        // Popup will usually have a padding of at least 1, so if the size is 2x2 then nothing is in it
        let render_vec = if size.y > 2 && size.x > 2 {
            let top_border = "┌".to_owned() + "─".repeat((size.x - 2) as usize).as_str() + "┐";
            let middle = "│".to_owned() + " ".repeat((size.x - 2) as usize).as_str() + "│";
            let bottom_border = "└".to_owned() + "─".repeat((size.x - 2) as usize).as_str() + "┘";
            let mut vec = vec![top_border];
            for _ in 2..size.y {
                vec.push(middle.clone())
            }
            vec.push(bottom_border);
            vec
        } else {
            Vec::default()
        };
        tr.update(render_vec);
    }
}

/// Need to disable mouse events when popup menu is minimized, else it causes
/// a deadzone in the middle of the screen
pub fn sys_mouse_popup_menu(
    mut commands: Commands,
    q_popup_menu: Query<
        (
            Entity,
            AsDerefCopied<CalculatedSizeTty>,
            Has<MouseEventTtyDisabled>,
        ),
        (With<PopupMenu>, Changed<CalculatedSizeTty>),
    >,
) {
    for (popup_id, size, is_disabled) in q_popup_menu.iter() {
        let too_small = size.x <= 2 || size.y <= 2;
        if too_small && !is_disabled {
            commands.entity(popup_id).insert(MouseEventTtyDisabled);
        } else if !too_small && is_disabled {
            commands.entity(popup_id).remove::<MouseEventTtyDisabled>();
        }
    }
}
