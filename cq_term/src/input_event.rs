use std::collections::VecDeque;

use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::HierarchyQueryExt;
pub use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use game_core::prelude::*;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::layout::{CalculatedSizeTty, GlobalTranslationTty, LayoutUpdatedEvent};
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

// Note: This will cause pass-through of mouse events
#[derive(Component, Debug)]
pub struct MouseEventTtyDisabled;

#[derive(Clone, Copy, CopyGetters, Debug, Event, Getters)]
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
    // TODO store frame number here?
}

impl MouseEventTty {
    pub fn is_top_entity(&self) -> bool {
        self.top_entity == Some(self.entity)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MouseEventTtyKind {
    Down(MouseButton),
    Up(MouseButton), // TODO make Drag data a struct and include it Option<> here
    DoubleClick,     // Only applies to left mouse button
    Drag {
        // NOT IMPLEMENTED YET
        button: MouseButton,
        from: UVec2,
        dragged_entity: Option<Entity>,
    },
    Exit,
    Moved, // NOTE: Also triggers if mouse doesn't move but layout does
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

#[derive(Resource, Debug, Default, Deref, PartialEq)]
pub struct MouseLastPositionTty(UVec2);

pub fn sys_mouse_tty(
    mut evr_crossterm_mouse: EventReader<MouseEvent>,
    mut evr_layout_update: EventReader<LayoutUpdatedEvent>,
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
    mut drag_data: Local<Option<MouseEventTtyKind>>,
    mut entered_entities: Local<EntityHashSet>,
) {
    let render_target = match res_terminal_window.render_target {
        Some(render_target) => render_target,
        None => return,
    };
    let mut mouse_event_queue: VecDeque<(MouseEventTtyKind, UVec2, KeyModifiers, bool)> =
        VecDeque::default();

    for event @ MouseEvent(crossterm::event::MouseEvent {
        kind,
        column,
        row,
        modifiers,
    }) in evr_crossterm_mouse.read()
    {
        use crossterm::event::MouseEventKind as MEK;

        let absolute_pos = UVec2 {
            x: *column as u32,
            y: *row as u32,
        };
        let double_click = last_click
            .map(|(last_event_time, last_event)| {
                last_event_time.elapsed().as_millis() <= 500
                    && last_event.kind == *kind
                    && last_event.column == *column
                    && last_event.row == *row
            })
            .unwrap_or_default();

        if matches!(kind, MEK::Down(crossterm::event::MouseButton::Left)) {
            last_click.replace((std::time::Instant::now(), *event));
        };

        let event_kind = if double_click {
            MouseEventTtyKind::DoubleClick
        } else {
            match kind {
                MEK::Moved => MouseEventTtyKind::Moved,
                MEK::Down(mb) => MouseEventTtyKind::Down(mb.into()),
                MEK::Up(mb) => MouseEventTtyKind::Up(mb.into()),
                MEK::ScrollDown => MouseEventTtyKind::ScrollDown,
                MEK::ScrollUp => MouseEventTtyKind::ScrollUp,
                MEK::Drag(_mb) => drag_data.unwrap_or(MouseEventTtyKind::Moved), // TODO drag events
            }
        };

        mouse_event_queue.push_back((event_kind, absolute_pos, *modifiers, double_click));
    }

    if !evr_layout_update.is_empty() && mouse_event_queue.is_empty() {
        // Layout updated, but with no mouse event we must manually check new enter/exits
        mouse_event_queue.push_back((
            MouseEventTtyKind::Moved,
            **last_position,
            KeyModifiers::NONE,
            false,
        ));
        evr_layout_update.clear();
    }

    // Run through events, mouse or layout generated
    for (
        event_kind,
        absolute_pos @ UVec2 {
            x: event_x,
            y: event_y,
        },
        modifiers,
        double_click,
    ) in mouse_event_queue.into_iter()
    {
        let layout_elements = children
            .iter_descendants(render_target)
            .filter_map(|e| layout_elements.get(e).ok());

        let mut highest_order: u32 = 0;
        let mut top_entity: Option<Entity> = None;
        let event_entities: Vec<_> = layout_elements
            .filter_map(|(entity, size, translation, render_order)| {
                if translation.x <= event_x
                    && event_x < (translation.x + size.width32())
                    && translation.y <= event_y
                    && event_y < (translation.y + size.height32())
                {
                    entered_entities.insert(entity);
                    if render_order > highest_order {
                        highest_order = render_order;
                        top_entity = Some(entity);
                    }
                    let relative_pos = UVec2 {
                        x: event_x - translation.x,
                        y: event_y - translation.y,
                    };
                    Some((entity, relative_pos))
                } else if entered_entities.contains(&entity) {
                    entered_entities.remove(&entity);
                    evw_mouse_tty.send(MouseEventTty {
                        entity,
                        absolute_pos,
                        relative_pos: default(), // In this case, we don't really have a helpful value for relative pos
                        modifiers,
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
        for (entity, relative_pos) in event_entities {
            evw_mouse_tty.send(MouseEventTty {
                entity,
                relative_pos,
                absolute_pos,
                modifiers,
                event_kind,
                double_click,
                top_entity,
                is_top_entity_or_ancestor: top_entity == Some(entity)
                    || ancestors.contains(&entity),
            });
        }

        last_position.set_if_neq(MouseLastPositionTty(absolute_pos));
        match event_kind {
            MouseEventTtyKind::Down(button) => {
                *drag_data = Some(MouseEventTtyKind::Drag {
                    from: absolute_pos,
                    button,
                    dragged_entity: top_entity,
                });
            },
            MouseEventTtyKind::Drag { .. } => {},
            _ => {
                *drag_data = None;
            },
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
