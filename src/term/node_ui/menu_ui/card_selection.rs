use crossterm::event::{MouseButton, MouseEventKind};
use game_core::card::{Card, Deck};
use game_core::node::{AccessPoint, NodeOp, NodePiece};
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
use pad::PadStr;
use taffy::style::Dimension;

use crate::term::layout::{CalculatedSizeTty, FitToSize, LayoutEvent, StyleTty};
use crate::term::node_ui::{NodeUi, SelectedAction, SelectedEntity};
use crate::term::prelude::*;
use crate::term::render::{RenderTtySet, UpdateRendering};

#[derive(Component, Debug, Default)]
pub struct MenuUiCardSelection {
    scroll: usize,
}

#[derive(Default)]
pub struct MenuUiCardSelectionPlugin;

impl MenuUiCardSelection {
    pub fn handle_layout_events(
        mut layout_events: EventReader<LayoutEvent>,
        mut ui: Query<(&mut Self, &CalculatedSizeTty, &ForPlayer)>,
        cards: Query<&Card>,
        mut players: Query<(&Deck, &SelectedEntity, &mut SelectedAction), With<Player>>,
        access_points: Query<&AccessPoint, With<NodePiece>>,
        mut node_command: EventWriter<Op<NodeOp>>,
    ) {
        for layout_event in layout_events.iter() {
            if let Ok((mut card_selection, size, ForPlayer(player))) =
                ui.get_mut(layout_event.entity())
            {
                if let Ok((deck, selected_entity, mut selected_action)) = players.get_mut(*player) {
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
                            } else if y > 0 && y < height - 1 {
                                let index = card_selection.scroll + y as usize - 1;
                                if let Some((card_id, count)) = deck.cards_with_count().nth(index) {
                                    log::debug!(
                                        "Clicked on card: {:?} \"{}\", which we have {} of.",
                                        card_id,
                                        cards.get(card_id).unwrap().name_or_nickname(),
                                        count
                                    );
                                    // Selected

                                    let access_point_id = selected_entity.0.unwrap();
                                    let access_point = selected_entity.of(&access_points).unwrap();

                                    **selected_action = None;

                                    if access_point.card() == Some(card_id) {
                                        node_command.send(Op::new(
                                            *player,
                                            NodeOp::UnloadAccessPoint { access_point_id },
                                        ));
                                    } else {
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
        access_points: Query<&AccessPoint>,
        mut commands: Commands,
        cards: Query<&Card>,
        players: Query<(&Deck, &SelectedEntity), With<Player>>,
        mut ui: Query<(Entity, &mut Self, &CalculatedSizeTty, &ForPlayer)>,
    ) {
        for (id, mut card_selection, size, ForPlayer(player)) in ui.iter_mut() {
            let rendering = players
                .get(*player)
                .ok()
                .and_then(|(player_deck, selected_entity)| {
                    let access_point = selected_entity.of(&access_points)?;

                    let cards: Vec<String> = player_deck
                        .cards_with_count()
                        .map(|(id, count)| {
                            let is_selected = Some(id) == access_point.card();
                            let name = cards
                                .get(id)
                                .map(|card| card.short_name_or_nickname())
                                .unwrap_or("NotACard");
                            let width = size.width()
                                - 3
                                - count.ilog10() as usize
                                - if is_selected { 1 } else { 0 };
                            format!(
                                "{selection_indicator}{name} {count}",
                                name = name.with_exact_width(width),
                                count = count,
                                selection_indicator = if is_selected { "▶" } else { "" },
                            )
                        })
                        .collect();

                    let height = size.height();
                    card_selection.scroll = card_selection
                        .scroll
                        .min((player_deck.different_cards_len() + 1).saturating_sub(height));
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

                    let padding = if height > (cards.len() + 1) { 1 } else { 0 };

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
                .update_rendering(rendering.fit_to_size(size));
        }
    }
}

impl Plugin for MenuUiCardSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            MenuUiCardSelection::handle_layout_events.in_set(NDitCoreSet::ProcessInputs),
            MenuUiCardSelection::style_card_selection.in_set(RenderTtySet::PreCalculateLayout),
            MenuUiCardSelection::render_system.in_set(RenderTtySet::PostCalculateLayout),
        ));
    }
}

impl NodeUi for MenuUiCardSelection {
    type UiBundle = ();
    type UiPlugin = MenuUiCardSelectionPlugin;

    fn ui_bundle() -> Self::UiBundle {
        ()
    }
}
