mod curio_builder;

use getset::{CopyGetters, Getters, Setters};
use serde::{Deserialize, Serialize};

use curio_builder::CurioBuilder;
use super::inventory::Card;
use crate::{
    assets::AssetDictionary,
    Metadata,
    assets::CardDef,
    error::{ErrorMsg, Result},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Team {
    PlayerTeam = 0,
    EnemyTeam = 1,
}

#[derive(Clone, Debug, PartialEq, Eq, Getters, CopyGetters, Setters, Serialize, Deserialize)]
pub struct Curio {
    display: String,
    #[get_copy = "pub"]
    #[set = "pub"]
    max_size: usize,
    #[get = "pub"]
    metadata: Metadata,
    #[get_copy = "pub"]
    #[set]
    speed: usize,
    #[get_copy = "pub"]
    moves_taken: usize,
    name: String,
    #[get_copy = "pub"]
    team: Team,
    tapped: bool,
    #[get = "pub"]
    actions: Vec<String>, // Vec<Metadata>
}

impl Curio {
    pub fn builder() -> CurioBuilder {
        CurioBuilder::new()
    }

    fn from_card(card: &Card, team: Team, card_dictionary: &AssetDictionary<CardDef>) -> Result<Curio> {
        let card_def = &card_dictionary[card.basis.as_str()];

        let name = card.nickname.as_ref().unwrap_or(&card_def.name).clone();
        Ok(Curio {
                display: card_def.display.clone(),
                max_size: card_def.max_size,
                speed: card_def.speed,
                metadata: card.metadata.clone(),
                name,
                team,
                actions: card_def.actions.clone(),
                tapped: false,
                moves_taken: 0,
        })
    }

    pub fn new(display: &str) -> Curio {
        Curio {
            display: String::from(display),
            max_size: 3,
            speed: 3,
            moves_taken: 0,
            name: String::from("George"),
            team: Team::PlayerTeam,
            metadata: Metadata::default(),
            tapped: false,
            actions: vec![
                "Brutus".to_string(),
                "Bite".to_string(),
                "Fiddle".to_string(),
            ],
        }
    }

    pub fn display(&self) -> &str {
        self.display.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn moves(&self) -> usize {
        self.speed - self.moves_taken
    }

    pub fn tap(&mut self) {
        self.tapped = true;
    }

    pub fn untap(&mut self) {
        self.tapped = false;
        self.moves_taken = 0;
    }

    pub fn tapped(&self) -> bool {
        self.tapped
    }

    pub fn untapped(&self) -> bool {
        !self.tapped
    }

    pub fn took_a_move(&mut self) {
        self.moves_taken += 1;
        if self.actions.is_empty() && self.moves_taken == self.speed {
            self.tap()
        }
    }
}

impl Team {
    pub fn is_ai(&self) -> bool {
        matches!(self, Team::EnemyTeam)
    }
}

