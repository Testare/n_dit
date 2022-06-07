use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::Sprite;
use std::fmt;

#[derive(Clone, Debug, Default, Getters, CopyGetters, Serialize, Deserialize)]
pub struct Inventory {
    bag: Vec<Item>,
    deck: Vec<Card>,
    #[get_copy = "pub"]
    wallet: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Pickup {
    Mon(usize),
    Item(Item),
    Card(Card),
}

impl Inventory {
    pub fn remove(&mut self, pickup: &Pickup) {
        match pickup {
            Pickup::Mon(mon) => {
                self.wallet -= mon;
            }
            Pickup::Item(item) => {
                // Will obviously need more complex logic if we have "stackable" items
                self.bag.retain(|iter_item| item != iter_item);
            }
            Pickup::Card(card) => {
                // Will obviously need more complex logic if we have "stackable" cards
                self.deck.retain(|iter_card| card != iter_card);
            }
        }
    }

    pub fn pick_up(&mut self, pickup: Pickup) {
        match pickup {
            Pickup::Mon(mon) => {
                self.wallet += mon;
            }
            Pickup::Item(item) => {
                // Will obviously need more complex logic if we have "stackable" items
                self.bag.push(item);
            }
            Pickup::Card(card) => {
                // Will obviously need more complex logic if we have "stackable" cards
                self.deck.push(card);
            }
        }
    }
}

// TODO Should this live here or with Node?
impl Pickup {
    // TODO move this render-specific logic, configurable symbols
    const MON_SQUARE: &'static str = "$$";
    const ITEM_SQUARE: &'static str = "++";
    // const CARD_SQUARE: &'static str = "🃁 ";
    // const CARD_SQUARE: &'static str = "♠♥";
    // const CARD_SQUARE: &'static str = "==";
    // const CARD_SQUARE: &'static str = "++";
    // const CARD_SQUARE: &'static str = "&]";
    // const CARD_SQUARE: &'static str = "□]";
    const CARD_SQUARE: &'static str = "🂠 ";

    pub fn square_display(&self) -> &'static str {
        match self {
            Pickup::Mon(..) => Pickup::MON_SQUARE,
            Pickup::Item(..) => Pickup::ITEM_SQUARE,
            Pickup::Card(..) => Pickup::CARD_SQUARE,
        }
    }

    pub fn to_sprite(self) -> Sprite {
        Sprite::Pickup(self)
    }
}

impl From<Pickup> for Sprite {
    fn from(pickup: Pickup) -> Sprite {
        pickup.to_sprite()
    }
}

impl fmt::Display for Pickup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pickup::Mon(mon) => write!(f, "${}", mon),
            Pickup::Item(item) => write!(f, "{}", item.name()),
            Pickup::Card(card) => write!(f, "\"{}\" card", card.name()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Getters, Serialize, Deserialize)]
/// Not sure if we'll make use of this much
pub struct Item {
    #[get = "pub"]
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Getters, Serialize, Deserialize)]
/// A card that can be turned into playable curio in a game
/// Might want to have separate "type" and "name" fields, so cards
/// can have their own unique names
// TODO Implement actual logic
pub struct Card {
    #[get = "pub"]
    pub name: String,
}

impl From<Card> for Pickup {
    fn from(card: Card) -> Pickup {
        Pickup::Card(card)
    }
}

impl From<Card> for Sprite {
    fn from(card: Card) -> Sprite {
        Sprite::from(Pickup::from(card))
    }
}
