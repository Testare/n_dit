use std::sync::Arc;

use bevy::ecs::system::Command;

use crate::input_event::{MouseButton, MouseEventTty, MouseEventTtyKind};
use crate::prelude::*;

#[derive(Debug)]
pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sys_context_actions);
    }
}

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct ContextMenuSettings {
    /// If there are only two actions, changes right click to perform that action
    adaptive_right_click: bool,
    /// If there is only one action, whether we should display context menu
    single_action_context_menu: bool,
}

impl ContextMenuSettings {
    fn determine_action(
        &self,
        mb: MouseButton,
        context_menu_size: usize,
    ) -> Option<MouseButtonAction> {
        match mb {
            MouseButton::Left => Some(MouseButtonAction::PerformContextAction(0)),
            MouseButton::Middle => None, //(context_menu_size > 1).then_some(MouseButtonAction::CycleContextAction),
            MouseButton::Right => {
                if self.single_action_context_menu && context_menu_size == 1 {
                    None
                } else if self.adaptive_right_click && context_menu_size == 2 {
                    Some(MouseButtonAction::PerformContextAction(1))
                } else {
                    Some(MouseButtonAction::DisplayContextMenu)
                }
            },
        }
    }
}

#[derive(Component, Debug)]
pub struct ContextActions {
    settings_source: Entity,
    actions: Vec<Entity>,
}

impl ContextActions {
    pub fn new(source: Entity, actions: Vec<Entity>) -> Self {
        Self {
            settings_source: source,
            actions,
        }
    }
}

#[derive(Component)]
pub struct ContextAction {
    action_name: String,
    action_op: Arc<dyn Fn(Entity, &mut World) + Send + Sync>,
}

impl std::fmt::Debug for ContextAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContextAction(\"{}\")", self.action_name)
    }
}
impl ContextAction {
    pub fn new<F: Fn(Entity, &mut World) + Send + Sync + 'static>(
        action_name: String,
        action_op: F,
    ) -> Self {
        let action_op = Arc::new(action_op);

        ContextAction {
            action_name,
            action_op,
        }
    }
    pub fn from_command_default<C: Command + Default>(action_name: String) -> Self {
        let action_op = Arc::new(|_, world: &'_ mut World| {
            C::default().apply(world);
        });

        ContextAction {
            action_name,
            action_op,
        }
    }
    pub fn from_command_clone<C: Command + Sync + Clone>(action_name: String, command: C) -> Self {
        let action_op = Arc::new(move |_, world: &'_ mut World| {
            command.clone().apply(world);
        });

        ContextAction {
            action_name,
            action_op,
        }
    }

    pub fn from_command_generator<C: Command, F: Sync + Send + 'static + Fn() -> C>(
        action_name: String,
        command_gen: F,
    ) -> Self {
        let action_op = Arc::new(move |_, world: &'_ mut World| {
            command_gen().apply(world);
        });

        ContextAction {
            action_name,
            action_op,
        }
    }
}

// Determines what we do in response to a mouse button click
#[derive(Clone, Copy, Debug)]
pub enum MouseButtonAction {
    PerformContextAction(usize),
    DisplayContextMenu,
    CycleContextAction,
}

// Perform ContextMenu related actions on mouse click
pub fn sys_context_actions(
    mut evr_mouse: EventReader<MouseEventTty>,
    mut commands: Commands,
    context_actions: Query<&ContextActions>,
    source_settings: Query<CopiedOrDefault<ContextMenuSettings>>,
    context_action: Query<&ContextAction>,
) {
    for mouse_event in evr_mouse.read() {
        let id = mouse_event.entity();
        context_actions.get(id).ok().and_then(|context_actions| {
            let mb = match mouse_event.event_kind() {
                // Should we swap this with up to enable draggable things too?
                MouseEventTtyKind::Down(mousebutton) => *mousebutton,
                _ => return None,
            };
            let settings = source_settings
                .get(context_actions.settings_source)
                .expect("should default");
            let action = match settings.determine_action(mb, context_actions.actions.len())? {
                MouseButtonAction::PerformContextAction(n) => {
                    let context_action =
                        context_action.get(*context_actions.actions.get(n)?).ok()?;
                    log::trace!("Performing context action: {}", context_action.action_name);
                    context_action.action_op.clone()
                },
                MouseButtonAction::CycleContextAction => Arc::new(|_, _: &'_ mut World| {
                    log::error!("TODO Cycling context actions not implemented yet");
                }),
                MouseButtonAction::DisplayContextMenu => Arc::new(|_, _: &'_ mut World| {
                    log::error!("TODO Displaying context menu not implemented yet");
                }),
            };
            commands.add(move |w: &'_ mut World| action(id, w));
            Some(())
        });
    }
}
