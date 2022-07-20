mod dynamic_layout;

use bevy::prelude::*;
use dynamic_layout::{
    CharmieRenderingComponent, DynamicTextLayout, MenuUi, MenuUiItem, SimpleUi, TextRendering,
};
use game_core::Bounds;
use std::collections::HashMap;
use taffy::prelude::*;

pub fn start_with_charmie() {
    App::new()
        .insert_non_send_resource(Taffy::new())
        .add_startup_system(setup_node_layout)
        .init_resource::<crate::DrawConfiguration>()
        .add_system(calculate_layout)
        // .add_system_to_stage(CoreStage::PostUpdate, render_menu_system)
        .add_system_to_stage(CoreStage::PostUpdate, render_charmie::<MenuUi>)
        .add_system_to_stage(CoreStage::PostUpdate, render_charmie::<SimpleUi>)
        // .add_system(pause)
        .run()
}

fn setup_node_layout(mut taffy: NonSendMut<Taffy>, mut commands: Commands) {
    log::debug!("Hello whirled!");
    println!("Hello world");

    let menu_ui_id = commands
        .spawn()
        .insert(MenuUi {
            options: vec![
                MenuUiItem {
                    name: "Item A".to_string(),
                    onselect: vec![],
                },
                MenuUiItem {
                    name: "Item B".to_string(),
                    onselect: vec![],
                },
                MenuUiItem {
                    name: "Item C".to_string(),
                    onselect: vec![],
                },
                MenuUiItem {
                    name: "Item D".to_string(),
                    onselect: vec![],
                },
                MenuUiItem {
                    name: "Item E".to_string(),
                    onselect: vec![],
                },
                MenuUiItem {
                    name: "Item F".to_string(),
                    onselect: vec![],
                },
                /*MenuUiItem { name: "Item G But |its LONG, too long and gets truncated".to_string(), onselect: vec![]},
                MenuUiItem { name: "それは".to_string(), onselect: vec![]},
                MenuUiItem { name: "それはほんとうにすごい！".to_string(), onselect: vec![]},
                MenuUiItem { name: "Item J".to_string(), onselect: vec![]},
                MenuUiItem { name: "Item K".to_string(), onselect: vec![]},
                MenuUiItem { name: "Item L".to_string(), onselect: vec![]},
                MenuUiItem { name: "Item M".to_string(), onselect: vec![]},
                MenuUiItem { name: "Item N".to_string(), onselect: vec![]},
                */
            ],
            selected_option: Some(5),
            scroll_offset: 3,
        })
        .insert(TextRendering::default())
        .id();
    let simple_ui_id = commands
        .spawn()
        .insert(SimpleUi {
            draw: vec![
                "----|----|----|----|".to_string(),
                "Hullo Hey".to_string(),
                "hulloguvnaiamheretokillyou".to_string(),
            ],
        })
        .insert(TextRendering::default())
        .id();

    let menu_ui_node = taffy
        .new_node(
            taffy::prelude::Style {
                min_size: taffy::prelude::Size {
                    width: taffy::prelude::Dimension::Points(11.0),
                    height: taffy::prelude::Dimension::Points(4.0),
                },
                flex_grow: 1.0,
                ..Default::default()
            },
            &[],
        )
        .unwrap();
    let simple_ui_node = taffy
        .new_node(
            taffy::prelude::Style {
                min_size: taffy::prelude::Size {
                    width: taffy::prelude::Dimension::Points(10.0),
                    height: taffy::prelude::Dimension::Points(4.0),
                },
                ..Default::default()
            },
            &[],
        )
        .unwrap();

    commands.spawn().insert(DynamicTextLayout {
        root: taffy
            .new_node(
                taffy::prelude::Style {
                    size: taffy::prelude::Size {
                        width: taffy::prelude::Dimension::Points(25.0),
                        height: taffy::prelude::Dimension::Points(7.0),
                    },
                    ..Default::default()
                },
                &[menu_ui_node, simple_ui_node],
            )
            .unwrap(),
        bounds: Bounds(50, 50),
        cache: HashMap::new(),
        nodes: HashMap::from([(menu_ui_id, menu_ui_node), (simple_ui_id, simple_ui_node)]),
        focus: menu_ui_id,
    });
}

fn pause() {
    crossterm::event::read().unwrap();
}

// TODO In the future, use "Changed<DynamicTextLayouT>" filter https://bevy-cheatbook.github.io/programming/change-detection.html
fn calculate_layout(mut taffy: NonSendMut<Taffy>, text_layouts: Query<&DynamicTextLayout>) {
    log::debug!("Calculating layout");
    for text_layout in text_layouts.iter() {
        log::debug!("Inner calculating layout");
        if (*taffy).dirty(text_layout.root).unwrap_or(false) {
            taffy
                .compute_layout(
                    text_layout.root,
                    taffy::prelude::Size {
                        width: taffy::prelude::Number::Defined(text_layout.bounds.width() as f32),
                        height: taffy::prelude::Number::Defined(text_layout.bounds.height() as f32),
                    },
                )
                .unwrap();
            log::debug!(
                "Layout of root {:?}",
                taffy.layout(text_layout.root).unwrap()
            );
        } else {
            log::debug!(
                "Layout of root (nondirty) {:?}",
                taffy.layout(text_layout.root).unwrap()
            );
        }
    }
}

fn render_charmie<T: CharmieRenderingComponent>(
    taffy: NonSend<Taffy>,
    draw_config: Res<crate::DrawConfiguration>,
    mut menu_uis: Query<(Entity, &T, &mut TextRendering)>,
    views: Query<&DynamicTextLayout>,
) {
    // let draw_config = crate::DrawConfiguration::default();
    for (entity_id, crc, mut menu_rendering) in menu_uis.iter_mut() {
        log::debug!("Rendering a menu!");
        for view in views.iter() {
            log::debug!("Checking a view!");
            if view.nodes.contains_key(&entity_id) {
                log::debug!("Oh hey, it does contain it! Quick let's render");
                crc.render(
                    view,
                    &entity_id,
                    &*taffy,
                    &draw_config,
                    &mut *menu_rendering,
                );

                log::debug!("Menu rendering: {:?}", menu_rendering);
                for line in menu_rendering.draw.iter() {
                    println!("{}", line);
                }
                break;
            }
        }
    }
}
