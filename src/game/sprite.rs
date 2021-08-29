#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Team {
    PlayerTeam = 0,
    EnemyTeam = 1,
}

#[derive(PartialEq, Eq)]
pub struct Sprite {
    display: String,
    name: String,
    max_size: usize,
    moved: bool,
    team: Team,
    // actions
}

impl Sprite {
    pub fn new(display: &str) -> Sprite {
        Sprite {
            display: String::from(display),
            name: String::from("George"),
            max_size: 3,
            moved: false,
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
