use charmi::CharacterMapImage;
use game_core::player::ForPlayer;
use getset::{CopyGetters, Getters};
use pad::PadStr;
use taffy::prelude::Style;

use super::input_event::{KeyModifiers, MouseEventKind};
use super::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};
use super::TerminalWindow;
use crate::prelude::*;

#[derive(Default)]
pub struct TaffyTuiLayoutPlugin;

#[derive(Default, Deref, DerefMut, Resource)]
struct Taffy(taffy::Taffy);

/// Hidden component, ties Entity to Taffy Node
#[derive(Component, Debug, Deref, DerefMut)]
struct NodeTty(taffy::node::Node);

/// Root of a layout. Is fitted to terminal
#[derive(Component)]
pub struct LayoutRoot;

#[derive(Component, CopyGetters, Debug, Event, Getters)]
pub struct LayoutEvent {
    #[getset(get_copy = "pub")]
    entity: Entity,
    #[getset(get_copy = "pub")]
    pos: UVec2,
    #[getset(get = "pub")]
    modifiers: KeyModifiers,
    #[getset(get = "pub")]
    event_kind: MouseEventKind,
    #[getset(get_copy = "pub")]
    double_click: bool,
}

#[derive(Component, Debug)]
pub struct LayoutMouseTarget;

/// Indicates a UI element that should be focused on
/// when clicked on.
#[derive(Component, Debug)]
pub struct UiFocusOnClick;

/// Indicates the UI element that is being focused on for
/// controls. Added on a player entity.
/// If it is empty, default controls can be defined.
/// Does not have to have a [UiFocusOnClick] component, but
/// should have a [StyleTty] component.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct UiFocus(pub Option<Entity>);

#[derive(Bundle, Default)]
pub struct UiFocusBundle {
    ui_focus: UiFocus,
    ui_focus_next: UiFocusNext,
}

/// When focus shifts from one UI element to the next, we set it here first so
/// that we don't have inputs counted multiple times
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct UiFocusNext(pub Option<Entity>);

#[derive(Component, Debug, Deref, DerefMut)]
pub struct UiFocusCycleOrder(pub u32);

/// Part of a layout, defines the style
#[derive(Component, Debug, Deref, DerefMut)]
pub struct StyleTty(pub taffy::prelude::Style);

// Actually these components probably should be part of render
#[derive(Component, Debug, Default, Deref)]
pub struct GlobalTranslationTty(UVec2);

#[derive(Component, Debug, Default, Deref)]
pub struct CalculatedSizeTty(UVec2);

impl NodeTty {
    fn new(taffy: &mut Taffy, style: Style) -> Self {
        let node = taffy.new_leaf(style).unwrap();
        NodeTty(node)
    }
}

impl CalculatedSizeTty {
    pub fn width32(&self) -> u32 {
        self.0.x
    }

    pub fn height32(&self) -> u32 {
        self.0.y
    }

    pub fn width(&self) -> usize {
        self.0.x as usize
    }

    pub fn height(&self) -> usize {
        self.0.y as usize
    }
}

impl Plugin for TaffyTuiLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Taffy>()
            .add_event::<LayoutEvent>()
            .add_systems(Last, remove_ui_focus_if_not_displayed)
            .add_systems(PreUpdate, generate_layout_events)
            .add_systems(
                RENDER_TTY_SCHEDULE,
                (
                    (
                        taffy_apply_style_updates,
                        taffy_new_style_components,
                        apply_deferred,
                        taffy_apply_hierarchy_updates,
                        calculate_layouts,
                    )
                        .chain()
                        .in_set(RenderTtySet::CalculateLayout),
                    (apply_deferred, render_layouts)
                        .chain()
                        .in_set(RenderTtySet::RenderLayouts),
                ),
            );
    }
}

fn remove_ui_focus_if_not_displayed(
    mut ui_foci: Query<&mut UiFocusNext>,
    styles: Query<&StyleTty>,
) {
    for mut next_focus in ui_foci.iter_mut() {
        if next_focus.is_some() {
            let not_focusable = next_focus
                .and_then(|focus| {
                    let style = styles.get(focus).ok()?;
                    Some(style.display == taffy::style::Display::None)
                })
                .unwrap_or(true);
            if not_focusable {
                **next_focus = None;
            }
        }
    }
}

fn generate_layout_events(
    mut crossterm_events: EventReader<MouseEvent>,
    mut layout_event_writer: EventWriter<LayoutEvent>,
    layout_elements: Query<
        (Entity, &CalculatedSizeTty, &GlobalTranslationTty),
        (With<StyleTty>, With<LayoutMouseTarget>),
    >,
    mut last_click: Local<Option<(std::time::Instant, MouseEvent)>>,
) {
    for event @ MouseEvent(crossterm::event::MouseEvent {
        kind,
        column,
        row,
        modifiers,
    }) in crossterm_events.iter()
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

        for (entity, size, translation) in layout_elements.iter() {
            if translation.x <= event_x
                && event_x < (translation.x + size.width32())
                && translation.y <= event_y
                && event_y < (translation.y + size.height32())
            {
                let pos = UVec2 {
                    x: event_x - translation.x,
                    y: event_y - translation.y,
                };
                layout_event_writer.send(LayoutEvent {
                    entity,
                    pos,
                    modifiers: modifiers.clone(),
                    event_kind: kind.clone(),
                    double_click,
                })
            }
        }
        if matches!(kind, MouseEventKind::Down(_)) {
            last_click.replace((std::time::Instant::now(), *event));
        }
    }
}

fn taffy_new_style_components(
    mut commands: Commands,
    mut taffy: ResMut<Taffy>,
    new_styles: Query<(Entity, &StyleTty), (Added<StyleTty>, Without<NodeTty>)>,
) {
    for (id, style) in new_styles.iter() {
        commands.get_entity(id).unwrap().insert((
            NodeTty::new(&mut taffy, **style),
            CalculatedSizeTty::default(),
            GlobalTranslationTty::default(),
        ));
    }
}

fn taffy_apply_style_updates(
    mut taffy: ResMut<Taffy>,
    changed_styles: Query<(&NodeTty, &StyleTty), Changed<StyleTty>>,
) {
    for (node_id, style) in changed_styles.iter() {
        (**taffy).set_style(**node_id, **style).unwrap()
    }
}

fn taffy_apply_hierarchy_updates(
    mut taffy: ResMut<Taffy>,
    nodes: Query<&NodeTty>,
    new_child_nodes: Query<(&NodeTty, &Children), Changed<Children>>,
) {
    for (parent, children) in new_child_nodes.iter() {
        let children_nodes: Vec<taffy::node::Node> =
            nodes.iter_many(children).map(|node| **node).collect();
        taffy.set_children(**parent, &children_nodes).unwrap();
    }
}

fn calculate_layouts(
    mut taffy: ResMut<Taffy>,
    window: Res<TerminalWindow>,
    roots: Query<(Entity, &NodeTty), Without<Parent>>,
    children: Query<&Children>,
    mut tui_nodes: Query<(
        &NodeTty,
        &mut CalculatedSizeTty,
        &mut GlobalTranslationTty,
        Option<&Name>,
    )>,
) {
    use taffy::prelude::*;
    let space = Size {
        width: AvailableSpace::Definite(window.width() as f32),
        height: AvailableSpace::Definite(window.height() as f32),
    };
    let window_size = Size {
        width: Dimension::Points(window.width() as f32),
        height: Dimension::Points(window.height() as f32),
    };
    for (root_id, root) in roots.iter() {
        let root_style = taffy.style(**root).cloned().unwrap();
        let size_changed = root_style.size != window_size;

        if size_changed {
            taffy
                .set_style(
                    **root,
                    Style {
                        size: window_size,
                        ..root_style
                    },
                )
                .unwrap();
        }
        if size_changed || (*taffy).dirty(**root).unwrap_or(false) {
            taffy.compute_layout(**root, space.clone()).unwrap();
            log::debug!("Recalculated Layouts");
            update_layout_traversal(root_id, &children, UVec2::default(), &mut |id, offset| {
                if let Ok((node, mut size, mut translation, name_opt)) = tui_nodes.get_mut(id) {
                    let layout = taffy.layout(**node).unwrap();
                    if let Some(name) = name_opt {
                        log::trace!("{} layout: {:?}", name.as_str(), layout);
                    }
                    translation.0.x = layout.location.x as u32 + offset.x;
                    translation.0.y = layout.location.y as u32 + offset.y;
                    size.0.x = layout.size.width as u32;
                    size.0.y = layout.size.height as u32;
                    translation.0
                } else {
                    log::warn!("Child of TUI component without all TUI components, possible weird behavior: {:?}", id);
                    offset
                }
            })
        }
    }
}

pub fn render_layouts(
    mut render_layouts: Query<
        (Entity, &CalculatedSizeTty, Option<&mut TerminalRendering>),
        With<LayoutRoot>,
    >,
    children: Query<&Children>,
    child_renderings: Query<(&TerminalRendering, &GlobalTranslationTty), Without<LayoutRoot>>,
) {
    for (root_id, root_size, rendering) in render_layouts.iter_mut() {
        let mut leaves: Vec<(&TerminalRendering, &GlobalTranslationTty)> =
            collect_leaves(root_id, &children)
                .into_iter()
                .filter_map(|id| child_renderings.get(id).ok())
                .collect();
        leaves.sort_by_cached_key(|leaf_info| (leaf_info.1.x as u32, leaf_info.1.y as u32));

        let mut charmie = CharacterMapImage::new();
        for leaf in leaves {
            charmie = charmie.draw(leaf.0.charmie(), leaf.1.x, leaf.1.y, Default::default());
        }
        charmie.fit_to_size(root_size.width32(), root_size.height32());

        if let Some(mut rendering) = rendering {
            rendering.update_charmie(charmie);
        }
    }
}

// Helper function

pub fn ui_focus_cycle_next(
    from_entity: Option<Entity>,
    player: Entity,
    default_pos: u32,
    ui_nodes: &Query<(Entity, &StyleTty, &UiFocusCycleOrder, &ForPlayer)>,
) -> Option<Entity> {
    let from_pos = from_entity
        .and_then(|entity| ui_nodes.get(entity).ok())
        .map(|player| **player.2)
        .unwrap_or(default_pos);
    let mut candidate_pos = from_pos;
    ui_nodes.iter().fold(
        from_entity,
        |current_candidate,
         (candidate, style, UiFocusCycleOrder(pos), ForPlayer(candidate_player))| {
            if *candidate_player != player || style.display == taffy::prelude::Display::None {
                current_candidate
            } else if current_candidate.is_none() {
                candidate_pos = *pos;
                Some(candidate)
            } else if from_pos < *pos {
                if candidate_pos <= from_pos {
                    candidate_pos = *pos;
                    Some(candidate)
                } else if *pos < candidate_pos {
                    candidate_pos = *pos;
                    Some(candidate)
                } else {
                    current_candidate
                }
            } else if candidate_pos <= from_pos && *pos < candidate_pos {
                candidate_pos = *pos;
                Some(candidate)
            } else {
                current_candidate
            }
        },
    )
}

pub fn ui_focus_cycle_prev(
    from_entity: Option<Entity>,
    player: Entity,
    default_pos: u32,
    ui_nodes: &Query<(Entity, &StyleTty, &UiFocusCycleOrder, &ForPlayer)>,
) -> Option<Entity> {
    let from_pos = from_entity
        .and_then(|entity| ui_nodes.get(entity).ok())
        .map(|player| **player.2)
        .unwrap_or(default_pos);
    let mut candidate_pos = from_pos;
    ui_nodes.iter().fold(
        from_entity,
        |current_candidate,
         (candidate, style, UiFocusCycleOrder(pos), ForPlayer(candidate_player))| {
            if *candidate_player != player || style.display == taffy::prelude::Display::None {
                current_candidate
            } else if current_candidate.is_none() {
                candidate_pos = *pos;
                Some(candidate)
            } else if from_pos > *pos {
                if candidate_pos >= from_pos {
                    candidate_pos = *pos;
                    Some(candidate)
                } else if *pos > candidate_pos {
                    candidate_pos = *pos;
                    Some(candidate)
                } else {
                    current_candidate
                }
            } else if candidate_pos >= from_pos && *pos > candidate_pos {
                candidate_pos = *pos;
                Some(candidate)
            } else {
                current_candidate
            }
        },
    )
}

fn update_layout_traversal<F: FnMut(Entity, UVec2) -> UVec2>(
    current: Entity,
    children_query: &Query<&Children>,
    accumulated_offset: UVec2,
    update_fn: &mut F,
) {
    let new_offset = update_fn(current, accumulated_offset);
    if let Ok(children) = children_query.get(current) {
        for child in children.into_iter() {
            update_layout_traversal(*child, children_query, new_offset, update_fn);
        }
    }
}

// TODO Make it so if a Node has NodeTty and children, but the children don't have NodeTty, it counts as a leaf
// Or just... That children are rendered on top/below parent nodes? IDK
pub fn collect_leaves(root: Entity, children_query: &Query<&Children>) -> Vec<Entity> {
    if let Ok(children) = children_query.get(root) {
        children
            .into_iter()
            .flat_map(|child| collect_leaves(*child, children_query))
            .collect()
    } else {
        vec![root]
    }
}

pub trait FitToSize {
    fn fit_to_size(self, size: &CalculatedSizeTty) -> Self;
}

impl FitToSize for Vec<String> {
    fn fit_to_size(mut self, size: &CalculatedSizeTty) -> Self {
        self.truncate(size.height());
        for line in self.iter_mut() {
            *line = line.with_exact_width(size.width())
        }
        self
    }
}
