use crate::{DrawConfiguration, SuperState, UiAction};
use bevy::{
    ecs::bundle::Bundle,
    prelude::{Component, Entity},
};
use old_game_core::{Bounds, Point, Pt};
use pad::PadStr;
use std::ops::Deref;
use taffy::{style::Style as TaffyStyle, Taffy};

pub trait CharmieRenderingComponent: Component + Sized {
    // Consider: Change dependency on "view" and "entity" to simply "size"
    fn render(&self, size: &Bounds, draw_config: &DrawConfiguration, output: &mut TextRendering);

    fn bundle(self, taffy: &mut Taffy, style: TaffyStyle) -> CharmieRenderingBundle<Self> {
        let node_id = taffy.new_node(style, &[]).unwrap();
        CharmieRenderingBundle {
            crc: self,
            node: TaffyNodeComponent {
                node: node_id,
                style,
            },
            text_rendering: Default::default(),
        }
    }

    // fn click( view, entity, taffy, EventWriter<UiAction>)
    // hide?
}

#[derive(Bundle)]
pub struct CharmieRenderingBundle<T: CharmieRenderingComponent> {
    pub crc: T,
    pub node: TaffyNodeComponent,
    pub text_rendering: TextRendering,
}
// Renders a set of multiple options
#[derive(Component)]
pub struct MenuUi {
    pub options: Vec<MenuUiItem>,
    pub selected_option: Option<usize>,
    pub scroll_offset: usize,
}

#[derive(Component)]
pub struct TaffyNodeComponent {
    pub node: taffy::node::Node,
    pub style: taffy::style::Style,
}

impl TaffyNodeComponent {
    pub fn new(taffy: &mut Taffy, style: TaffyStyle) -> Self {
        let node = taffy.new_node(style.clone(), &[]).unwrap();
        TaffyNodeComponent { node, style }
    }
}

impl Deref for TaffyNodeComponent {
    type Target = taffy::node::Node;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

pub struct MenuUiItem {
    pub name: String,
    pub onselect: Vec<UiAction>,
}

// For the gird map
#[derive(Component, Debug)]
struct GridMapUi {
    selected_square: Point,
    scrolling: Pt<isize>,
}

// Used for simple text display.
#[derive(Component, Debug)]
pub struct SimpleUi {
    pub draw: Vec<String>,
}

// Like SimpleUI, but the text wraps.
#[derive(Component, Debug)]
struct MessageUi {
    message: String,
}

// Result of rendering any UI component
// Might need to include positioning for render?
#[derive(Component, Debug, Default)]
pub struct TextRendering {
    pub draw: Vec<String>,
    pub location: Point,
    pub size: Bounds,
}

#[derive(Component)]
pub struct DynamicTextLayout {
    // pub root: TaffyNode,
    pub bounds: Bounds,
    // pub nodes: HashMap<Entity, TaffyNode>,
}

impl TaffyNodeComponent {
    pub fn bounds(&self, taffy: &Taffy) -> Option<Bounds> {
        let layout = taffy.layout(self.node).ok()?;
        let taffy::geometry::Size { width, height } = layout.size;
        let taffy::geometry::Point { x, y } = layout.location; //taffy::geometry::Point
        let x2 = x + width;
        let y2 = y + height;
        Some(Bounds::of(
            (x2.ceil() - x.ceil()) as usize,
            (y2.ceil() - y.ceil()) as usize,
        ))
    }
    pub fn bounds_and_pt(&self, taffy: &Taffy) -> Option<(Bounds, Point)> {
        let layout = taffy.layout(self.node).ok()?;
        let taffy::geometry::Size { width, height } = layout.size;
        let taffy::geometry::Point { x, y } = layout.location; //taffy::geometry::Point
        let x2 = x + width;
        let y2 = y + height;
        let pt = (x.ceil() as usize, y.ceil() as usize);
        let bounds = Bounds::of((x2.ceil() as usize - pt.0), (y2.ceil() as usize - pt.1));
        Some((bounds, pt))
    }
}

impl CharmieRenderingComponent for MenuUi {
    fn render(&self, size: &Bounds, draw_config: &DrawConfiguration, output: &mut TextRendering) {
        let options: Vec<String> = self
            .options
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .map(|(i, menu_item)| {
                let line_item = menu_item.name.with_exact_width(size.width());
                if Some(i) == self.selected_option {
                    draw_config
                        .color_scheme()
                        .selected_menu_item()
                        .apply(line_item)
                } else {
                    line_item
                }
            })
            .chain(std::iter::repeat("".pad_to_width(size.width())))
            .take(size.height())
            .collect();
        output.draw = options;
    }
}

impl CharmieRenderingComponent for SimpleUi {
    fn render(&self, size: &Bounds, _draw_config: &DrawConfiguration, output: &mut TextRendering) {
        output.draw = self
            .draw
            .iter()
            .map(|s| s.with_exact_width(size.width()))
            .chain(std::iter::repeat("".pad_to_width(size.width())))
            .take(size.height())
            .collect();
    }
}
