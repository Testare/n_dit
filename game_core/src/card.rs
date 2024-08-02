use std::borrow::Cow;
use std::num::NonZeroU32;

use crate::node::PreventNoOp;
use crate::player::Player;
use crate::prelude::*;
use crate::saving::{LoadData, LoadSchedule, SaveData, SaveSchedule};
use crate::NDitCoreSet;

mod card_action;
mod card_as_asset;

use bevy::ecs::query::QueryData;
use bevy::prelude::AssetApp;
pub use card_action::{
    key, Action, ActionEffect, ActionRange, ActionTarget, Actions, Prereqs, Prerequisite,
    RangeShape,
};
pub use card_as_asset::{CardDefinition, NO_OP_ACTION_ID};
use serde::{Deserialize, Serialize};

// TODO better key architecture
pub mod save_key {
    use typed_key::*;

    use super::*;
    pub const DECK: Key<Deck> = typed_key!("deck");
}

#[derive(Debug, Default)]
pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<CardDefinition>()
            .init_asset::<Action>()
            .register_type::<BaseName>()
            .register_type::<Card>()
            .register_type::<Deck>()
            .register_type::<Description>()
            .register_type::<MaximumSize>()
            .register_type::<MovementSpeed>()
            .register_type::<Nickname>()
            .register_type::<Tag>()
            .register_type::<Tags>()
            .register_type::<HashMap<Entity, NonZeroU32>>()
            .register_type::<NonZeroU32>()
            .register_type::<Vec<Entity>>()
            .init_asset_loader::<card_as_asset::CardAssetLoader>()
            .init_asset_loader::<card_as_asset::ActionAssetLoader>()
            .add_systems(
                Update,
                (sys_load_cards, sys_sort_decks)
                    .chain()
                    .in_set(NDitCoreSet::PostProcessCommands),
            )
            .add_systems(SaveSchedule, sys_save_deck)
            .add_systems(LoadSchedule, sys_load_deck);
    }
}

#[derive(Clone, Component, Debug, Deref, Reflect)]
#[reflect(Component)]
pub struct BaseName(String);

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Card;

#[derive(Bundle, Debug)]
pub struct CardBundle {
    actions: Actions,
    card: Card,
    description: Description,
    max_size: MaximumSize,
    movement_speed: MovementSpeed,
    base_name: BaseName,
}

impl CardBundle {
    fn from_def(card_def: &CardDefinition) -> Self {
        CardBundle {
            actions: Actions(card_def.actions().clone()),
            card: Card,
            description: Description::new(card_def.description()),
            max_size: MaximumSize(card_def.max_size()),
            movement_speed: MovementSpeed(card_def.movement_speed()),
            base_name: BaseName(card_def.id().to_owned()),
        }
    }
}

#[derive(Debug, QueryData)]
pub struct CardQuery {
    card: &'static Card,
    pub actions: AsDeref<Actions>,
    pub description: AsDeref<Description>,
    pub max_size: OrU32<AsDerefCopied<MaximumSize>, 0>,
    pub movement_speed: OrU32<AsDerefCopied<MovementSpeed>, 0>,
    pub base_name: AsDeref<BaseName>,
    pub nickname: Option<AsDeref<Nickname>>,
    // TODO Replace with "Has" when it implements Debug
    // https://github.com/bevyengine/bevy/pull/12722
    prevent_no_op: Option<&'static PreventNoOp>,
}

impl CardQueryItem<'_> {
    pub fn nickname_or_name(&self) -> &str {
        self.nickname.unwrap_or(self.base_name).as_str()
    }

    pub fn nickname_or_name_cow(&self) -> Cow<'static, str> {
        Cow::Owned(self.nickname.unwrap_or(self.base_name).clone())
    }

    pub fn prevent_no_op(&self) -> bool {
        self.prevent_no_op.is_some()
    }
}

// TODO MapEntities?
#[derive(Component, Debug, Default, Deserialize, Eq, PartialEq, Reflect, Serialize)]
#[reflect(Component, Deserialize, Serialize)]
pub struct Deck {
    cards: HashMap<Entity, NonZeroU32>,
    ordering: Vec<Entity>,
}

impl Deck {
    const ONE: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1) };
    const MAX_CARD_COUNT: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(9) };

    pub fn index_of_card(&self, entity: Entity) -> Option<usize> {
        // TODO use ordering when I actually use ordering logic here
        self.ordering
            .iter()
            .enumerate()
            .find(|(_, card)| **card == entity)
            .map(|(index, _)| index)
    }

    pub fn count_of_card(&self, entity: Entity) -> u32 {
        self.cards
            .get(&entity)
            .copied()
            .map(NonZeroU32::get)
            .unwrap_or_default()
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn different_cards_len(&self) -> usize {
        self.cards.len()
    }

    pub fn cards_with_count(&self) -> impl Iterator<Item = (Entity, NonZeroU32)> + '_ {
        self.ordering.iter().map(|card| (*card, self.cards[card]))
    }

    pub fn cards_iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.ordering.iter().copied()
    }

    /// Adds a card to the inventory, including another copy if it is already in the deck
    pub fn add_card(&mut self, card: Entity) -> &mut Self {
        self.cards
            .entry(card)
            .and_modify(|count| {
                if (*count).cmp(&Self::MAX_CARD_COUNT).is_lt() {
                    *count = count.saturating_add(1);
                }
            })
            .or_insert(Self::ONE);
        if !self.ordering.contains(&card) {
            self.ordering.push(card);
        }
        self
    }

    pub fn sort_by_key<F, K: Ord>(&mut self, f: F)
    where
        F: FnMut(&Entity) -> K,
    {
        self.ordering.sort_by_key(f);
    }

    /// Adds a card to the inventory, including another copy if it is already in the deck.
    pub fn with_card(mut self, card: Entity) -> Self {
        self.add_card(card);
        self
    }

    /// Removes a card from the deck. If there are multiple copies,
    /// only removes one copy. Returns false if the card is not in the deck.
    pub fn remove_card(&mut self, card: Entity) -> bool {
        if let Some(card_count) = self.cards.get_mut(&card) {
            if let Some(new_count) = NonZeroU32::new(card_count.get() - 1) {
                *card_count = new_count;
            } else {
                self.cards.remove(&card);
                let index = self
                    .ordering
                    .iter()
                    .enumerate()
                    .find(|(_, e)| **e == card)
                    .expect("If it is present in the hashmap, it should be present in the ordering")
                    .0;
                self.ordering.remove(index);
            }
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Component, Debug, Default, Deref, Reflect)]
#[reflect(Component)]
pub struct Description(String);

impl Description {
    pub fn new<S: Into<String>>(description: S) -> Self {
        Description(description.into())
    }
}

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct MaximumSize(pub u32);

#[derive(Clone, Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct MovementSpeed(pub u32);

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Nickname(String);

impl Nickname {
    pub fn new<S: ToString>(name: S) -> Self {
        Self(name.to_string())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Reflect, Serialize)]
pub enum Tag {
    Damage,
    Healing,
    Fire,
    Flying,
}

#[derive(Component, Default, Deref, Reflect)]
#[reflect(Component)]
struct Tags {
    tags: Vec<Tag>,
}

pub fn sys_sort_decks(cards: Query<CardQuery>, mut decks: Query<&mut Deck, Changed<Deck>>) {
    // TODO Make sure we sort this when cards are loaded
    for mut deck in decks.iter_mut() {
        // In the future, perhaps have a property of Deck configure sorting method
        deck.sort_by_key(|card_id| {
            cards
                .get(*card_id)
                .map(|card| Cow::Owned(card.nickname_or_name().into()))
                .unwrap_or(Cow::Borrowed("")) // Might not have loaded yet
        })
    }
}

pub fn sys_load_cards(
    ast_card_defs: Res<Assets<CardDefinition>>,
    mut commands: Commands,
    unloaded_cards: Query<(Entity, &Handle<CardDefinition>), Without<Card>>,
) {
    for (id, card_handle) in unloaded_cards.iter() {
        if let Some(card_def) = ast_card_defs.get(card_handle) {
            let mut card = commands.entity(id);
            card.insert(CardBundle::from_def(card_def));
            if card_def.prevent_no_op() {
                card.insert(PreventNoOp);
            }
        }
    }
}

pub fn sys_save_deck(res_save_data: Res<SaveData>, q_player: Query<&Deck, With<Player>>) {
    for deck in q_player.iter() {
        res_save_data
            .put(save_key::DECK, deck)
            .expect("Should be able to save!");
        let card_entities: Vec<Entity> = deck.cards_iter().collect();
        res_save_data.add_entities(&card_entities);
    }
}

pub fn sys_load_deck(
    res_load_data: ResMut<LoadData>,
    mut q_player: Query<&mut Deck, With<Player>>,
) {
    for mut deck in q_player.iter_mut() {
        if let Ok(Some(load_deck)) = res_load_data.get_optional(save_key::DECK) {
            deck.set_if_neq(load_deck);
        }
    }
}
