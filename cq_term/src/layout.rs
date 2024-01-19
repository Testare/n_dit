use std::collections::VecDeque;

use charmi::CharacterMapImage;
use game_core::player::ForPlayer;
use pad::PadStr;
use serde::{Deserialize, Serialize};
use taffy::prelude::{Display, Style};

use super::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};
use super::TerminalWindow;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct TaffyTuiLayoutPlugin;

#[derive(Deref, DerefMut, Resource)]
struct Taffy(taffy::Taffy);

impl Default for Taffy {
    fn default() -> Self {
        let mut taffy = taffy::Taffy::default();
        taffy.disable_rounding();
        Taffy(taffy)
    }
}

/// Hidden component, ties Entity to Taffy Node
#[derive(Component, Debug, Deref, DerefMut)]
struct NodeTty(taffy::node::Node);

/// Root of a layout. Is fitted to terminal
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct LayoutRoot;

/// Indicates a UI element that should be focused on
/// when clicked on.
///
/// Not sure if this is still being used
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct UiFocusOnClick;

/// Indicates the UI element that is being focused on for
/// controls. Added on a player entity.
/// If it is empty, default controls can be defined.
/// Does not have to have a [UiFocusOnClick] component, but
/// should have a [StyleTty] component.
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct UiFocus(pub Option<Entity>);

#[derive(Bundle, Debug, Default)]
pub struct UiFocusBundle {
    ui_focus: UiFocus,
}

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct UiFocusCycleOrder(pub u32);

/// Part of a layout, defines the style
#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Serialize, Deserialize, Reflect)]
#[reflect_value(Component, Serialize, Deserialize)]
#[serde(default, transparent)]
pub struct StyleTty(pub taffy::prelude::Style);

// Actually these components probably should be part of render
#[derive(Clone, Component, Copy, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct GlobalTranslationTty(pub UVec2);

#[derive(Clone, Component, Copy, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct CalculatedSizeTty(pub UVec2);

#[derive(Clone, Component, Copy, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct VisibilityTty(pub bool);

type IsVisibleTty = AsDerefOrBool<VisibilityTty, true>;

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

impl Default for VisibilityTty {
    fn default() -> Self {
        VisibilityTty(true)
    }
}

impl VisibilityTty {
    pub fn invisible() -> Self {
        VisibilityTty(false)
    }
}

impl StyleTty {
    fn taffy_style(&self, visible: bool) -> Style {
        if visible {
            self.0.clone()
        } else {
            Style {
                display: Display::None,
                ..self.0.clone()
            }
        }
    }

    pub fn buffer() -> StyleTty {
        StyleTty(taffy::prelude::Style {
            flex_grow: 1.0,
            ..default()
        })
    }
}

impl Plugin for TaffyTuiLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Taffy>()
            .register_type::<CalculatedSizeTty>()
            .register_type::<GlobalTranslationTty>()
            .register_type::<LayoutRoot>()
            .register_type::<StyleTty>()
            .register_type::<UiFocus>()
            .register_type::<UiFocusCycleOrder>()
            .register_type::<UiFocusOnClick>()
            .register_type::<VisibilityTty>()
            .add_systems(Last, remove_ui_focus_if_not_displayed)
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
    mut ui_foci: Query<&mut UiFocus>,
    styles: Query<(&StyleTty, Option<&VisibilityTty>)>,
) {
    for mut focus in ui_foci.iter_mut() {
        if focus.is_some() {
            let not_focusable = focus
                .and_then(|focus| {
                    let (style, visibility) = styles.get(focus).ok()?;
                    Some(
                        style.display == taffy::style::Display::None
                            || !*visibility.copied().unwrap_or_default(),
                    )
                })
                .unwrap_or(true);
            if not_focusable {
                **focus = None;
            }
        }
    }
}

fn taffy_new_style_components(
    mut commands: Commands,
    mut taffy: ResMut<Taffy>,
    new_styles: Query<(Entity, &StyleTty, IsVisibleTty), (Added<StyleTty>, Without<NodeTty>)>,
) {
    for (id, style, vis) in new_styles.iter() {
        commands.get_entity(id).unwrap().insert((
            NodeTty::new(&mut taffy, style.taffy_style(vis)),
            CalculatedSizeTty::default(),
            GlobalTranslationTty::default(),
        ));
    }
}

fn taffy_apply_style_updates(
    mut taffy: ResMut<Taffy>,
    changed_styles: Query<
        (
            &NodeTty,
            &StyleTty,
            OrBool<AsDerefCopied<VisibilityTty>, true>,
        ),
        Or<(Changed<StyleTty>, Changed<VisibilityTty>)>,
    >,
) {
    for (node_id, style, vis) in changed_styles.iter() {
        (**taffy)
            .set_style(**node_id, style.taffy_style(vis))
            .unwrap()
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
            taffy.compute_layout(**root, space).unwrap();
            log::debug!("Recalculated Layouts");
            update_layout_traversal(root_id, &children, UVec2::default(), &mut |id, offset| {
                if let Ok((node, mut size, mut translation, name_opt)) = tui_nodes.get_mut(id) {
                    let layout = taffy.layout(**node).unwrap();
                    if let Some(name) = name_opt {
                        log::trace!("{}[{id:?}] layout: {:?}", name.as_str(), layout);
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

/// System creates terminal rendering for nodes with LayoutRoot. It compiles all descendant node
/// renderings to the root node. So long as each node along the chain is visible and does not have
/// a LayoutRoot component (Though this may change in the future).
///
/// A node is considered visible if it either does NOT have a `VisibilityTty`` component, or that
/// VisibilityTty component is set to `true`.` Elements are drawn as they are visited, starting
/// from the root itself and going depth first, with later siblings drawn first. This means if you
/// want something to be drawn over another thing in the same space, it must be either a descendant
/// of that node, or be earlier in the sibling hierarchy (e.g., if node A has children `[B, C]`, B
/// will be drawn over C, which will be drawn over A)
///
/// Nodes without GlobalTranslationTty or TerminalRendering components can still be valid in the
/// heirarchy for rendering, but will not be rendered themselves. Be cautious though with the way
/// that'll affect other StyleTty/layout systems.
///
/// TODO consider moving this to render.rs and change "LayoutRoot" to "RenderRoot", as it doesn't
/// need layout specific logic.
///
/// TODO consider allowing RenderRoot children as intermediate steps of rendering? Could implement
/// by switching render target when a render root is encountered, and keeping
/// track of which have already been rendered so you don't render it again.
pub fn render_layouts(
    mut render_layouts: Query<
        (
            &CalculatedSizeTty,
            AsDeref<Children>,
            &mut TerminalRendering,
        ),
        With<LayoutRoot>,
    >,
    visibility: Query<AsDeref<VisibilityTty>>,
    q_children: Query<&Children, Without<LayoutRoot>>,
    child_renderings: Query<(&TerminalRendering, &GlobalTranslationTty), Without<LayoutRoot>>,
) {
    for (root_size, root_children, mut rendering) in render_layouts.iter_mut() {
        let mut children: VecDeque<Entity> = VecDeque::from_iter(root_children.iter().copied());

        let mut charmie = CharacterMapImage::new();
        while let Some(id) = children.pop_back() {
            if matches!(visibility.get(id), Ok(false)) {
                continue;
            }
            if let Ok(my_children) = q_children.get(id) {
                children.extend(&**my_children);
            }
            if let Ok((rendering, pos)) = child_renderings.get(id) {
                charmie = charmie.draw(rendering.charmie(), pos.x, pos.y, Default::default());
            }
        }

        charmie.fit_to_size(root_size.width32(), root_size.height32());
        rendering.update_charmie(charmie);
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
                if candidate_pos <= from_pos || *pos < candidate_pos {
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
                if candidate_pos >= from_pos || *pos > candidate_pos {
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
