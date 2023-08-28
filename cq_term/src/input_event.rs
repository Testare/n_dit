pub use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use game_core::prelude::*;
use getset::{CopyGetters, Getters};

use crate::layout::{
    CalculatedSizeTty, GlobalTranslationTty, LayoutMouseTarget, LayoutMouseTargetDisabled,
};

#[derive(Clone, Copy, Deref, DerefMut, Event)]
pub struct CrosstermEvent(pub crossterm::event::Event);

#[derive(Clone, Copy, Deref, DerefMut, Event)]
pub struct MouseEvent(pub crossterm::event::MouseEvent);

#[derive(Clone, Copy, Event, PartialEq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    // pub kind: KeyEventKind,
    // pub state: KeyEventState,
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(
        crossterm::event::KeyEvent {
            code, modifiers, ..
        }: crossterm::event::KeyEvent,
    ) -> Self {
        Self {
            code,
            modifiers,
            // kind,
            // state,
        }
    }
}

#[derive(Component, CopyGetters, Debug, Event, Getters)]
pub struct MouseEventTty {
    #[getset(get_copy = "pub")]
    entity: Entity,
    #[getset(get_copy = "pub")]
    pos: UVec2,
    #[getset(get = "pub")]
    modifiers: KeyModifiers,
    #[getset(get = "pub")]
    event_kind: MouseEventTtyKind,
    #[getset(get_copy = "pub")]
    double_click: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum MouseEventTtyKind {
    Down(MouseButtonTty),
    Up(MouseButtonTty),
    DoubleClick, // Only applies to left mosue button
    Drag {
        button: MouseButtonTty,
        from: UVec2,
        origin: UVec2,
        dragged_entity: Option<Entity>,
        // Should there be an event for releasing item?
    },
    Exit,
    Moved,
    ScrollUp,
    ScrollDown,
    Todo,
    // Move,
}

#[derive(Clone, Copy, Debug)]
pub enum MouseButtonTty {
    LeftButton,
    RightButton,
    MiddleButton,
}

pub fn sys_mouse_tty(
    mut evr_crossterm_mouse: EventReader<MouseEvent>,
    layout_elements: Query<
        (Entity, &CalculatedSizeTty, &GlobalTranslationTty, DebugName),
        (With<LayoutMouseTarget>, Without<LayoutMouseTargetDisabled>),
    >,
    mut evw_mouse_tty: EventWriter<MouseEventTty>,
    mut last_click: Local<Option<(std::time::Instant, MouseEvent)>>,
    mut last_position: Local<Option<UVec2>>,
) {
    for event @ MouseEvent(crossterm::event::MouseEvent {
        kind,
        column,
        row,
        modifiers,
    }) in evr_crossterm_mouse.iter()
    {
        let (event_x, event_y) = (*column as u32, *row as u32);
        let double_click = last_click
            .map(|(last_event_time, last_event)| {
                last_event_time.elapsed().as_millis() <= 500
                    && last_event.kind == *kind
                    && last_event.column == *column
                    && last_event.row == *row
            })
            .unwrap_or_default();

        let event_kind = if double_click {
            MouseEventTtyKind::DoubleClick
        } else if matches!(kind, MouseEventKind::Moved) {
            MouseEventTtyKind::Moved
        } else {
            MouseEventTtyKind::Todo
        };

        for (entity, size, translation, debug_name) in layout_elements.iter() {
            if translation.x <= event_x
                && event_x < (translation.x + size.width32())
                && translation.y <= event_y
                && event_y < (translation.y + size.height32())
            {
                let pos = UVec2 {
                    x: event_x - translation.x,
                    y: event_y - translation.y,
                };
                evw_mouse_tty.send(MouseEventTty {
                    entity,
                    pos,
                    modifiers: modifiers.clone(),
                    event_kind,
                    double_click,
                })
            } else if last_position
                .as_ref()
                .map(
                    |UVec2 {
                         x: last_x,
                         y: last_y,
                     }| {
                        translation.x <= *last_x
                            && *last_x < (translation.x + size.width32())
                            && translation.y <= *last_y
                            && *last_y < (translation.y + size.height32())
                    },
                )
                .unwrap_or(false)
            {
                evw_mouse_tty.send(MouseEventTty {
                    entity,
                    pos: default(), // In this case, we don't really have a helpful value for pos
                    modifiers: modifiers.clone(),
                    event_kind: MouseEventTtyKind::Exit,
                    double_click,
                });
            }
        }
        match *kind {
            MouseEventKind::Down(_) => {
                last_click.replace((std::time::Instant::now(), *event));
            },
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                last_position.replace(UVec2 {
                    x: event_x,
                    y: event_y,
                });
            },
            _ => {},
        }
    }
}
