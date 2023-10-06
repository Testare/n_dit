use std::num::NonZeroU32;

use crate::prelude::*;

mod card_action;
mod card_as_asset;

pub use card_action::{
    key, Action, ActionEffect, ActionRange, ActionTarget, Actions, Prereqs, Prerequisite,
    RangeShape,
};
pub use card_as_asset::{CardDefinition, NO_OP_ACTION_ID};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<CardDefinition>()
            .add_asset::<Action>()
            .register_type::<Card>()
            // .register_type::<Deck>() // Can't register because of NonZeroU32
            .add_asset_loader(card_as_asset::CardAssetLoader)
            .add_asset_loader(card_as_asset::ActionAssetLoader);
    }
}
#[derive(Component, Debug, Deserialize, Default, Reflect, Serialize)]
#[reflect(Component, Deserialize, Serialize)]
pub struct Deck {
    cards: HashMap<Entity, NonZeroU32>,
    ordering: Vec<Entity>,
}

#[derive(Component, Debug, Default, Reflect, getset::Getters)]
#[reflect(Component)]
pub struct Card {
    card_name: String,
    #[getset(get = "pub")]
    display_id: String,
    short_name: Option<String>,
    nickname: Option<String>,
    #[getset(get = "pub")]
    definition: Handle<CardDefinition>,
}

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
pub struct MovementSpeed(pub u32);

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
pub struct MaximumSize(pub u32);

#[derive(Clone, Component, Deref, Reflect)]
pub struct Description(String);

#[derive(Component, Deref, Reflect)]
struct Tags {
    tags: Vec<Tag>,
}

#[derive(Clone, Copy, Debug, Deserialize, Reflect, Serialize)]
pub enum Tag {
    Damage,
    Healing,
    Fire,
    Flying,
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

    pub fn cards_with_count<'a>(&'a self) -> impl Iterator<Item = (Entity, NonZeroU32)> + 'a {
        self.ordering.iter().map(|card| (*card, self.cards[card]))
    }

    pub fn cards_iter<'a>(&'a self) -> impl Iterator<Item = Entity> + 'a {
        self.ordering.iter().map(|e| *e)
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

impl Card {
    pub fn new<S: Into<String>>(
        card_name: S,
        display_id: S,
        short_name: Option<S>,
        definition: Handle<CardDefinition>,
    ) -> Self {
        Card {
            card_name: card_name.into(),
            display_id: display_id.into(),
            short_name: short_name.map(Into::into),
            nickname: None,
            definition,
        }
    }

    pub fn name_or_nickname(&self) -> &str {
        self.nickname.as_ref().unwrap_or(&self.card_name).as_str()
    }

    pub fn short_name_or_nickname(&self) -> &str {
        self.nickname
            .as_ref()
            .or(self.short_name.as_ref())
            .unwrap_or(&self.card_name)
            .as_str()
    }

    pub fn card_name(&self) -> &str {
        self.card_name.as_str()
    }
}

impl Description {
    pub fn new<S: Into<String>>(description: S) -> Self {
        Description(description.into())
    }
}

pub fn sys_sort_decks(cards: Query<&Card>, mut decks: Query<&mut Deck, Changed<Deck>>) {
    for mut deck in decks.iter_mut() {
        // In the future, perhaps have a property of Deck configure sorting method
        deck.sort_by_key(|card_id| {
            get_assert!(*card_id, &cards)
                .map(|card| card.short_name_or_nickname())
                .unwrap_or("")
        })
    }
}
