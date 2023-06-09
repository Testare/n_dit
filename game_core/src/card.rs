use std::num::NonZeroU32;

use crate::prelude::*;

#[derive(Component, Default, FromReflect, Reflect)]
pub struct Deck {
    cards: HashMap<Entity, NonZeroU32>,
}

impl Deck {
    const ONE: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(1) };

    /// Adds a card to the inventory, including another copy if it is already in the deck
    fn add_card(&mut self, card: Entity) -> &mut Self {
        self.cards
            .entry(card)
            .and_modify(|count| { 
                *count=count.saturating_add(1);
            })
            .or_insert(Self::ONE);
        self
    }

    /// Removes a card from the deck. If there are multiple copies,
    /// only removes one copy. Returns false if the card is not in the deck.
    fn remove_card(&mut self, card: Entity) -> bool {
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

#[derive(Component, FromReflect, Reflect)]
pub struct Card;

#[derive(Component, Deref, FromReflect, Reflect)]
struct Tags {
    tags: Vec<Tag>,
}

#[derive(FromReflect, Reflect)]
enum Tag {
    Fire,
    Flying,
}

mod action {
    use bevy::prelude::*;

    #[derive(Component, FromReflect, Reflect)]
    struct Actions {
        actions: Vec<Entity>, // Entities or just a list of them directly?
    }

    #[derive(Component, FromReflect, Reflect)]
    struct Action {}
}
