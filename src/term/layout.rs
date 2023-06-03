use super::{render::RenderTtySet, render::TerminalRendering, TerminalWindow};
use crate::term::prelude::*;
use bevy::core::FrameCount;
use pad::PadStr;
use taffy::prelude::Style;
use unicode_width::UnicodeWidthStr;

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
            .add_systems(
                (
                    taffy_apply_style_updates,
                    taffy_new_style_components,
                    apply_system_buffers,
                    taffy_apply_hierarchy_updates,
                    calculate_layouts,
                )
                    .chain()
                    .in_set(RenderTtySet::CalculateLayout),
            )
            .add_system(render_layouts.in_set(RenderTtySet::RenderLayouts));
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
            log::trace!(
                "Recalculated Layout of root {:?}",
                taffy.layout(**root).unwrap()
            );
            update_layout_traversal(root_id, &children, UVec2::default(), &mut |id, offset| {
                if let Ok((node, mut size, mut translation, name_opt)) = tui_nodes.get_mut(id) {
                    let layout = taffy.layout(**node).unwrap();
                    log::debug!(
                        "{} layout: {:?}",
                        name_opt.map(|name| name.as_str()).unwrap_or("Unnamed"),
                        layout
                    );
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
    mut commands: Commands,
    frame_count: Res<FrameCount>,
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

        let root_width = root_size.width32() as usize;
        let mut rows = vec![String::default(); root_size.height32() as usize];
        for leaf in leaves {
            let x_offset = leaf.1.x as usize;
            let y_offset = leaf.1.y as usize;
            for (i, child_row) in leaf.0.rendering().iter().enumerate() {
                let row = &mut rows[i + y_offset];
                let row_len = UnicodeWidthStr::width(row.as_str());
                let new_row_text = format!(
                    "{current_row}{space:padding$}{child_row}",
                    current_row = row,
                    space = "",
                    padding = x_offset - row_len,
                    child_row = child_row
                );
                *row = new_row_text;
            }
        }
        let padded_rendering = rows
            .into_iter()
            .map(|row| row.pad_to_width(root_width))
            .collect();

        if let Some(mut rendering) = rendering {
            rendering.update(padded_rendering, frame_count.0);
        } else {
            log::debug!("Adding layout rendering");
            let rendering = TerminalRendering::new(padded_rendering, frame_count.0);
            commands.get_entity(root_id).unwrap().insert(rendering);
        }
    }
}

// Helper function

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
