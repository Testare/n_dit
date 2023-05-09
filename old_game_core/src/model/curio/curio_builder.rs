use crate::{Curio, Team, Metadata};

#[derive(Debug, PartialEq, Eq)]
pub struct CurioBuilder {
    actions: Vec<String>,
    display: Option<String>,
    max_size: Option<usize>,
    speed: Option<usize>,
    metadata: Option<Metadata>,
    name: Option<String>,
    team: Option<Team>,
}

impl CurioBuilder {
    pub fn action(&mut self, action: &str) -> &mut Self {
        self.actions.push(action.to_string());
        self
    }

    pub fn actions(&mut self, actions: &Vec<String>) -> &mut Self {
        self.actions.extend(actions.clone());
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

    pub fn speed(&mut self, speed: usize) -> &mut Self {
        self.speed = Some(speed);
        self
    }

    pub fn metadata(&mut self, metadata: Metadata) -> &mut Self {
        self.metadata = Some(metadata);
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
        CurioBuilder {
            display: None,
            max_size: None,
            speed: None,
            metadata: None,
            name: None,
            team: None,
            actions: Vec::new(),
        }
    }

    pub fn build(&self) -> Option<Curio> {
        Some(Curio {
            display: self.display.as_ref()?.clone(),
            max_size: self.max_size?,
            speed: self.speed?,
            metadata: self.metadata.clone().unwrap_or_default(),
            name: self.name.clone().unwrap_or_else(|| "George".to_string()),
            team: self.team.unwrap_or(Team::EnemyTeam),
            actions: self.actions.clone(),
            tapped: false,
            moves_taken: 0,
        })
    }
}

impl Default for CurioBuilder {
    fn default() -> Self {
        CurioBuilder::new()
    }
}
