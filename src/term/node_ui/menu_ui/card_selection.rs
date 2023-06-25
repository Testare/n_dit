use crossterm::event::{MouseButton, MouseEventKind};
use game_core::card::{Card, Deck};
use game_core::node::{AccessPoint, NodeOp, NodePiece, PlayedCards};
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
use pad::PadStr;
use taffy::style::Dimension;

use crate::term::key_map::NamedInput;
use crate::term::layout::{
    CalculatedSizeTty, FitToSize, LayoutEvent, LayoutMouseTarget, StyleTty, UiFocus, UiFocusOnClick,
};
use crate::term::node_ui::{NodeUi, NodeUiQItem, SelectedAction, SelectedEntity};
use crate::term::prelude::*;
use crate::term::render::{RenderTtySet, UpdateRendering};
use crate::term::{KeyMap, Submap};

#[derive(Component, Debug, Default)]
pub struct MenuUiCardSelection {
    scroll: usize,
}

// Perhaps these subcomponents should be part of MenuUiCardSelection?
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct SelectedItem(Option<usize>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct IsPadded(bool);

#[derive(Default)]
pub struct MenuUiCardSelectionPlugin;

impl MenuUiCardSelection {
    pub fn handle_layout_events(
        mut layout_events: EventReader<LayoutEvent>,
        mut ui: Query<(
            &mut Self,
            &CalculatedSizeTty,
            &ForPlayer,
            &mut SelectedItem,
            &IsPadded,
        )>,
        cards: Query<&Card>,
        mut players: Query<
            (&Deck, &SelectedEntity, &mut SelectedAction, &PlayedCards),
            With<Player>,
        >,
        access_points: Query<&AccessPoint, With<NodePiece>>,
        mut node_command: EventWriter<Op<NodeOp>>,
    ) {
        for layout_event in layout_events.iter() {
            if let Ok((mut card_selection, size, ForPlayer(player), mut selected_item, is_padded)) =
                ui.get_mut(layout_event.entity())
            {
                if let Ok((deck, selected_entity, mut selected_action, played_cards)) =
                    players.get_mut(*player)
                {
                    let max_scroll = (deck.different_cards_len() + 1).saturating_sub(size.height());
                    match layout_event.event_kind() {
                        MouseEventKind::ScrollDown => {
                            card_selection.scroll = (card_selection.scroll + 1).min(max_scroll);
                        },
                        MouseEventKind::ScrollUp => {
                            card_selection.scroll = card_selection.scroll.saturating_sub(1);
                        },
                        MouseEventKind::Down(MouseButton::Left) => {
                            let height = size.height32();
                            let padding: u32 = is_padded.0.into();
                            let UVec2 { x, y } = layout_event.pos();
                            if x == 0 && max_scroll != 0 {
                                // Click on scroll bar
                                match y {
                                    1 | 2 => {
                                        card_selection.scroll =
                                            card_selection.scroll.saturating_sub(1);
                                    },
                                    i if i == height - 1 || i == height - 2 => {
                                        card_selection.scroll =
                                            (card_selection.scroll + 1).min(max_scroll);
                                    },
                                    _ => {},
                                }
                            } else if y > 0 && y < height - padding {
                                let index = card_selection.scroll + y as usize - 1;
                                if let Some((card_id, count)) = deck.cards_with_count().nth(index) {
                                    log::debug!(
                                        "Clicked on card: {:?} \"{}\", which we have {}/{} of.",
                                        card_id,
                                        cards.get(card_id).unwrap().name_or_nickname(),
                                        played_cards.remaining_count(deck, card_id),
                                        count,
                                    );
                                    // Selected

                                    let access_point_id = selected_entity.0.unwrap();
                                    let access_point = selected_entity.of(&access_points).unwrap();

                                    **selected_action = None;
                                    **selected_item = Some(index);

                                    if access_point.card() == Some(card_id) {
                                        node_command.send(Op::new(
                                            *player,
                                            NodeOp::UnloadAccessPoint { access_point_id },
                                        ));
                                    } else if played_cards.can_be_played(deck, card_id) {
                                        node_command.send(Op::new(
                                            *player,
                                            NodeOp::LoadAccessPoint {
                                                access_point_id,
                                                card_id,
                                            },
                                        ));
                                    }
                                }
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
    }

    pub fn card_selection_focus_status_change(
        players: Query<
            (Entity, &UiFocus, &Deck, &SelectedEntity),
            (Changed<UiFocus>, With<Player>),
        >,
        mut card_selection_menus: Query<(&MenuUiCardSelection, &mut SelectedItem, &ForPlayer)>,
        access_points: Query<&AccessPoint>,
    ) {
        for (player, ui_focus, deck, selected_entity) in players.iter() {
            if let Some((menu_ui_card_selection, mut selected_item, _)) =
                ui_focus.and_then(|ui_focus| card_selection_menus.get_mut(ui_focus).ok())
            {
                // If the menu_ui is focused, but no selected_item created, create a default
                if selected_item.is_none() {
                    **selected_item = selected_entity
                        .of(&access_points)
                        .and_then(|ap| deck.index_of_card(ap.card()?))
                        .or(Some(menu_ui_card_selection.scroll));
                }
            } else {
                for (_, mut selected_item, for_player) in card_selection_menus.iter_mut() {
                    if for_player.0 == player {
                        // If the ui focus changes from card_selection, clear selected_item
                        if selected_item.is_some() {
                            **selected_item = None;
                        }
                        break;
                    }
                }
            }
        }
    }

    pub fn card_selection_keyboard_controls(
        mut uis: Query<(&mut Self, &ForPlayer, &mut SelectedItem)>,
        players: Query<
            (
                Entity,
                &KeyMap,
                &Deck,
                &SelectedEntity,
                &UiFocus,
                &PlayedCards,
            ),
            With<Player>,
        >,
        access_points: Query<&AccessPoint>,
        mut ev_keys: EventReader<KeyEvent>,
        mut ev_node_op: EventWriter<Op<NodeOp>>,
    ) {
        for KeyEvent { code, modifiers } in ev_keys.iter() {
            for (
                player,
                key_map,
                deck,
                selected_entity,
                focus_opt,
                played_cards,
            ) in players.iter()
            {
                focus_opt.and_then(|focused_ui| {
                    let (card_selection_menu, for_player, mut selected_item) =
                        uis.get_mut(focused_ui).ok()?;
                    if for_player.0 != player {
                        return None;
                    }
                    let named_input =
                        key_map.named_input_for_key(Submap::Node, *code, *modifiers)?;
                    match named_input {
                        NamedInput::Direction(dir) => {
                            let current_point = selected_item
                                .or_else(|| {
                                    let selected_card =
                                        get_assert!((**selected_entity)?, &access_points)?
                                            .card()?;
                                    Some(
                                        deck.cards_with_count()
                                            .enumerate()
                                            .find(|(_, (card_entity, _))| {
                                                *card_entity == selected_card
                                            })?
                                            .0,
                                    )
                                })
                                .unwrap_or_else(|| card_selection_menu.scroll);
                            let next_pt = match dir {
                                Compass::North => current_point.saturating_sub(1),
                                Compass::South => {
                                    (current_point + 1).min(deck.different_cards_len() - 1)
                                },
                                _ => current_point,
                            };
                            **selected_item = Some(next_pt);
                        },
                        NamedInput::Activate => {
                            // TODO assert_and_then
                            selected_entity.and_then(|access_point_id| {
                                let card_id = deck.cards_with_count().nth((**selected_item)?)?.0;
                                let access_point = get_assert!(access_point_id, &access_points)?;

                                if access_point.card() == Some(card_id) {
                                    ev_node_op.send(Op::new(
                                        player,
                                        NodeOp::UnloadAccessPoint { access_point_id },
                                    ));
                                } else if played_cards.can_be_played(deck, card_id) {
                                    ev_node_op.send(Op::new(
                                        player,
                                        NodeOp::LoadAccessPoint {
                                            access_point_id,
                                            card_id,
                                        },
                                    ));
                                }
                                Some(())
                            });
                        },
                        _ => {},
                    }
                    Some(())
                });
            }
        }
    }

    fn style_card_selection(
        access_points: Query<(), With<AccessPoint>>,
        player_info: Query<(&Deck, &SelectedEntity), With<Player>>,
        mut ui: Query<(&mut StyleTty, &ForPlayer), With<Self>>,
    ) {
        for (mut style, ForPlayer(player)) in ui.iter_mut() {
            let (min_height, max_height) = player_info
                .get(*player)
                .ok()
                .and_then(|(deck, selected_entity)| {
                    selected_entity.of(&access_points)?;
                    let full_len = deck.different_cards_len() as f32 + 2.0;
                    Some((full_len.min(6.0), full_len))
                })
                .unwrap_or((0.0, 0.0));
            if Dimension::Points(max_height) != style.max_size.height {
                style.max_size.height = Dimension::Points(max_height);
                style.min_size.height = Dimension::Points(min_height);
                style.display = if max_height == 0.0 {
                    taffy::style::Display::None
                } else {
                    taffy::style::Display::Flex
                };
            }
        }
    }

    /// System for rendering a simple submenu
    fn render_system(
        access_points: Query<Ref<AccessPoint>>,
        mut commands: Commands,
        cards: Query<&Card>,
        players: Query<(&Deck, &SelectedEntity, &PlayedCards), With<Player>>,
        mut ui: Query<(
            Entity,
            &mut Self,
            &mut IsPadded,
            Ref<CalculatedSizeTty>,
            &ForPlayer,
            Ref<SelectedItem>,
        )>,
    ) {
        for (id, mut card_selection, mut is_padded, size, ForPlayer(player), selected_item) in
            ui.iter_mut()
        {
            let rendering = players
                .get(*player)
                .ok()
                .and_then(|(player_deck, selected_entity, played_cards)| {
                    let access_point = selected_entity.of(&access_points)?;

                    let cards: Vec<String> = player_deck
                        .cards_with_count()
                        .enumerate()
                        .map(|(num, (id, _))| {
                            let remaining_count = played_cards.remaining_count(player_deck, id);
                            let is_selected = Some(id) == access_point.card();
                            let is_hover = **selected_item == Some(num);
                            let name = cards
                                .get(id)
                                .map(|card| card.short_name_or_nickname())
                                .unwrap_or("NotACard");
                            let width =
                                size.width() - 4 - if is_selected || is_hover { 1 } else { 0 };
                            format!(
                                "{selection_indicator}{name} {count}",
                                name = name.with_exact_width(width),
                                count = remaining_count,
                                selection_indicator = if is_hover {
                                    "▷"
                                } else if is_selected {
                                    "▶"
                                } else {
                                    ""
                                },
                            )
                        })
                        .collect();

                    let height = size.height();
                    **is_padded = height > cards.len() + 1; // Might change this logic sometime
                    let padding: usize = is_padded.0.into();
                    card_selection.scroll = if let (true, Some(index)) = (
                        selected_item.is_changed()
                            || access_point.is_changed()
                            || size.is_changed(), // Ideally we could remove this if we can get scheduling figured out
                        **selected_item,
                    ) {
                        // Focus on selected item
                        card_selection
                            .scroll
                            .min(index)
                            .max((index + 2 + padding).saturating_sub(height))
                    } else {
                        card_selection.scroll
                    }
                    .min((player_deck.different_cards_len() + 1 + padding).saturating_sub(height));
                    let no_scroll_bar_needed = height > cards.len();
                    let scroll_bar = (0..height).map(|i| {
                        if no_scroll_bar_needed {
                            " "
                        } else if i <= 1 {
                            "↑"
                        } else if i >= height - 3 {
                            "↓"
                        } else {
                            "|"
                        }
                    });

                    let mut cards_menu = vec![format!("{0:═<1$}", "═Cards", size.width())];
                    for (scroll_bar, card) in scroll_bar.zip(
                        cards
                            .into_iter()
                            .skip(card_selection.scroll)
                            .take(size.height() - 1 - padding),
                    ) {
                        cards_menu.push(format!("{}{}", scroll_bar, card));
                    }
                    Some(cards_menu)
                })
                .unwrap_or_default();
            commands
                .entity(id)
                .update_rendering(rendering.fit_to_size(&size));
        }
    }
}

impl Plugin for MenuUiCardSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            MenuUiCardSelection::card_selection_keyboard_controls
                .in_set(NDitCoreSet::ProcessInputs),
            MenuUiCardSelection::card_selection_focus_status_change
                .in_set(RenderTtySet::PreCalculateLayout),
            MenuUiCardSelection::handle_layout_events.in_set(NDitCoreSet::ProcessInputs),
            MenuUiCardSelection::style_card_selection.in_set(RenderTtySet::PreCalculateLayout),
            MenuUiCardSelection::render_system.in_set(RenderTtySet::PostCalculateLayout),
        ));
    }
}

impl NodeUi for MenuUiCardSelection {
    const NAME: &'static str = "Menu Card Selection";
    type UiBundleExtras = (LayoutMouseTarget, UiFocusOnClick, SelectedItem, IsPadded);
    type UiPlugin = MenuUiCardSelectionPlugin;

    fn initial_style(_: &NodeUiQItem) -> StyleTty {
        use taffy::prelude::*;

        StyleTty(Style {
            display: Display::None,
            min_size: Size {
                width: Dimension::Auto,
                height: Dimension::Points(0.0),
            },
            flex_grow: 1.0,
            ..default()
        })
    }

    fn ui_bundle_extras() -> Self::UiBundleExtras {
        (
            LayoutMouseTarget,
            UiFocusOnClick,
            SelectedItem::default(),
            IsPadded::default(),
        )
    }
}
