use std::num::NonZeroU32;

use crate::prelude::*;

#[derive(Component, Debug, Default, FromReflect, Reflect)]
pub struct Deck {
    cards: HashMap<Entity, NonZeroU32>,
    ordering: Vec<(String, Entity)>,
}

impl Deck {
    const ONE: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1) };

    pub fn new() -> Self {
        Default::default()
    }

    pub fn different_cards_len(&self) -> usize {
        self.cards.len()
    }

    pub fn cards_with_count<'a>(&'a self) -> impl Iterator<Item = (Entity, NonZeroU32)> + 'a {
        self.cards.iter().map(|(&e, &c)| (e, c))
    }

    /// Adds a card to the inventory, including another copy if it is already in the deck
    pub fn add_card(&mut self, card: Entity) -> &mut Self {
        self.cards
            .entry(card)
            .and_modify(|count| {
                *count = count.saturating_add(1);
            })
            .or_insert(Self::ONE);
        self
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
            }
            true
        } else {
            false
        }
    }
}

#[derive(Component, Debug, FromReflect, Reflect)]
pub struct Card {
    card_name: String,
    short_name: Option<String>,
    nickname: Option<String>,
}

impl Card {
    pub fn new<S: Into<String>>(card_name: S, short_name: Option<S>) -> Self {
        Card {
            card_name: card_name.into(),
            short_name: short_name.map(Into::into),
            nickname: None,
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

#[derive(Component, Deref, FromReflect, Reflect)]
struct Tags {
    tags: Vec<Tag>,
}

#[derive(FromReflect, Reflect)]
enum Tag {
    Damage,
    Healing,
    Fire,
    Flying,
}

#[derive(Clone, Debug, FromReflect, Reflect)]
pub enum ActionEffect {
    Damage(usize),
    Heal(usize),
}
