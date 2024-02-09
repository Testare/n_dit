use charmi::{CharacterMapImage, CharmieString};
use crossterm::style::Stylize;
use game_core::card::{Card, Deck};
use game_core::node::{AccessPoint, NodeOp, PlayedCards};
use game_core::op::CoreOps;
use game_core::player::{ForPlayer, Player};
use game_core::NDitCoreSet;
use taffy::style::Dimension;

use crate::base_ui::context_menu::{ContextAction, ContextActions};
use crate::base_ui::{HoverPoint, Tooltip};
use crate::configuration::DrawConfiguration;
use crate::input_event::{MouseButton, MouseEventListener, MouseEventTty, MouseEventTtyKind};
use crate::key_map::NamedInput;
use crate::layout::{CalculatedSizeTty, StyleTty, UiFocus, VisibilityTty};
use crate::node_ui::node_context_actions::NodeContextActions;
use crate::node_ui::node_ui_op::{FocusTarget, UiOps};
use crate::node_ui::{NodeUi, NodeUiOp, NodeUiQItem, SelectedAction, SelectedNodePiece};
use crate::prelude::*;
use crate::render::{RenderTtySet, TerminalRendering, RENDER_TTY_SCHEDULE};
use crate::{KeyMap, Submap};

#[derive(Default)]
pub struct MenuUiCardSelectionPlugin;

impl Plugin for MenuUiCardSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                MenuUiCardSelection::kb_card_selection.in_set(NDitCoreSet::ProcessInputs),
                MenuUiCardSelection::handle_layout_events.in_set(NDitCoreSet::ProcessInputs),
            ),
        )
        .add_systems(
            RENDER_TTY_SCHEDULE,
            (
                MenuUiCardSelection::style_card_selection.in_set(RenderTtySet::AdjustLayoutStyle),
                MenuUiCardSelection::card_selection_focus_status_change
                    .in_set(RenderTtySet::PreCalculateLayout),
                MenuUiCardSelection::render_system.in_set(RenderTtySet::PostCalculateLayout),
            ),
        )
        .add_systems(
            Update,
            (
                sys_create_load_context_items, // TODO consider scheduling for this
                sys_card_selection_adjust_ca_on_hover, // Should probably be done after rendering
            )
                .chain(),
        );
    }
}

#[derive(Component, Debug, Default)]
pub struct MenuUiCardSelection {
    scroll: usize,
}

// Perhaps these subcomponents should be part of MenuUiCardSelection?
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct SelectedItem(Option<usize>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct IsPadded(bool);

#[derive(Component, Debug, Deref)]
pub struct LoadCardContextAction(Entity);

impl MenuUiCardSelection {
    pub fn handle_layout_events(
        mut evr_mouse: EventReader<MouseEventTty>,
        mut res_ui_ops: ResMut<UiOps>,
        mut ui: Query<(
            &mut Self,
            &CalculatedSizeTty,
            &ForPlayer,
            &mut SelectedItem,
            &IsPadded,
        )>,
        mut players: Query<(&Deck, &mut SelectedAction), With<Player>>,
    ) {
        // Mostly handles scrolling now
        for layout_event in evr_mouse.read() {
            if let Ok((mut card_selection, size, ForPlayer(player), mut selected_item, is_padded)) =
                ui.get_mut(layout_event.entity())
            {
                if let Ok((deck, mut selected_action)) = players.get_mut(*player) {
                    let max_scroll = (deck.different_cards_len() + 1).saturating_sub(size.height());
                    match layout_event.event_kind() {
                        MouseEventTtyKind::ScrollDown => {
                            card_selection.scroll = (card_selection.scroll + 1).min(max_scroll);
                        },
                        MouseEventTtyKind::ScrollUp => {
                            card_selection.scroll = card_selection.scroll.saturating_sub(1);
                        },
                        MouseEventTtyKind::Down(MouseButton::Left) => {
                            if !layout_event.is_top_entity() {
                                continue;
                            }
                            res_ui_ops
                                .request(*player, NodeUiOp::ChangeFocus(FocusTarget::CardMenu));
                            let height = size.height32();

                            let padding: u32 = is_padded.0.into();
                            let UVec2 { x, y } = layout_event.relative_pos();
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
                                // TODO Should this be a NodeUiOp?
                                **selected_action = None;
                                **selected_item = Some(card_selection.scroll + y as usize - 1);
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
            (Entity, &UiFocus, &Deck, &SelectedNodePiece),
            (Changed<UiFocus>, With<Player>),
        >,
        mut card_selection_menus: IndexedQuery<
            ForPlayer,
            (Entity, &MenuUiCardSelection, &mut SelectedItem),
        >,
        access_points: Query<&AccessPoint>,
    ) {
        for (player, ui_focus, deck, selected_entity) in players.iter() {
            if let Ok((id, menu_ui_card_selection, mut selected_item)) =
                card_selection_menus.get_for_mut(player)
            {
                if **ui_focus == Some(id) {
                    if selected_item.is_none() {
                        **selected_item = selected_entity
                            .of(&access_points)
                            .and_then(|ap| deck.index_of_card(ap.card()?))
                            .or(Some(menu_ui_card_selection.scroll));
                    }
                } else if selected_item.is_some() {
                    **selected_item = None;
                }
            }
        }
    }

    pub fn kb_card_selection(
        mut card_menus: Query<(&mut Self, &ForPlayer, &mut SelectedItem)>,
        mut res_core_ops: ResMut<CoreOps>,
        players: Query<
            (
                Entity,
                &KeyMap,
                &Deck,
                &SelectedNodePiece,
                &UiFocus,
                &PlayedCards,
            ),
            With<Player>,
        >,
        access_points: Query<&AccessPoint>,
        mut ev_keys: EventReader<KeyEvent>,
    ) {
        for KeyEvent { code, modifiers } in ev_keys.read() {
            for (player, key_map, deck, selected_entity, focus_opt, played_cards) in players.iter()
            {
                focus_opt.and_then(|focused_ui| {
                    let (card_selection_menu, for_player, mut selected_item) =
                        card_menus.get_mut(focused_ui).ok()?;
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
                                .unwrap_or(card_selection_menu.scroll);
                            let next_pt = match dir {
                                Compass::North => current_point.saturating_sub(1),
                                Compass::South => {
                                    (current_point + 1).min(deck.different_cards_len() - 1)
                                },
                                _ => current_point,
                            };
                            **selected_item = Some(next_pt);
                        },
                        NamedInput::Activate | NamedInput::AltActivate => {
                            selected_entity.and_then(|access_point_id| {
                                let card_id = deck.cards_with_count().nth((**selected_item)?)?.0;
                                let access_point = get_assert!(access_point_id, &access_points)?;

                                if access_point.card() == Some(card_id) {
                                    if named_input != NamedInput::AltActivate {
                                        res_core_ops.request(
                                            player,
                                            NodeOp::UnloadAccessPoint { access_point_id },
                                        );
                                    }
                                } else if played_cards.can_be_played(deck, card_id) {
                                    res_core_ops.request(
                                        player,
                                        NodeOp::LoadAccessPoint {
                                            access_point_id,
                                            card_id,
                                        },
                                    );
                                }
                                Some(())
                            });
                        },
                        NamedInput::Undo => {
                            selected_entity.and_then(|access_point_id| {
                                let access_point = get_assert!(access_point_id, &access_points)?;
                                if access_point.card().is_some() {
                                    res_core_ops.request(
                                        player,
                                        NodeOp::UnloadAccessPoint { access_point_id },
                                    );
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
        player_info: Query<(&Deck, &SelectedNodePiece), With<Player>>,
        mut ui: Query<(&mut StyleTty, &ForPlayer, AsDerefMut<VisibilityTty>), With<Self>>,
    ) {
        for (mut style, ForPlayer(player), mut is_visible) in ui.iter_mut() {
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
                is_visible.set_if_neq(max_height != 0.0);
            }
        }
    }

    /// System for rendering a simple submenu
    fn render_system(
        res_draw_config: Res<DrawConfiguration>,
        access_points: Query<Ref<AccessPoint>>,
        cards: Query<&Card>,
        players: Query<(&Deck, &SelectedNodePiece, &PlayedCards, &UiFocus), With<Player>>,
        mut ui: Query<(
            Entity,
            &mut Self,
            &mut IsPadded,
            AsDerefCopied<VisibilityTty>,
            Ref<CalculatedSizeTty>,
            &ForPlayer,
            Ref<SelectedItem>,
            AsDeref<HoverPoint>,
            &mut TerminalRendering,
        )>,
    ) {
        for (
            id,
            mut card_selection,
            mut is_padded,
            is_visible,
            size,
            ForPlayer(player),
            selected_item,
            hover_point,
            mut tr,
        ) in ui.iter_mut()
        {
            if !is_visible {
                continue;
            }
            let mouse_hover_index = hover_point
                .as_ref()
                .filter(|pt| pt.x > 0)
                .and_then(|pt| (card_selection.scroll + pt.y as usize).checked_sub(1));
            let mut rendering = players
                .get(*player)
                .ok()
                .and_then(|(player_deck, selected_entity, played_cards, focus)| {
                    let access_point = selected_entity.of(&access_points)?;

                    let cards: Vec<CharmieString> = player_deck
                        .cards_with_count()
                        .enumerate()
                        .map(|(num, (id, _))| {
                            let remaining_count = played_cards.remaining_count(player_deck, id);
                            let is_selected = Some(id) == access_point.card();
                            let is_hover = **selected_item == Some(num);
                            let is_mouse_hover = mouse_hover_index == Some(num);
                            let name = cards
                                .get(id)
                                .map(|card| card.short_name_or_nickname())
                                .unwrap_or("NotACard");
                            let mut row = CharmieString::new();
                            if is_hover {
                                row.add_styled_text("▷".green());
                            } else if is_selected {
                                row.add_plain_text("▶");
                            }

                            let style = is_mouse_hover
                                .then(|| res_draw_config.color_scheme().menu_hover())
                                .unwrap_or_default();

                            row.add_text(name, &style)
                                .fit_to_len(size.width32() - 4)
                                .add_plain_text(" ")
                                .add_plain_text(remaining_count.to_string());
                            row
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
                        CharmieString::of_plain_text(if no_scroll_bar_needed {
                            " "
                        } else if i <= 1 {
                            "↑"
                        } else if i >= height - 3 {
                            "↓"
                        } else {
                            "│"
                        })
                    });

                    let mut cards_menu = CharacterMapImage::new();
                    let title_style = if Some(id) == **focus {
                        res_draw_config.color_scheme().menu_title_hover()
                    } else {
                        res_draw_config.color_scheme().menu_title()
                    };
                    let title_bar = CharmieString::of_text(
                        format!("{0:═<1$}", "═Cards", size.width()).as_str(),
                        &title_style,
                    );
                    cards_menu.push_row(title_bar);
                    for (scroll_bar, card) in scroll_bar.zip(
                        cards
                            .into_iter()
                            .skip(card_selection.scroll)
                            .take(size.height() - 1 - padding),
                    ) {
                        let mut row = scroll_bar;
                        row += card;
                        cards_menu.push_row(row);
                        // cards_menu.push(format!("{}{}", scroll_bar, card));
                    }
                    Some(cards_menu)
                })
                .unwrap_or_default();
            rendering.fit_to_size(size.width32(), size.height32());
            tr.update_charmie(rendering);
        }
    }
}

impl NodeUi for MenuUiCardSelection {
    const NAME: &'static str = "Menu Card Selection";
    type UiBundleExtras = (
        MouseEventListener,
        HoverPoint,
        SelectedItem,
        IsPadded,
        Tooltip,
        VisibilityTty,
    );
    type UiPlugin = MenuUiCardSelectionPlugin;

    fn initial_style(_: &NodeUiQItem) -> StyleTty {
        use taffy::prelude::*;

        StyleTty(Style {
            display: Display::Flex,
            min_size: Size {
                width: Dimension::Auto,
                height: Dimension::Points(0.0),
            },
            flex_grow: 1.0,
            ..default()
        })
    }

    // TODO refactor to accept player_id and add ContextActions here
    fn ui_bundle_extras() -> Self::UiBundleExtras {
        (
            MouseEventListener,
            HoverPoint::default(),
            SelectedItem::default(),
            IsPadded::default(),
            Tooltip::new("Select card to play"),
            VisibilityTty(false),
        )
    }
}

pub fn sys_create_load_context_items(
    mut commands: Commands,
    q_players_with_csm: Query<
        (AsDerefCopied<ForPlayer>, Ref<MenuUiCardSelection>),
        With<MenuUiCardSelection>,
    >,
    q_player_decks: Query<Ref<Deck>>,
    q_load_card_ca: Query<(), (With<LoadCardContextAction>, With<Card>)>,
    q_card: Query<&Card>,
) {
    for (player, csm_ref) in q_players_with_csm.iter() {
        if let Ok(deck) = q_player_decks.get(player) {
            if !csm_ref.is_added() && !deck.is_changed() {
                continue;
            }
            for card_id in deck.cards_iter() {
                if !q_load_card_ca.contains(card_id) {
                    if let Ok(card) = q_card.get(card_id) {
                        let card_name = card.card_name();
                        let ca_name = format!("Load {card_name} to access point");
                        let load_card_ca = commands.spawn((
                            Name::new(ca_name.clone()),
                            ContextAction::new(ca_name, move |id, world: &mut World| {
                                if let Some(&ForPlayer(player_id)) = world.get(id) {
                                    if let Some(deck) = world.get::<Deck>(player_id) {
                                        let pos = deck.cards_iter().position(|id|id == card_id).expect("deck should contain card played");
                                        if let Some(mut selected_item) = world.get_mut::<SelectedItem>(id) {
                                            **selected_item = Some(pos);
                                        }
                                    }
                                    if let Some(&SelectedNodePiece(Some(access_point_id))) = world.get(player_id) {
                                        world.get_resource_mut::<CoreOps>()
                                            .expect("CoreOps should be initialized")
                                            .request(player_id, NodeOp::LoadAccessPoint { access_point_id, card_id});
                                    } else {
                                        log::warn!("Player [{player_id:?}] does not have a selected entity to load card into")
                                    }
                                } else {
                                    log::warn!("ContextAction performed for [{id:?}] does not hae ForPlayer component")
                                }
                            })
                        )).id();
                        commands
                            .entity(card_id)
                            .insert(LoadCardContextAction(load_card_ca))
                            .add_child(load_card_ca);
                    }
                }
            }
        }
    }
}

pub fn sys_card_selection_adjust_ca_on_hover(
    res_node_ca: Res<NodeContextActions>,
    mut q_csm_ui: Query<
        (
            &ForPlayer,
            AsDerefCopied<HoverPoint>,
            &MenuUiCardSelection,
            &mut ContextActions,
        ),
        Or<(Changed<HoverPoint>, Changed<MenuUiCardSelection>)>,
    >,
    q_player: Query<(&Deck, AsDerefCopied<SelectedNodePiece>), With<Player>>,
    q_access_point: Query<Ref<AccessPoint>>,
    q_load_card_ca: Query<AsDerefCopied<LoadCardContextAction>, With<Card>>,
) {
    for (&ForPlayer(for_player), hover_point, card_selection, mut context_actions) in
        q_csm_ui.iter_mut()
    {
        get_assert!(for_player, q_player, |(deck, selected_entity)| {
            // TODO Do not perform actions for scrolling
            // Must have a selected access point for these to make sense
            // Question: Should I make sure to unload the actions if this is not the case?
            let access_point = q_access_point.get(selected_entity?).ok()?;

            // No context actions while hovering over the scroll bar
            if hover_point.as_ref().filter(|hp| hp.x != 0).is_none() {
                *context_actions.actions_mut() = vec![];
                return None;
            }

            let load_ca_action = hover_point.as_ref().and_then(|hover_point| {
                let mouse_hover_index =
                    (hover_point.y as usize + card_selection.scroll).checked_sub(1)?; // If this is nothing, why bother updating it?

                let card_id = deck.cards_iter().nth(mouse_hover_index);
                if access_point.card() == card_id {
                    None
                } else {
                    // Expect: We should have been populated in `sys_create_load_context_items` prior to this.
                    get_assert!(card_id?, q_load_card_ca)
                }
            });

            let unload_ca_action = access_point
                .card()
                .map(|_| res_node_ca.unload_selected_access_point());

            *context_actions.actions_mut() =
                load_ca_action.into_iter().chain(unload_ca_action).collect();
            Some(())
        });
    }
}
