use crate::Piece;
use getset::{CopyGetters, Getters};
use std::{cmp::min, fmt};

const MAX_WALLET_SIZE: usize = 99999;

#[derive(Clone, Debug, Default, Getters, CopyGetters)]
pub struct Inventory {
    bag: Vec<Item>,
    deck: Vec<Card>,
    #[get_copy = "pub"]
    wallet: usize,
}

#[derive(Clone, Debug)]
pub enum Pickup {
    Mon(usize),
    Item(Item),
    Card(Card),
}

impl Inventory {
    pub fn max_wallet_size(&self) -> usize {
        MAX_WALLET_SIZE
    }

    pub fn pick_up(&mut self, pickup: Pickup) {
        match pickup {
            Pickup::Mon(mon) => {
                self.wallet = min(self.wallet + mon, self.max_wallet_size());
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

    pub fn to_piece(self) -> Piece {
        Piece::Pickup(self)
    }
}

impl From<Pickup> for Piece {
    fn from(pickup: Pickup) -> Piece {
        pickup.to_piece()
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

#[derive(Clone, Debug, Getters)]
/// Not sure if we'll make use of this much
pub struct Item {
    #[get = "pub"]
    name: String,
}

#[derive(Clone, Debug, Getters)]
/// A card that can be turned into playable sprite in a game
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

impl From<Card> for Piece {
    fn from(card: Card) -> Piece {
        Piece::from(Pickup::from(card))
    }
}
