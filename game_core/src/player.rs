use crate::prelude::*;

#[derive(Component, Debug, )]
pub struct Player(usize);

impl Player {
    fn new(player_num: usize) -> Self {
        Player(player_num)
    }

    fn num(&self) -> usize {
        self.0
    }
}