mod dynamic_layout;

use bevy::prelude::*;
use dynamic_layout::{
    CharmieRenderingComponent, DynamicTextLayout, MenuUi, MenuUiItem, SimpleUi, TextRendering,
};
use game_core::{Bounds, GameState};
use pad::PadStr;
use taffy::prelude::*;

use self::dynamic_layout::TaffyNodeComponent;

use std::collections::{BTreeMap, VecDeque};

#[derive(Component, Debug)]
struct GameStateComponent(GameState);

pub fn start_with_charmie(state: GameState) {
    App::new()
        .insert_non_send_resource(Taffy::new())
        .add_startup_system(setup_node_layout)
        .init_resource::<crate::DrawConfiguration>()
        .add_plugin(HierarchyPlugin::default())
        // .add_system_to_stage(CoreStage::PostUpdate, render_menu_system)
        .add_system(taffy_follow_entity_model)
        .add_system_to_stage(CoreStage::PostUpdate, calculate_layout)
        .add_system_to_stage(CoreStage::Last, render_layout)
        .add_system_to_stage(
            CoreStage::Last,
            render_charmie::<MenuUi>.before(render_layout),
        )
        .add_system_to_stage(
            CoreStage::Last,
            render_charmie::<SimpleUi>.before(render_layout),
        )
        // .add_system(pause)
        .run()
}

fn setup_node_layout(mut taffy: NonSendMut<Taffy>, mut commands: Commands) {
    log::debug!("Hello whirled!");
    println!("Hello world");

    let deck_list_bundle = MenuUi {
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
            MenuUiItem {
                name: "Item G But |its LONG, too long and gets truncated".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "それは".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "それはほんとうにすごい！".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "Item J".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "Item K".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "Item L".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "Item M".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "Item N".to_string(),
                onselect: vec![],
            },
        ],
        selected_option: Some(5),
        scroll_offset: 3,
    }
    .bundle(
        &mut taffy,
        taffy::prelude::Style {
            min_size: taffy::prelude::Size {
                width: taffy::prelude::Dimension::Points(11.0),
                height: taffy::prelude::Dimension::Points(4.0),
            },
            margin: taffy::geometry::Rect {
                bottom: Dimension::Points(1.0),
                end: Dimension::Points(1.0),
                start: Dimension::Points(1.0),
                top: Dimension::Points(1.0),
            },
            flex_grow: 1.0,
            ..Default::default()
        },
    );

    let curio_desc_bundle = SimpleUi {
        draw: vec![
            "----|----|----|----|".to_string(),
            "Hullo Hey".to_string(),
            "hulloguvnaiamheretokillyou".to_string(),
        ],
    }
    .bundle(
        &mut taffy,
        taffy::prelude::Style {
            min_size: taffy::prelude::Size {
                width: taffy::prelude::Dimension::Points(10.0),
                height: taffy::prelude::Dimension::Points(4.0),
            },
            margin: taffy::geometry::Rect {
                bottom: Dimension::Points(1.0),
                end: Dimension::Points(1.0),
                start: Dimension::Points(1.0),
                top: Dimension::Points(1.0),
            },
            ..Default::default()
        },
    );

    let grid_map_bundle = SimpleUi {
        draw: vec!["GRID".to_string(), "GRID2".to_string()],
    }
    .bundle(
        &mut taffy,
        taffy::prelude::Style {
            min_size: taffy::prelude::Size {
                width: taffy::prelude::Dimension::Points(10.0),
                height: taffy::prelude::Dimension::Points(10.0),
            },
            flex_grow: 5.0,
            ..Default::default()
        },
    );

    // NodeLayoutView V
    // * Title bar
    // * main screen H
    //   * Sidebar V (Might use "====" lines instead of margins)
    //     * Sprite label (Curio/AccessPoint/Pickup)
    //     * Deck List
    //     * Sprite Desc
    //     * Action List (No margin)
    //   * GridMap
    // * messages

    //
    // sidebar
    //

    // let deck_list = commands.spawn().insert_bundle(deck_list_bundle).id();
    // let curio_desc = commands.spawn().insert_bundle(curio_desc_bundle).id();

    commands
        .spawn()
        .insert(DynamicTextLayout {
            bounds: Bounds(30, 30), // TODO Make sure this is coupled with taffy style
        })
        .insert(TaffyNodeComponent::new(
            &mut taffy,
            taffy::prelude::Style {
                size: taffy::prelude::Size {
                    width: taffy::prelude::Dimension::Points(30.0),
                    height: taffy::prelude::Dimension::Points(30.0),
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            // parent.spawn().insert_bundle(deck_list_bundle); // tmp
            // parent.spawn().insert_bundle(curio_desc_bundle); // tmp
            parent
                .spawn()
                .insert(TaffyNodeComponent::new(
                    &mut taffy,
                    taffy::style::Style {
                        flex_direction: taffy::style::FlexDirection::Column,
                        flex_grow: 1.0,
                        ..Default::default()
                    },
                ))
                .insert(Name::new("Sidebar"))
                .with_children(|parent| {
                    parent
                        .spawn()
                        .insert(Name::new("Decklist"))
                        .insert_bundle(deck_list_bundle);
                    parent
                        .spawn()
                        .insert(Name::new("Curio Desc"))
                        .insert_bundle(curio_desc_bundle);
                });
            parent.spawn().insert_bundle(grid_map_bundle);
        });

    // commands.spawn().insert_bundle(curio_desc_bundle);
}

fn pause() {
    crossterm::event::read().unwrap();
}

fn taffy_follow_entity_model(
    mut taffy: NonSendMut<Taffy>,
    nodes: Query<&TaffyNodeComponent>,
    new_child_nodes: Query<(&TaffyNodeComponent, &Parent), Added<TaffyNodeComponent>>,
) {
    for (child, parent_entity_id) in new_child_nodes.iter() {
        if let Ok(parent_node) = nodes.get(parent_entity_id.0).map(|p| p.node) {
            log::debug!("Adding child to parent");
            taffy.add_child(parent_node, child.node).unwrap();
        }
    }
}

// TODO In the future, use "Changed<DynamicTextLayouT>" filter https://bevy-cheatbook.github.io/programming/change-detection.html
fn calculate_layout(
    mut taffy: NonSendMut<Taffy>,
    text_layouts: Query<(&DynamicTextLayout, &TaffyNodeComponent)>,
) {
    log::debug!("Calculating layout ");
    for (text_layout, root) in text_layouts.iter() {
        log::debug!("Inner calculating layout");
        if (*taffy).dirty(root.node).unwrap_or(false) {
            taffy
                .compute_layout(
                    root.node,
                    taffy::prelude::Size {
                        width: taffy::prelude::Number::Defined(text_layout.bounds.width() as f32),
                        height: taffy::prelude::Number::Defined(text_layout.bounds.height() as f32),
                    },
                )
                .unwrap();
            log::debug!("Layout of root {:?}", taffy.layout(root.node).unwrap());
        } else {
            log::debug!(
                "Layout of root (nondirty) {:?}",
                taffy.layout(root.node).unwrap()
            );
        }
    }
}

fn render_charmie<T: CharmieRenderingComponent>(
    taffy: NonSend<Taffy>,
    draw_config: Res<crate::DrawConfiguration>,
    mut menu_uis: Query<(&T, &TaffyNodeComponent, &mut TextRendering)>,
    // views: Query<&DynamicTextLayout>,
) {
    // let draw_config = crate::DrawConfiguration::default();
    for (crc, node, mut text_rendering) in menu_uis.iter_mut() {
        let (size, pt) = node.bounds_and_pt(&taffy).unwrap();
        crc.render(&size, &draw_config, &mut *text_rendering);
        log::debug!("Rendering charmie!");
        text_rendering.location = pt;
        text_rendering.size = size;

        log::debug!("Charmie rendering: {:?}", text_rendering);
        for line in text_rendering.draw.iter() {
            println!("{}", line);
        }
    }
}

fn render_layout(
    taffy: NonSend<Taffy>,
    dynamic_text_layouts: Query<(&DynamicTextLayout, &TaffyNodeComponent, &Children)>,
    nodes: Query<(
        &TaffyNodeComponent,
        Option<&Children>,
        Option<&TextRendering>,
    )>,
) {
    for (dynamic_text_layout, root_node_comp, children_entities) in dynamic_text_layouts.iter() {
        // TODO some logic that skips non-displayed text_layouts
        let mut rendering =
            vec![BTreeMap::<usize, (usize, String)>::new(); dynamic_text_layout.bounds.height()];
        let mut descendents: VecDeque<((usize, usize), &Entity)> = std::iter::repeat((0, 0))
            .zip(children_entities.into_iter())
            .collect();
        while let Some((parent_pt, entity)) = descendents.pop_back() {
            if let Ok((node, children_opt, text_rendering_opt)) = nodes.get(*entity) {
                log::debug!("Parent pt [{:?}]", parent_pt);
                if let Some(children) = children_opt {
                    // Recursive heirarchy would break stuff.
                    let taffy::geometry::Point { x: x_add, y: y_add } =
                        taffy.layout(**node).unwrap().location;
                    let child_pt = (
                        parent_pt.0 + x_add.ceil() as usize,
                        parent_pt.1 + y_add.ceil() as usize,
                    );
                    descendents.extend(children.into_iter().map(|child| (child_pt, child)));
                }

                if let Some(text_rendering) = text_rendering_opt {
                    // TODO Figure out which direction Y is ine
                    let pt = (
                        text_rendering.location.0 + parent_pt.0,
                        text_rendering.location.1 + parent_pt.1,
                    );
                    log::debug!("Parent pt [{:?}] -> pt [{:?}]", parent_pt, pt);
                    for (idx, line) in (pt.1..).zip(text_rendering.draw.iter()) {
                        rendering[idx].insert(pt.0, (text_rendering.size.width(), line.clone()));
                    }
                }
            }
        }
        log::debug!("Lines...");
        for charmie_string_map in rendering.iter() {
            log::debug!("> {:?}", charmie_string_map);
        }

        let lines: Vec<_> = rendering
            .iter()
            .map(|string_map| {
                let (line_width, mut render_line) = string_map.iter().fold(
                    (0usize, String::new()),
                    |(length, full_line), (x, (width, segment))| {
                        if *x < length {
                            log::error!("Currently overlapping boxes is not supported");
                            (length, full_line)
                        } else {
                            let border: String = std::iter::repeat('\\').take(x - length).collect();
                            (
                                x + width,
                                format!(
                                    "{full_line}{border}{segment}",
                                    full_line = full_line,
                                    border = border,
                                    segment = segment
                                ),
                            )
                        }
                    },
                );
                let edge_border: String = std::iter::repeat('\\')
                    .take(dynamic_text_layout.bounds.width() - line_width)
                    .collect();
                render_line.push_str(&edge_border);
                render_line
            })
            .collect();
        log::debug!("Rendering!");
        log::debug!(
            "   *{}*",
            ('0'..='9')
                .cycle()
                .take(dynamic_text_layout.bounds.width())
                .collect::<String>()
        );
        for (n, line) in lines.iter().enumerate() {
            log::debug!("{0:2} |{1}|", n, line)
        }
        log::debug!("Rendering complete");

        // First step: Get all Nodes and their associated TextRendering from children of text layout.

        // Reduce text renderings into a collection of Vec<Vec<(x, width, String)>>
        // Iterator through it, then fold such that any difference between width and x is made up with "\" characters" for now.
        //
        // Get layout of nodes to get X/Y, and get TextRendering for entities.
        // Create raster of strings, as well as an array of usize to track length.
        // Sort by X
        //
        //
    }
}
