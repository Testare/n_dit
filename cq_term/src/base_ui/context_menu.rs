use std::sync::Arc;
use std::time::Duration;

use bevy::ecs::query::QueryEntityError;
use bevy::ecs::system::{Command, SystemId};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::time::{Time, Timer, TimerMode};
use charmi::CharacterMapImage;
use crossterm::style::{ContentStyle, Stylize};

use super::HoverPoint;
use crate::input_event::{MouseButton, MouseEventListener, MouseEventTty, MouseEventTtyKind};
use crate::layout::{CalculatedSizeTty, StyleTty, VisibilityTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering};

/// The amount of time the context menu will stay open if the mouse moves off of it
/// This is the default value, but it can be changed through configuration
const CONTEXT_MENU_OPEN_DURATION_DEFAULT: Duration = Duration::from_millis(1000);

#[derive(Debug)]
pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sys_context_actions)
            .add_systems(
                Update,
                sys_context_menu_fade.in_set(RenderTtySet::PreCalculateLayout),
            )
            .init_resource::<SystemIdDisplayContextMenu>();
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
    settings_source: Entity, // Potentially make separate component?
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

/// The component for the UI that displays the context actions
#[derive(Component, Debug, Default)]
pub struct ContextMenu {
    /// The entity whose context actions ought to be displayed
    position: UVec2,
    actions_context: Option<Entity>,
}

#[derive(Component, Debug, Deref, DerefMut)]
pub struct ContextMenuTimer(Timer);

impl Default for ContextMenuTimer {
    fn default() -> Self {
        Self(Timer::new(
            CONTEXT_MENU_OPEN_DURATION_DEFAULT,
            TimerMode::Once,
        ))
    }
}

/// To indicate a context menu pane, where the context menu will be rendered.
#[derive(Component, Debug)]
pub struct ContextMenuPane;

impl ContextMenuPane {
    pub fn spawn(commands: &mut Commands) -> Entity {
        use taffy::prelude::*;
        commands
            .spawn((
                Name::new("Context Menu pane"),
                ContextMenuPane,
                StyleTty(Style {
                    grid_row: line(1),
                    grid_column: line(1),
                    display: Display::Grid,
                    grid_template_rows: vec![points(4.0), points(2.0)],
                    grid_template_columns: vec![points(10.0), points(22.0)],
                    ..default()
                }),
            ))
            .with_children(|content_menu_pane| {
                content_menu_pane.spawn((
                    Name::new("Context Menu"),
                    StyleTty(Style {
                        display: Display::Grid,
                        grid_row: line(2),
                        grid_column: line(2),
                        grid_template_rows: vec![repeat(
                            GridTrackRepetition::AutoFill,
                            vec![points(1.0)],
                        )],
                        grid_template_columns: vec![points(1.), fr(1.), points(1.)],
                        ..default()
                    }),
                    TerminalRendering::new(vec![
                        "[You should not be]".to_string(),
                        "[reading this.    ]".to_string(),
                    ]),
                    VisibilityTty(false),
                    ContextMenu::default(),
                    ContextMenuTimer::default(),
                    HoverPoint::default(),
                    MouseEventListener,
                ));
            })
            .id()
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

#[derive(Debug, Resource)]
pub struct SystemIdDisplayContextMenu(SystemId);

impl FromWorld for SystemIdDisplayContextMenu {
    fn from_world(world: &mut World) -> Self {
        Self(world.register_system(sys_display_context_menu))
    }
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

            let mouse_pos = mouse_event.absolute_pos();
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
                MouseButtonAction::DisplayContextMenu => {
                    Arc::new(move |id, world: &'_ mut World| {
                        for mut context_menu in world.query::<&mut ContextMenu>().iter_mut(world) {
                            // TODO what if there are multiple???
                            context_menu.actions_context = Some(id);
                            context_menu.position = mouse_pos;
                        }
                        let display_system = world.resource::<SystemIdDisplayContextMenu>().0;
                        world.run_system(display_system).unwrap();
                    })
                },
            };
            commands.add(move |w: &'_ mut World| action(id, w));
            Some(())
        });
    }
}

fn sys_display_context_menu(
    mut commands: Commands,
    mut context_menu_q: Query<(
        Entity,
        &ContextMenu,
        AsDerefCopied<Parent>,
        AsDerefMut<VisibilityTty>,
        &mut TerminalRendering,
    )>,
    mut context_menu_pane: Query<
        (AsDerefMut<StyleTty>, AsDerefCopied<CalculatedSizeTty>),
        With<ContextMenuPane>,
    >,
    context_actions_q: Query<&ContextActions>,
    context_action_q: Query<&ContextAction>,
) {
    for (cm_id, context_menu, parent_id, mut is_visible, mut rendering) in context_menu_q.iter_mut()
    {
        if context_menu.actions_context.is_none() {
            is_visible.set_if_neq(false);
            continue;
        }
        let context_actions_id = context_menu
            .actions_context
            .expect("should have been checked before this step");
        let context_menu_pane = context_menu_pane.get_mut(parent_id);
        if context_menu_pane.is_err() {
            log::error!("Entity [{parent_id:?} is parent of ContextMenu, but does not have required components.");
            continue;
        }
        let (mut pane_style, pane_size) =
            context_menu_pane.expect("Should have been checked previously");

        let context_menu_actions: Vec<&String> = context_actions_q.get(context_actions_id).map(|context_actions|
            context_actions.actions.iter().copied().filter_map(|context_action_id| {
                match context_action_q.get(context_action_id) {
                    Ok(context_action) => Some(&context_action.action_name),
                    Err(e) => {
                        match e {
                            QueryEntityError::NoSuchEntity(id) => {
                                log::error!("Couldn't find context action, no entity: [Entity: {id:?}]");
                            },
                            QueryEntityError::QueryDoesNotMatch(id) => {
                                log::error!("Couldn't find context action, not a match: [Entity: {id:?}], logging components");
                                commands.entity(context_action_id).log_components();
                            },
                            QueryEntityError::AliasedMutability(_) => unreachable!("Should not be a possible result")
                        }
                        None
                    }
                }
            }).collect()
        ).unwrap_or_default();

        if context_menu_actions.is_empty() {
            is_visible.set_if_neq(false);
            continue;
        }
        let target_height = context_menu_actions.len() as u32 + 2;
        let target_width = context_menu_actions
            .iter()
            .map(|ca_name| ca_name.len())
            .max()
            .expect("should not be empty") as u32
            + 2;
        // target_pos Needs testing
        let target_pos_x = if pane_size.x >= context_menu.position.x + target_width {
            context_menu.position.x
        } else {
            context_menu.position.x.saturating_sub(target_width - 1)
        };

        let target_pos_y = if pane_size.y >= context_menu.position.y + target_height {
            context_menu.position.y
        } else {
            context_menu.position.y.saturating_sub(target_height - 1)
        };

        // TODO Perhaps the rendering should be a different system
        let mut charmi = CharacterMapImage::new();
        // TODO customizable context menu styles
        let border_style = ContentStyle::new().cyan();
        let content_style = ContentStyle::new().yellow();
        charmi
            .new_row()
            .add_text("-".repeat(target_width as usize), &border_style);
        /*if let Some(children) = children {
            for child in children.iter().copied() {
                // Using despawn_recursive so that it will be removed from the parent's Children component
                commands.entity(child).despawn_recursive();
            }
        }*/
        commands
            .entity(cm_id)
            .despawn_descendants()
            .with_children(|cm_commands| {
                use taffy::prelude::*;

                for (ca_name, row) in context_menu_actions.into_iter().zip(2..) {
                    charmi
                        .new_row()
                        .add_char('/', &border_style)
                        // Not adding a gap - don't want this see-through
                        // If I ever add merging effects to charmi, might do some slightly opaque
                        .add_text(" ".repeat((target_width - 2) as usize), &content_style)
                        // .add_text(ca_name.as_str(), &content_style)
                        .add_char('/', &border_style);
                    cm_commands.spawn((
                        StyleTty(Style {
                            grid_row: line(row),
                            grid_column: line(2),
                            ..default()
                        }),
                        Name::new(format!("Context Menu Item [{}]", ca_name)),
                        TerminalRendering::new(vec![ca_name.to_string()]),
                    ));
                }
            });
        charmi
            .new_row()
            .add_text("-".repeat(target_width as usize), &border_style);
        rendering.update_charmie(charmi);

        use taffy::prelude::points;
        pane_style.grid_template_rows =
            vec![points(target_pos_y as f32), points(target_height as f32)];
        pane_style.grid_template_columns =
            vec![points(target_pos_x as f32), points(target_width as f32)];
        is_visible.set_if_neq(true);
    }
}

fn sys_context_menu_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            AsDerefMut<ContextMenuTimer>,
            AsDeref<HoverPoint>,
            AsDerefMut<VisibilityTty>,
        ),
        With<ContextMenu>,
    >,
) {
    for (cm_id, mut cm_timer, hover_point, mut is_visible) in query.iter_mut() {
        if *is_visible {
            cm_timer.tick(time.delta());

            if hover_point.is_some() {
                if !cm_timer.paused() {
                    cm_timer.pause();
                    cm_timer.reset();
                }
            } else if cm_timer.paused() {
                cm_timer.unpause()
            } else if cm_timer.finished() {
                *is_visible = false;
                cm_timer.pause();
                cm_timer.reset();
                commands.entity(cm_id).despawn_descendants();
            }
        }
    }
}
