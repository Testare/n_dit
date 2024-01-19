use bevy::prelude::HierarchyQueryExt;
pub use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use game_core::prelude::*;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::layout::{CalculatedSizeTty, GlobalTranslationTty};
use crate::render::RenderOrder;
use crate::TerminalWindow;

#[derive(Clone, Copy, Debug, Deref, DerefMut, Event)]
pub struct CrosstermEvent(pub crossterm::event::Event);

#[derive(Clone, Copy, Debug, Deref, DerefMut, Event)]
pub struct MouseEvent(pub crossterm::event::MouseEvent);

#[derive(Clone, Copy, Debug, Deserialize, Event, PartialEq, Reflect, Serialize)]
#[reflect_value(Deserialize, Serialize)]
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

/// Indicates an entity that interacts with mouse events
/// TODO might want to subdivide different mouse event type listeners (Movement, click, etc.)
#[derive(Component, Debug)]
pub struct MouseEventListener;

#[derive(Component, Debug)]
pub struct MouseEventTtyDisabled;

#[derive(Component, CopyGetters, Debug, Event, Getters)]
pub struct MouseEventTty {
    #[getset(get_copy = "pub")]
    entity: Entity,
    #[getset(get_copy = "pub")]
    relative_pos: UVec2,
    #[getset(get_copy = "pub")]
    absolute_pos: UVec2,
    #[getset(get = "pub")]
    modifiers: KeyModifiers,
    #[getset(get = "pub")]
    event_kind: MouseEventTtyKind,
    #[getset(get_copy = "pub")]
    double_click: bool,
    /// The entity that is rendered highest at current mouse position
    /// Not set for exit events
    #[getset(get_copy = "pub")]
    top_entity: Option<Entity>,
    #[getset(get_copy = "pub")]
    is_top_entity_or_ancestor: bool,
}

impl MouseEventTty {
    pub fn is_top_entity(&self) -> bool {
        self.top_entity == Some(self.entity)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MouseEventTtyKind {
    Down(MouseButton),
    Up(MouseButton),
    DoubleClick, // Only applies to left mosue button
    Drag {
        // NOT IMPLEMENTED YET
        button: MouseButton,
        from: UVec2,
        origin: UVec2,
        dragged_entity: Option<Entity>,
    },
    Exit,
    Moved,
    ScrollUp,
    ScrollDown,
    Todo, // Placeholder
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Resource, Debug, Default, Deref)]
pub struct MouseLastPositionTty(UVec2);

pub fn sys_mouse_tty(
    mut evr_crossterm_mouse: EventReader<MouseEvent>,
    res_terminal_window: Res<TerminalWindow>,
    children: Query<&Children>,
    parent_q: Query<&Parent>,
    layout_elements: Query<
        (
            Entity,
            &CalculatedSizeTty,
            &GlobalTranslationTty,
            AsDerefCopied<RenderOrder>,
        ),
        (With<MouseEventListener>, Without<MouseEventTtyDisabled>),
    >,
    mut evw_mouse_tty: EventWriter<MouseEventTty>,
    mut last_click: Local<Option<(std::time::Instant, MouseEvent)>>,
    mut last_position: ResMut<MouseLastPositionTty>,
) {
    for event @ MouseEvent(crossterm::event::MouseEvent {
        kind,
        column,
        row,
        modifiers,
    }) in evr_crossterm_mouse.read()
    {
        let (event_x, event_y) = (*column as u32, *row as u32);
        let absolute_pos = UVec2 {
            x: event_x,
            y: event_y,
        };
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
        } else {
            use crossterm::event::MouseEventKind as MEK;
            match kind {
                MEK::Moved => MouseEventTtyKind::Moved,
                MEK::Down(mb) => MouseEventTtyKind::Down(mb.into()),
                MEK::Up(mb) => MouseEventTtyKind::Up(mb.into()),
                MEK::ScrollDown => MouseEventTtyKind::ScrollDown,
                MEK::ScrollUp => MouseEventTtyKind::ScrollUp,
                MEK::Drag(_mb) => MouseEventTtyKind::Todo, // TODO drag events
            }
        };
        if let Some(render_target) = res_terminal_window.render_target {
            let layout_elements = children
                .iter_descendants(render_target)
                .filter_map(|e| layout_elements.get(e).ok());

            let mut highest_order: u32 = 0;
            let mut top_entity: Option<Entity> = None;
            let mut exit_events = Vec::<Entity>::default();
            let event_entities: Vec<_> = layout_elements
                .filter_map(|(entity, size, translation, render_order)| {
                    if translation.x <= event_x
                        && event_x < (translation.x + size.width32())
                        && translation.y <= event_y
                        && event_y < (translation.y + size.height32())
                    {
                        if render_order > highest_order {
                            highest_order = render_order;
                            top_entity = Some(entity);
                        }
                        let relative_pos = UVec2 {
                            x: event_x - translation.x,
                            y: event_y - translation.y,
                        };
                        Some((entity, relative_pos, render_order))
                    } else if translation.x <= last_position.x
                        && last_position.x < (translation.x + size.width32())
                        && translation.y <= last_position.y
                        && last_position.y < (translation.y + size.height32())
                    {
                        evw_mouse_tty.send(MouseEventTty {
                            entity,
                            absolute_pos,
                            relative_pos: default(), // In this case, we don't really have a helpful value for relative pos
                            modifiers: *modifiers,
                            event_kind: MouseEventTtyKind::Exit,
                            double_click,
                            top_entity: None, // This one is a little more dubious. This still might be helpful information
                            is_top_entity_or_ancestor: false,
                        });
                        None
                    } else {
                        None
                    }
                })
                .collect();
            let ancestors: Vec<Entity> = top_entity
                .map(|top_entity| parent_q.iter_ancestors(top_entity).collect())
                .unwrap_or_default();
            // TODO store top_entity and ancestors in some sort of resource?
            for (entity, relative_pos, render_order) in event_entities {
                evw_mouse_tty.send(MouseEventTty {
                    entity,
                    relative_pos,
                    absolute_pos,
                    modifiers: *modifiers,
                    event_kind,
                    double_click,
                    top_entity,
                    is_top_entity_or_ancestor: top_entity == Some(entity)
                        || ancestors.contains(&entity),
                });
            }
        }

        match *kind {
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                last_click.replace((std::time::Instant::now(), *event));
            },
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                last_position.0 = UVec2 {
                    x: event_x,
                    y: event_y,
                };
            },
            _ => {},
        }
    }
}

impl From<&crossterm::event::MouseButton> for MouseButton {
    fn from(value: &crossterm::event::MouseButton) -> Self {
        use crossterm::event::MouseButton as MB;
        match value {
            MB::Left => Self::Left,
            MB::Right => Self::Right,
            MB::Middle => Self::Middle,
        }
    }
}
