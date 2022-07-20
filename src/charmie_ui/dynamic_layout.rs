use crate::{DrawConfiguration, SuperState, UiAction};
use bevy::prelude::{Component, Entity};
use game_core::{Bounds, Point, Pt};
use pad::PadStr;
use std::collections::{BTreeMap, HashMap};
use taffy::prelude::Node as TaffyNode;
use taffy::Taffy;

pub trait CharmieRenderingComponent: Component {
    // Consider: Change dependency on "view" and "entity" to simply "size"
    fn render(
        &self,
        view: &DynamicTextLayout,
        entity: &Entity,
        taffy: &Taffy,
        draw_config: &DrawConfiguration,
        output: &mut TextRendering,
    );

    // fn click( view, entity, taffy, EventWriter<UiAction>)
    // hide?
}

// Renders a set of multiple options
#[derive(Component)]
pub struct MenuUi {
    pub options: Vec<MenuUiItem>,
    pub selected_option: Option<usize>,
    pub scroll_offset: usize,
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
#[derive(Component, Debug, Default)]
pub struct TextRendering {
    pub draw: Vec<String>,
}

#[derive(Component)]
pub struct DynamicTextLayout {
    pub root: TaffyNode,
    pub bounds: Bounds,
    pub cache: HashMap<TaffyNode, Option<Vec<String>>>,
    pub nodes: HashMap<Entity, TaffyNode>,
    pub focus: Entity,
}

impl DynamicTextLayout {
    pub fn size(&self, taffy: &Taffy, entity: &Entity) -> Option<Bounds> {
        let node = self.nodes.get(entity)?;
        let layout = taffy.layout(*node).ok()?;
        let taffy::geometry::Size { width, height } = layout.size;
        let taffy::geometry::Point { x, y } = layout.location; //taffy::geometry::Point
        let x2 = x + width;
        let y2 = y + height;
        Some(Bounds::of(
            (x2.ceil() - x.ceil()) as usize,
            (y2.ceil() - y.ceil()) as usize,
        ))
    }
}

impl CharmieRenderingComponent for MenuUi {
    fn render(
        &self,
        view: &DynamicTextLayout,
        entity: &Entity,
        taffy: &Taffy,
        draw_config: &DrawConfiguration,
        output: &mut TextRendering,
    ) {
        log::debug!("{}", "hwa?");
        let menu_size = view.size(taffy, entity).unwrap();
        let options: Vec<String> = self
            .options
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .map(|(i, menu_item)| {
                let line_item = menu_item.name.with_exact_width(menu_size.width());
                if Some(i) == self.selected_option {
                    draw_config
                        .color_scheme()
                        .selected_menu_item()
                        .apply(line_item)
                } else {
                    line_item
                }
            })
            .chain(std::iter::repeat("".pad_to_width(menu_size.width())))
            .take(menu_size.height())
            .collect();
        output.draw = options;
    }
}

impl CharmieRenderingComponent for SimpleUi {
    fn render(
        &self,
        view: &DynamicTextLayout,
        entity: &Entity,
        taffy: &Taffy,
        draw_config: &DrawConfiguration,
        mut output: &mut TextRendering,
    ) {
        let size = view.size(taffy, entity).unwrap();
        output.draw = self
            .draw
            .iter()
            .map(|s| s.with_exact_width(size.width()))
            .chain(std::iter::repeat("".pad_to_width(size.width())))
            .take(size.height())
            .collect();
    }
}

fn click_event(state: &SuperState) {}
