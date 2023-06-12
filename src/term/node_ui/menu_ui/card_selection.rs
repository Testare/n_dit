use bevy::app::SystemAppConfig;
use crossterm::event::{MouseButton, MouseEventKind};
use game_core::card::{Card, Deck};
use game_core::player::PlayerN;
use game_core::{AccessPoint, NodeAct, NodePiece};
use pad::PadStr;
use taffy::style::Dimension;

use super::{selected_piece_data, NodePieceQ, NodeUi};
use crate::term::layout::{CalculatedSizeTty, FitToSize, LayoutEvent, StyleTty};
use crate::term::node_ui::{NodeFocus, NodeUiDataParam, NodeUiQ};
use crate::term::prelude::*;
use crate::term::render::UpdateRendering;

#[derive(Component, Debug, Default)]
pub struct MenuUiCardSelection<const P: usize> {
    scroll: usize,
}

impl<const P: usize> MenuUiCardSelection<P> {
    pub fn handle_layout_events(
        mut layout_events: EventReader<LayoutEvent>,
        mut ui: Query<(&mut Self, &CalculatedSizeTty)>,
        cards: Query<&Card>,
        deck: Query<&Deck, With<PlayerN<P>>>,
        node_data: NodeUiDataParam,
        access_points: Query<&AccessPoint, With<NodePiece>>,
        mut node_command: EventWriter<Act<NodeAct>>,
    ) {
        if let Ok(deck) = deck.get_single() {
            for layout_event in layout_events.iter() {
                if let Ok((mut card_selection, size)) = ui.get_mut(layout_event.entity()) {
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
                            if layout_event.pos().x == 0 && max_scroll != 0 {
                                // Click on scroll bar
                                match layout_event.pos().y {
                                    1 | 2 => {
                                        card_selection.scroll =
                                            card_selection.scroll.saturating_sub(1);
                                    },
                                    x if x == height - 1 || x == height - 2 => {
                                        card_selection.scroll =
                                            (card_selection.scroll + 1).min(max_scroll);
                                    },
                                    _ => {},
                                }
                            } else if layout_event.pos().y > 0 && layout_event.pos().y < height - 1
                            {
                                let index =
                                    card_selection.scroll + layout_event.pos().y as usize - 1;
                                if let Some((card_id, count)) = deck.cards_with_count().nth(index) {
                                    log::debug!(
                                        "Clicked on card: {:?} \"{}\", which we have {} of.",
                                        card_id,
                                        cards.get(card_id).unwrap().name_or_nickname(),
                                        count
                                    );
                                    let access_point_id =
                                        node_data.node_data().unwrap().selected_entity.unwrap();
                                    let access_point = access_points.get(access_point_id).unwrap();

                                    if access_point.card() == Some(card_id) {
                                        node_command.send(Act::new::<P>(
                                            NodeAct::UnloadAccessPoint { access_point_id },
                                        ));
                                    } else {
                                        node_command.send(Act::new::<P>(
                                            NodeAct::LoadAccessPoint {
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
        node_data: Query<NodeUiQ>,
        node_focus: Res<NodeFocus>,
        node_pieces: Query<NodePieceQ>,
        player_deck: Query<&Deck, With<PlayerN<P>>>,
        mut ui: Query<&mut StyleTty, With<Self>>,
    ) {
        if let Ok(mut style) = ui.get_single_mut() {
            let (min_height, max_height) =
                selected_piece_data(&node_data, node_focus, &node_pieces)
                    .and_then(|selected| {
                        selected.access_point.and_then(|_| {
                            let deck = player_deck.get_single().ok()?;
                            let full_len = deck.different_cards_len() as f32 + 2.0;

                            Some((full_len.min(6.0), full_len))
                        })
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
        node_data: Query<NodeUiQ>,
        node_focus: Res<NodeFocus>,
        node_pieces: Query<NodePieceQ>,
        mut commands: Commands,
        cards: Query<&Card>,
        player_deck: Query<&Deck, With<PlayerN<P>>>,
        mut ui: Query<(Entity, &mut Self, &CalculatedSizeTty)>,
    ) {
        if let Ok((id, mut card_selection, size)) = ui.get_single_mut() {
            let rendering = node_focus
                .and_then(|node_id| {
                    let node_data = node_data.get(node_id).ok()?;
                    let selected = node_pieces.get((**node_data.selected_entity)?).ok()?;
                    let access_point = selected.access_point?;

                    let player_deck = player_deck.get_single().ok()?;
                    // TODO ensure order, enable scrolling, possibly cache this card list, handle non-cards in deck
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

                    let mut cards_menu = vec![format!("{0:═<1$}", "═Cards", size.width())];
                    for (scroll_bar, card) in scroll_bar.zip(
                        cards
                            .into_iter()
                            .skip(card_selection.scroll)
                            .take(size.height() - 2),
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

impl<const P: usize> NodeUi for MenuUiCardSelection<P> {
    fn style_update_system() -> Option<SystemAppConfig> {
        Some(Self::style_card_selection.into_app_config())
    }

    fn render_system() -> SystemAppConfig {
        Self::render_system.into_app_config()
    }
}
