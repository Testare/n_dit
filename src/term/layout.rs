use super::{render::TerminalRendering, TerminalWindow};
use crate::term::prelude::*;
use bevy::core::FrameCount;
use pad::PadStr;
use taffy::prelude::Style;
use unicode_width::UnicodeWidthStr;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum LayoutSet {
    RenderLeaves,
    RenderRoots,
}

#[derive(Default)]
pub struct TaffyTuiLayoutPlugin;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct Taffy(taffy::Taffy);

#[derive(Component)]
pub struct LayoutRoot;

#[derive(Component, Debug, Deref, DerefMut)]
pub struct NodeTty(taffy::node::Node);

// TODO Users use this instead of TuiNode, or a marker component. TuiNode added by systems, like
// CalculatedSize
#[derive(Component, Debug, Deref, DerefMut)]
pub struct CalculatedLayoutTty(taffy::prelude::Layout);

impl NodeTty {
    pub fn new(taffy: &mut Taffy, style: Style) -> Self {
        let node = taffy.new_leaf(style).unwrap();
        NodeTty(node)
    }
}

#[derive(Component, Debug)]
pub struct TuiCalculations {
    pub size: UVec2,
    pub transform: UVec2,
}

impl Plugin for TaffyTuiLayoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Taffy>()
            .add_systems(
                (taffy_follow_entity_model, calculate_layouts)
                    .chain()
                    .before(LayoutSet::RenderLeaves),
            )
            .add_system(render_layouts.in_set(LayoutSet::RenderRoots))
            .configure_set(LayoutSet::RenderLeaves.before(LayoutSet::RenderRoots));
    }
}

fn taffy_follow_entity_model(
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
    roots: Query<&NodeTty, Without<Parent>>,
    tui_nodes: Query<(&NodeTty, &Name)>,
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
    for root in roots.iter() {
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
        }
    }
    for (node, name) in tui_nodes.iter() {
        log::debug!("{} layout: {:?}", name.as_str(), taffy.layout(**node));
    }
}

pub fn render_layouts(
    mut commands: Commands,
    taffy: Res<Taffy>,
    frame_count: Res<FrameCount>,
    mut render_layouts: Query<(Entity, &NodeTty, Option<&mut TerminalRendering>), With<LayoutRoot>>,
    children: Query<&Children>,
    nodes: Query<&NodeTty>,
    child_renderings: Query<&TerminalRendering, Without<LayoutRoot>>,
) {
    for (root_id, render_layout_node, rendering) in render_layouts.iter_mut() {
        struct LeafInfo<'a> {
            rendering: &'a TerminalRendering,
            x: u32,
            y: u32,
        }
        #[derive(Default, Clone)]
        struct RowInfo {
            text: String,
            // later might include padding/border/margin information
        }

        let mut leaves: Vec<LeafInfo> = collect_leaves(root_id, &children, &|id| {
            let child_tty = nodes.get(id).unwrap();
            let location = taffy.layout(**child_tty).unwrap().location;
            UVec2 {
                x: location.x as u32,
                y: location.y as u32,
            }
        })
        .into_iter()
        .filter_map(|(pos_offset, id)| {
            let rendering = child_renderings.get(id).ok()?;
            Some(LeafInfo {
                rendering,
                x: pos_offset.x,
                y: pos_offset.y,
            })
        })
        .collect();
        leaves.sort_by_cached_key(|leaf_info| (leaf_info.x as u32, leaf_info.y as u32));

        let root_layout = taffy.layout(**render_layout_node).unwrap();
        let root_width = root_layout.size.width as usize;
        let mut rows = vec![RowInfo::default(); root_layout.size.height as usize];
        for leaf in leaves {
            let x_offset = leaf.x as usize;
            let y_offset = leaf.y as usize;
            for (i, child_row) in leaf.rendering.rendering().iter().enumerate() {
                let row = &mut rows[i + y_offset];
                let row_len = UnicodeWidthStr::width((*row).text.as_str());
                let new_row_text = format!(
                    "{current_row}{space:padding$}{child_row}",
                    current_row = row.text,
                    space = " ",
                    padding = x_offset - row_len,
                    child_row = child_row
                );
                row.text = new_row_text;
            }
        }
        let padded_rendering = rows
            .into_iter()
            .map(|row| row.text.pad_to_width(root_width))
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

/*
pub fn collect_leaves(root: Entity, children_query: &Query<&Children>) -> Vec<Entity> {
    if let Ok(children) = children_query.get(root) {
        children
            .into_iter()
            .flat_map(|child| collect_leaves(*child, children_query))
            .collect()
    } else {
        vec![root]
    }
}*/

pub fn collect_leaves<F: Fn(Entity) -> UVec2>(
    root: Entity,
    children_query: &Query<&Children>,
    get_xy: &F,
) -> Vec<(UVec2, Entity)> {
    let rootxy @ UVec2 {
        x: root_x,
        y: root_y,
    } = get_xy(root);
    if let Ok(children) = children_query.get(root) {
        children
            .into_iter()
            .flat_map(|child| {
                // Actually, scrap this whole approach. Pass a Fn(Entity) -> UVec2 to get the point of each parent, and pass the accumulated point on to the the recursions
                collect_leaves(*child, children_query, get_xy)
                    .into_iter()
                    .map(|(UVec2 { x, y }, id)| {
                        (
                            UVec2 {
                                x: x + root_x,
                                y: y + root_y,
                            },
                            id,
                        )
                    })
            })
            .collect()
    } else {
        vec![(rootxy, root)]
    }
}
