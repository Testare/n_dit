use getset::{CopyGetters, Getters, Setters};
use serde::{Deserialize, Serialize};

use crate::{Metadata, Sprite};
use crate::assets::{AssetDictionary, CardDef};
use std::fmt;

#[derive(Clone, Debug, Default, Setters, Getters, CopyGetters, Serialize, Deserialize)]
pub struct Inventory {
    bag: Vec<Item>,
    deck: Vec<Card>,
    #[get_copy = "pub"]
    wallet: usize,
    #[serde(default, skip)]
    #[set = "pub(crate)"]
    card_dict: AssetDictionary<CardDef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Pickup {
    Mon(usize),
    Item(Item),
    Card(String),
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
                let last_card = self.deck.pop().expect("Undo is dropping a card, but there are no cards!");
                assert_eq!(last_card.name(), card, "Undo is dropping some other card than the last card");
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
            Pickup::Card(card_name) => {
                // Will obviously need more complex logic if we have "stackable" cards
                self.deck.push(Card::named(&card_name, &self.card_dict));
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
            Pickup::Card(card) => write!(f, "\"{}\" card", card),
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
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if="Metadata::is_empty")]
    pub metadata: Metadata,
    #[serde(flatten)]
    pub base_card: CardDef,
}

impl Card {

    fn named(name: &str, card_dict: &AssetDictionary<CardDef>) -> Card {
        Card {
            nickname: None,
            metadata: Default::default(),
            base_card: (*card_dict[name]).clone(),
        }
    }

    fn name(&self) -> &str {
        self.base_card.name.as_str()
    }

}
