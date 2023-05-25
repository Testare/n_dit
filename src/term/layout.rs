use crate::term::prelude::*;
use taffy::prelude::Style;
use super::TerminalWindow;

#[derive(Default)]
pub struct TaffyTuiLayoutPlugin;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct Taffy(taffy::Taffy);


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
        ));
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
