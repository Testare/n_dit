mod curio_builder;

use getset::{CopyGetters, Getters, Setters};
use serde::{Deserialize, Serialize};

use super::curio_action::StandardCurioAction;
use curio_builder::CurioBuilder;

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
    #[set]
    movement_speed: usize,
    #[get_copy = "pub"]
    moves_taken: usize,
    name: String,
    #[get_copy = "pub"]
    team: Team,
    tapped: bool,
    #[get = "pub"]
    actions: Vec<StandardCurioAction>, // Vec<Metadata>
}

impl Curio {
    pub fn builder() -> CurioBuilder {
        CurioBuilder::new()
    }

    pub fn new(display: &str) -> Curio {
        Curio {
            display: String::from(display),
            max_size: 3,
            movement_speed: 3,
            moves_taken: 0,
            name: String::from("George"),
            team: Team::PlayerTeam,
            tapped: false,
            actions: vec![
                StandardCurioAction::Brutus,
                StandardCurioAction::Bite,
                StandardCurioAction::Fiddle,
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
        self.movement_speed - self.moves_taken
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
        if self.actions.is_empty() && self.moves_taken == self.movement_speed {
            self.tap()
        }
    }
}

impl Team {
    pub fn is_ai(&self) -> bool {
        matches!(self, Team::EnemyTeam)
    }
}
