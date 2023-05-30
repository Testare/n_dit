use crate::term::prelude::*;
use bevy::core::FrameCount;
use taffy::prelude::Style;
use unicode_width::UnicodeWidthStr;
use super::{TerminalWindow, render::TerminalRendering};


#[derive(Default)]
pub struct TaffyTuiLayoutPlugin;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct Taffy(taffy::Taffy);

#[derive(Component)]
pub struct RenderedLayout;

#[derive(Component, Debug, Deref, DerefMut)]
pub struct TuiNode(taffy::node::Node);

// TODO Users use this instead of TuiNode, or a marker component. TuiNode added by systems, like
// CalculatedSize
#[derive(Component, Debug, Deref, DerefMut)]
pub struct TuiStyle(taffy::prelude::Style);

impl TuiNode {
    pub fn new(taffy: &mut Taffy, style: Style) -> Self {
        let node = taffy.new_leaf(style).unwrap();
        TuiNode(node)
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
            .add_systems((
            taffy_follow_entity_model,
            calculate_layouts,
            render_layouts,
        ).chain());
    }
}

fn taffy_follow_entity_model(
    mut taffy: ResMut<Taffy>,
    nodes: Query<&TuiNode>,
    new_child_nodes: Query<(&TuiNode, &Children), Changed<Children>>,
) {
    for (parent, children) in new_child_nodes.iter() {
        let children_nodes: Vec<taffy::node::Node> = nodes
            .iter_many(children)
            .map(|node| **node)
            .collect();
        taffy.set_children(**parent, &children_nodes).unwrap();
    }
}

fn calculate_layouts(
    mut taffy: ResMut<Taffy>,
    window: Res<TerminalWindow>,
    roots: Query<&TuiNode, Without<Parent>>,
    tui_nodes: Query<(&TuiNode, &Name)>,
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
            taffy.set_style(**root, Style {
                size: window_size,
                ..root_style
            }).unwrap();
        }
        if size_changed || (*taffy).dirty(**root).unwrap_or(false) {
            taffy
                .compute_layout(
                    **root,
                    space.clone()
                )
                .unwrap();
            log::trace!("Recalculated Layout of root {:?}", taffy.layout(**root).unwrap());
        } 
    }
    for (node, name)  in tui_nodes.iter() {
        log::debug!("{} layout: {:?}", name.as_str(), taffy.layout(**node));

    }
}

pub fn render_layouts(
    taffy: Res<Taffy>,
    frame_count: Res<FrameCount>,
    mut render_layouts: Query<(Entity, &TuiNode, &mut TerminalRendering), With<RenderedLayout>>,
    children: Query<&Children>,
    child_renderings: Query<(&TuiNode, &TerminalRendering), Without<RenderedLayout>>
) {
    // do nothing
    for (root_id, render_layout_node, mut rendering) in render_layouts.iter_mut() {
        use pad::PadStr;
        struct LeafInfo<'a> {
            rendering: &'a TerminalRendering,
            layout: &'a taffy::prelude::Layout,
        }

        let leaves = collect_leaves(root_id, &children);
        let mut rendered_leaves: Vec<LeafInfo> = child_renderings.iter_many(leaves).map(|(node, rendering)|{
            let layout = taffy.layout(**node).unwrap();
            LeafInfo {
                rendering,
                layout
            }
        }).collect();
        rendered_leaves.sort_by_cached_key(|leaf_info| {
            (leaf_info.layout.location.x as u32, leaf_info.layout.location.y as u32)
        });
        
        let root_layout = taffy.layout(**render_layout_node).unwrap();
        let root_width = root_layout.size.width as usize;
        let mut rows = vec![String::default() ; root_layout.size.height as usize];
        for leaf in rendered_leaves {
            let x_offset = leaf.layout.location.x as usize;
            let y_offset = leaf.layout.location.y as usize;
            for (i, child_row) in leaf.rendering.rendering().iter().enumerate() {
                let row = &mut rows[i + y_offset];
                let row_len = UnicodeWidthStr::width(row.as_str());
                let new_row_string = format!(
                    "{current_row}{space:padding$}{child_row}",
                    current_row = row,
                    space = " ",
                    padding = x_offset - row_len,
                    child_row = child_row);
                *row = new_row_string;
            }
        }
        let new_rendering = rows.into_iter().map(|row| row.pad_to_width(root_width)).collect();
        rendering.update(new_rendering, frame_count.0);

    }
}

// Helper function

pub fn collect_leaves(root: Entity, children_query: &Query<&Children>) -> Vec<Entity> {
    if let Ok(children) = children_query.get(root) {
        children.into_iter().flat_map(|child|collect_leaves(*child, children_query)).collect()
    } else {
        vec![root]
    }
}