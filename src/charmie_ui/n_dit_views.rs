use super::dynamic_layout::{
    CharmieRenderingComponent, DynamicTextLayout, MenuUi, MenuUiItem, SimpleUi, TaffyNodeComponent,
};
use bevy::prelude::*;
use old_game_core::{Bounds, Node};
use taffy::{node::Taffy, prelude::Dimension, style::Style as TaffyStyle};

#[derive(Component)]
struct NodeView {
    deck_list: Entity,
    curio_desc: Entity,
    action_list: Entity,
    action_desc: Entity,
    /*grid_map: Entity,
    menu_bar: Entity,
    message_bar: Entity,*/
}

pub fn setup_node_view(
    mut taffy: NonSendMut<Taffy>, /*, deck: Deck*/
    node: &old_game_core::Node,
    commands: &mut Commands,
) {
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

    let action_list_bundle = MenuUi {
        options: vec![
            MenuUiItem {
                name: "No Action".to_string(),
                onselect: vec![],
            },
            MenuUiItem {
                name: "Slice".to_string(),
                onselect: vec![],
            },
        ],
        scroll_offset: 0,
        selected_option: None,
    }
    .bundle(
        &mut taffy,
        TaffyStyle {
            flex_grow: 1.0,

            ..Default::default()
        },
    );

    let action_desc_bundle = SimpleUi {
        draw: vec!["Does a thing".to_string()],
    }
    .bundle(
        &mut taffy,
        TaffyStyle {
            flex_grow: 1.0,
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
    //     * Aciton Desc
    //   * GridMap
    // * messages

    // BUG: Ordering is not really respected, as the system that adds child entities to taffy doesn't notice order.
    //
    // sidebar
    //

    // let deck_list = commands.spawn().insert_bundle(deck_list_bundle).id();
    // let curio_desc = commands.spawn().insert_bundle(curio_desc_bundle).id();

    let mut deck_menu: Option<Entity> = None;
    let mut curio_desc: Option<Entity> = None;
    let mut action_menu: Option<Entity> = None;
    let mut action_desc: Option<Entity> = None;
    /*grid_map,
    menu_bar,
    message_bar,*/
    commands
        .spawn(())
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
            parent
                .spawn(())
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
                    deck_menu = Some(
                        parent
                            .spawn(())
                            .insert(Name::new("Decklist"))
                            .insert(deck_list_bundle)
                            .id(),
                    );
                    curio_desc = Some(
                        parent
                            .spawn(())
                            .insert(Name::new("Curio Description"))
                            .insert(curio_desc_bundle)
                            .id(),
                    );
                    action_menu = Some(
                        parent
                            .spawn(())
                            .insert(Name::new("Action Menu"))
                            .insert(action_list_bundle)
                            .id(),
                    );
                    action_desc = Some(
                        parent
                            .spawn(())
                            .insert(Name::new("Action Description"))
                            .insert(action_desc_bundle)
                            .id(),
                    );
                });
            parent.spawn(()).insert(grid_map_bundle);
        })
        .insert(NodeView {
            deck_list: deck_menu.unwrap(),
            curio_desc: curio_desc.unwrap(),
            action_list: action_menu.unwrap(),
            action_desc: action_desc.unwrap(),
        });

    // commands.spawn().insert_bundle(curio_desc_bundle);

    /*NodeView {
        deck_menu,
        curio_desc_node,
        action_menu,
        action_desc_node,
        grid_map,
        menu_bar,
        message_bar,
    }*/
}

#[derive()]
struct NodeCompenent(Node);
