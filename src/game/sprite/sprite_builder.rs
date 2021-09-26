use crate::{Sprite, StandardSpriteAction, Team};

#[derive(Debug, PartialEq, Eq)]
pub struct SpriteBuilder {
    actions: Vec<StandardSpriteAction>,
    display: Option<String>,
    max_size: Option<usize>,
    movement_speed: Option<usize>,
    name: Option<String>,
    team: Option<Team>,
}

impl SpriteBuilder {
    pub fn action(&mut self, action: StandardSpriteAction) -> &mut Self {
        self.actions.push(action);
        self
    }

    pub fn display<S: ToString>(&mut self, display: S) -> &mut Self {
        // Validations here that display is 2 characters wide?
        self.display = Some(display.to_string());
        self
    }

    pub fn max_size(&mut self, max_size: usize) -> &mut Self {
        self.max_size = Some(max_size);
        self
    }

    pub fn movement_speed(&mut self, movement_speed: usize) -> &mut Self {
        self.movement_speed = Some(movement_speed);
        self
    }

    pub fn name<S: ToString>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn team(&mut self, team: Team) -> &mut Self {
        self.team = Some(team);
        self
    }

    pub fn new() -> Self {
        SpriteBuilder {
            display: None,
            max_size: None,
            movement_speed: None,
            name: None,
            team: None,
            actions: Vec::new(),
        }
    }

    pub fn build(&self) -> Option<Sprite> {
        Some(Sprite {
            display: self.display.as_ref()?.clone(),
            max_size: self.max_size?,
            movement_speed: self.movement_speed?,
            name: self.name.clone().unwrap_or("George".to_string()),
            team: self.team.unwrap_or(Team::EnemyTeam),
            actions: self.actions.clone(),
            tapped: false,
            moves_taken: 0,
        })
    }
}
