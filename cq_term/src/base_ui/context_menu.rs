use std::sync::Arc;
use std::time::Duration;

use bevy::ecs::query::{Has, QueryEntityError};
use bevy::ecs::system::{Command, SystemId};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::time::{Time, Timer, TimerMode};
use charmi::CharacterMapImage;
use getset::CopyGetters;

use super::HoverPoint;
use crate::configuration::DrawConfiguration;
use crate::input_event::{MouseButton, MouseEventListener, MouseEventTty, MouseEventTtyKind};
use crate::layout::{CalculatedSizeTty, StyleTty, VisibilityTty};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering};

/// The amount of time the context menu will stay open if the mouse moves off of it
const CONTEXT_MENU_OPEN_DURATION_DEFAULT: Duration = Duration::from_millis(1000);

#[derive(Debug)]
pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (sys_context_actions, sys_context_menu_item_click),
        )
        .add_systems(
            Update,
            (
                sys_context_menu_fade.in_set(RenderTtySet::PreCalculateLayout),
                sys_render_context_items.in_set(RenderTtySet::RenderLayouts),
            ),
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
        no_default_flag: bool,
    ) -> Option<MouseButtonAction> {
        match mb {
            MouseButton::Left => {
                if no_default_flag {
                    Some(MouseButtonAction::DisplayContextMenu)
                } else {
                    Some(MouseButtonAction::PerformContextAction(0))
                }
            },
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

/// Used to indicate that left-click should not perform the default (first)
/// action but instead open the context menu.
#[derive(Component, Debug)]
pub struct ContextActionsNoDefault;

#[derive(Component, Debug)]
pub struct ContextActions {
    settings_source: Entity, // Potentially make separate component?
    actions: Vec<Entity>,
}

impl ContextActions {
    pub fn new(settings_source: Entity, actions: &[Entity]) -> Self {
        Self {
            settings_source,
            actions: actions.into(),
        }
    }

    // TODO better API for changing context actions at runtime
    pub fn actions_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.actions
    }
}

/// The component for the UI that displays the context actions
#[derive(Component, Debug, Default, CopyGetters)]
pub struct ContextMenu {
    #[getset(get_copy = "pub")]
    position: UVec2,
    #[getset(get_copy = "pub")]
    mouse_event: Option<MouseEventTty>,
    /// The entity whose context actions ought to be displayed
    #[getset(get_copy = "pub")]
    actions_context: Option<Entity>,
}

#[derive(Component, Debug, Deref)]
pub struct ContextMenuItem(String, #[deref] Entity);

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
                    grid_template_rows: vec![length(4.0), length(2.0)],
                    grid_template_columns: vec![length(10.0), length(22.0)],
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
                        grid_auto_rows: vec![length(1.0)],
                        grid_template_columns: vec![length(1.), fr(1.), length(1.)],
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
    action_op: Arc<dyn Fn(Entity, MouseEventTty, &mut World) + Send + Sync>,
}

impl std::fmt::Debug for ContextAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContextAction(\"{}\")", self.action_name)
    }
}
impl ContextAction {
    pub fn new<S: ToString, F: Fn(Entity, &mut World) + Send + Sync + 'static>(
        action_name: S,
        action_op: F,
    ) -> Self {
        ContextAction {
            action_name: action_name.to_string(),
            action_op: Arc::new(move |id, _mouse, world| action_op(id, world)),
        }
    }

    pub fn new_with_mouse_event<
        S: ToString,
        F: Fn(Entity, MouseEventTty, &mut World) + Send + Sync + 'static,
    >(
        action_name: S,
        action_op: F,
    ) -> Self {
        let action_op = Arc::new(action_op);

        ContextAction {
            action_name: action_name.to_string(),
            action_op,
        }
    }
    pub fn from_command_default<C: Command + Default>(action_name: String) -> Self {
        let action_op = Arc::new(|_, _, world: &'_ mut World| {
            C::default().apply(world);
        });

        ContextAction {
            action_name,
            action_op,
        }
    }
    pub fn from_command_clone<C: Command + Sync + Clone>(action_name: String, command: C) -> Self {
        let action_op = Arc::new(move |_, _, world: &'_ mut World| {
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
        let action_op = Arc::new(move |_, _, world: &'_ mut World| {
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
    context_actions: Query<(&ContextActions, Has<ContextActionsNoDefault>)>,
    source_settings: Query<CopiedOrDefault<ContextMenuSettings>>,
    context_action: Query<&ContextAction>,
) {
    for mouse_event in evr_mouse.read() {
        let id = mouse_event.entity();
        context_actions
            .get(id)
            .ok()
            .and_then(|(context_actions, no_default)| {
                let mb = match mouse_event.event_kind() {
                    // Should we swap this with up to enable draggable things too?
                    MouseEventTtyKind::Down(mousebutton) if mouse_event.is_top_entity() => {
                        *mousebutton
                    },
                    _ => return None,
                };
                let settings = source_settings
                    .get(context_actions.settings_source)
                    .expect("should default");

                let mouse_event = *mouse_event;
                let action = match settings.determine_action(
                    mb,
                    context_actions.actions.len(),
                    no_default,
                )? {
                    MouseButtonAction::PerformContextAction(n) => {
                        let context_action =
                            context_action.get(*context_actions.actions.get(n)?).ok()?;
                        log::trace!("Performing context action: {}", context_action.action_name);
                        context_action.action_op.clone()
                    },
                    MouseButtonAction::CycleContextAction => Arc::new(|_, _, _: &'_ mut World| {
                        log::error!("TODO Cycling context actions not implemented yet");
                    }),
                    MouseButtonAction::DisplayContextMenu => {
                        Arc::new(
                            move |id, mouse_event: MouseEventTty, world: &'_ mut World| {
                                for mut context_menu in
                                    world.query::<&mut ContextMenu>().iter_mut(world)
                                {
                                    // TODO what if there are multiple???
                                    context_menu.actions_context = Some(id);
                                    context_menu.position = mouse_event.absolute_pos();
                                    context_menu.mouse_event = Some(mouse_event);
                                }
                                let display_system =
                                    world.resource::<SystemIdDisplayContextMenu>().0;
                                world.run_system(display_system).unwrap();
                            },
                        )
                    },
                };
                commands.add(move |w: &'_ mut World| action(id, mouse_event, w));
                Some(())
            });
    }
}

fn sys_display_context_menu(
    mut commands: Commands,
    res_draw_config: Res<DrawConfiguration>,
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

        let context_menu_actions: Vec<_> = context_actions_q.get(context_actions_id).map(|context_actions|
            context_actions.actions.iter().copied().filter_map(|context_action_id| {
                match context_action_q.get(context_action_id) {
                    Ok(context_action) => Some(ContextMenuItem(context_action.action_name.clone(), context_action_id)),
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
            .map(|ca| ca.0.len())
            .max()
            .expect("should not be empty") as u32
            + 2;
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

        let mut charmi = CharacterMapImage::new();
        let cm_style = res_draw_config.color_scheme().context_menu();

        use taffy::prelude::*;
        charmi
            .new_row()
            .add_char('┍', &cm_style)
            .add_text("━".repeat((target_width - 2) as usize), &cm_style)
            .add_char('┑', &cm_style);
        commands
            .entity(cm_id)
            .despawn_descendants()
            .with_children(|cm_commands| {
                for (context_menu_item, row) in context_menu_actions.into_iter().zip(2..) {
                    charmi
                        .new_row()
                        .add_char('│', &cm_style)
                        // Not adding a gap - don't want this see-through
                        // If I ever add merging effects to charmi, might do some slightly opaque
                        .add_text(" ".repeat((target_width - 2) as usize), &cm_style)
                        .add_char('│', &cm_style);
                    cm_commands.spawn((
                        StyleTty(Style {
                            max_size: Size {
                                width: auto(),
                                height: length(1.),
                            },
                            grid_row: line(row),
                            grid_column: line(2),
                            ..default()
                        }),
                        MouseEventListener,
                        Name::new(format!("Context Menu Item [{}]", &context_menu_item.0)),
                        TerminalRendering::new(vec![context_menu_item.0.to_string()]), // Style to come later
                        HoverPoint::default(),
                        context_menu_item,
                    ));
                }
            });
        charmi
            .new_row()
            .add_char('└', &cm_style)
            .add_text("─".repeat((target_width - 2) as usize), &cm_style)
            .add_char('┘', &cm_style);
        rendering.update_charmie(charmi);

        pane_style.grid_template_rows =
            vec![length(target_pos_y as f32), length(target_height as f32)];
        pane_style.grid_template_columns =
            vec![length(target_pos_x as f32), length(target_width as f32)];
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
            if cm_timer.finished() {
                *is_visible = false;
                cm_timer.pause();
                cm_timer.reset();
                commands.entity(cm_id).despawn_descendants();
            } else if hover_point.is_some() {
                if !cm_timer.paused() {
                    cm_timer.pause();
                    cm_timer.reset();
                }
            } else if cm_timer.paused() {
                cm_timer.unpause()
            }
        }
    }
}

pub fn sys_context_menu_item_click(
    mut commands: Commands,
    mut evr_mouse: EventReader<MouseEventTty>,
    context_menu: Query<&ContextMenu>,
    context_menu_item: Query<(AsDerefCopied<ContextMenuItem>, AsDerefCopied<Parent>)>,
    context_action_q: Query<&ContextAction>, // TODO w/o disabled
) {
    for mouse_event in evr_mouse.read() {
        if !matches!(
            mouse_event.event_kind(),
            MouseEventTtyKind::Down(MouseButton::Left)
        ) {
            continue;
        }
        context_menu_item
            .get(mouse_event.entity())
            .ok()
            .and_then(|(ca_id, cm_id)| {
                let action = context_action_q.get(ca_id).ok()?.action_op.clone();
                let context_menu = context_menu.get(cm_id).ok()?;
                let mouse_event = context_menu
                    .mouse_event()
                    .expect("Should have a source mouse event if context menu is displayed");
                let id = context_menu.actions_context?;
                commands.add(move |w: &'_ mut World| {
                    if let Some(mut timer) = w.get_mut::<ContextMenuTimer>(cm_id) {
                        timer.unpause();
                        let duration = timer.duration();
                        timer.tick(duration);
                    }
                    action(id, mouse_event, w)
                });
                Some(())
            });
    }
}

fn sys_render_context_items(
    res_draw_config: Res<DrawConfiguration>,
    mut cmi_q: Query<(&ContextMenuItem, &HoverPoint, &mut TerminalRendering)>,
) {
    for (context_menu_item, hover_point, mut rendering) in cmi_q.iter_mut() {
        let mut charmi: CharacterMapImage = CharacterMapImage::new();
        let charmi_row = charmi.new_row();
        let style = if hover_point.is_some() {
            res_draw_config.color_scheme().context_menu_item_hover()
        } else {
            res_draw_config.color_scheme().context_menu_item()
        };
        charmi_row.add_text(context_menu_item.0.as_str(), &style);
        rendering.update_charmie(charmi);
    }
}
