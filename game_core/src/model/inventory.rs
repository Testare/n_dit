use getset::{CopyGetters, Getters, Setters, MutGetters};
use serde::{Deserialize, Serialize};

use crate::{Metadata, Sprite, error::Result};
use std::{fmt, collections::HashMap};

#[derive(Clone, Debug, Default, Setters, MutGetters, Getters, CopyGetters, Serialize, Deserialize)]
pub struct Inventory {
    bag: Vec<Item>,
    #[get = "pub"]
    #[get_mut = "pub"]
    deck: Deck,
    #[get_copy = "pub"]
    wallet: usize
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Pickup {
    Mon(usize),
    Item(Item),
    Card(String),
}

impl Inventory {

    pub fn card_id(&self, name: &str) -> Option<CardId> {
        self.deck.card_id(name)
    }

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
                self.deck.drop_card(card).expect("Undo failed, card was never picked up!");
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
                self.deck.add_card(&card_name);
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
    #[get = "pub"]
    pub basis: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Deck(HashMap<String, (usize, Option<MarkedCard>)>);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct MarkedCard(String, Metadata);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CardId {
    Nickname(String),
    Cardname(String),
}

impl CardId {
    fn card_name(&self) -> &str {
        match self {
            CardId::Nickname(s) => s.as_str(),
            CardId::Cardname(s) => s.as_str(),
        }
    }
}

impl Deck {

    pub fn play_card(&mut self, _card_id: &CardId) -> Result<()> {
        Ok(())
    }

    pub fn return_card(&mut self, _card_id: &CardId) -> Result<()> {
        Ok(())
    }

    fn drop_card(&mut self, card_name: &str) -> Option<()> {
        let entry = self.0.entry(card_name.to_string()).or_default();
        if entry.0 == 0 {
            None
        } else {
            entry.0 -= 1;
            Some(())
        }
    }

    fn add_card(&mut self, card_name: &str) {
        self.0.entry(card_name.to_string()).or_default().0 += 1;
    }

    fn mark_card(&mut self, nickname: &str, card_name: &str) -> bool {
        let card_count = self.0.get(card_name).map(|card|card.0).unwrap_or(0);
        let marked_card_exists = self.0.get(nickname).and_then(|card|card.1.as_ref()).is_some();
        if card_count == 0 || marked_card_exists {
            false
        } else {
            let card_name_string = card_name.to_string();
            self.0.entry(card_name_string.clone()).or_default().0 -= 1;
            self.0.entry(nickname.to_string()).or_default().1 = Some(MarkedCard(card_name_string, Metadata::new()));
            true
        }
    }

    fn mark_up_card<M: FnMut(&mut Metadata)>(&mut self, nickname: &str, mut mark_up: M) -> Option<()> {
        Some(mark_up(&mut self.0.get_mut(nickname)?.1.as_mut()?.1))
    }

    /**
     * Looks for a card with the specified nickname, or the specified basis name
     * (base card name) if no card has that nickname. If multiple cards match, 
     * the first one in deck order is returned
     */
    pub fn card_id(&self, name: &str) -> Option<CardId> {
        let deck_card = self.0.get(name)?;
        if let Some(marked_card) = &self.0.get(name)?.1 {
            Some(CardId::Nickname(name.to_string()))
        } else if deck_card.0 > 0 {
            Some(CardId::Cardname(name.to_string()))
        } else {
            None
        }
    }

    /**
     * Looks for a card with the specified nickname, or the specified basis name
     * (base card name) if no card has that nickname. If multiple cards match, 
     * the first one in deck order is returned
     */
    pub fn card_by_name(&self, name: &str) -> Option<Card> {
        let deck_card = self.0.get(name)?;
        if let Some(marked_card) = &deck_card.1 {
            Some(Card {
                nickname: Some(name.to_string()),
                metadata: marked_card.1.clone(),
                basis: marked_card.0.clone(),
            })
        } else if deck_card.0 > 0 {
            Some(Card {
                nickname: None,
                metadata: Default::default(),
                basis: name.to_string(),
            })
        } else {
            None
        }
    }

    pub fn card_by_id(&self, id: &CardId) -> Option<Card> {
        match id {
            CardId::Cardname(card_name) => {
                let deck_card = self.0.get(card_name)?;
                if deck_card.0 > 0 {
                    Some(Card {
                        nickname: None,
                        metadata: Default::default(),
                        basis: card_name.to_string(),
                    })
                } else {
                    None
                }
            }
            CardId::Nickname(nickname) => {
                let deck_card = self.0.get(nickname)?.1.as_ref()?;
                Some(Card {
                    nickname: Some(nickname.to_string()),
                    metadata: deck_card.1.clone(),
                    basis: deck_card.0.clone(),
                })

            }
        }

    }

}
