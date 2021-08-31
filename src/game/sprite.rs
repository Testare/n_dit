#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Team {
    PlayerTeam = 0,
    EnemyTeam = 1,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sprite {
    display: String,
    max_size: usize,
    movement_speed: usize,
    moves_taken: usize,
    name: String,
    team: Team,
    // actions
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
        }
    }
    pub fn display(&self) -> &str {
        self.display.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn team(&self) -> Team {
        self.team
    }
}
