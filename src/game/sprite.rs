use super::sprite_action::{SpriteAction, StandardSpriteAction};
use getset::{CopyGetters, Getters};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Team {
    PlayerTeam = 0,
    EnemyTeam = 1,
}

#[derive(Debug, PartialEq, Eq, Getters, CopyGetters)]
pub struct Sprite {
    display: String,
    #[get_copy = "pub"]
    max_size: usize,
    movement_speed: usize,
    moves_taken: usize,
    name: String,
    #[get_copy = "pub"]
    team: Team,
    tapped: bool,
    #[get = "pub"]
    actions: Vec<StandardSpriteAction>, // Vec<Metadata>
}

impl Sprite {
    pub fn new(display: &str) -> Sprite {
        Sprite {
            display: String::from(display),
            max_size: 3,
            movement_speed: 3,
            moves_taken: 0,
            name: String::from("George"),
            team: Team::PlayerTeam,
            tapped: false,
            actions: vec![StandardSpriteAction::Brutus],
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
    }

    pub fn tapped(&self) -> bool {
        self.tapped
    }

    pub fn took_a_move(&mut self) {
        self.moves_taken += 1;
    }
}
