use getset::CopyGetters;
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Team {
    PlayerTeam = 0,
    EnemyTeam = 1,
}

#[derive(Debug, PartialEq, Eq, CopyGetters)]
pub struct Sprite {
    display: String,
    #[get_copy = "pub"]
    max_size: usize,
    movement_speed: usize,
    moves_taken: usize,
    name: String,
    team: Team,
    tapped: bool,
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
            tapped: false,
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
