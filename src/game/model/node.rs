mod node_change;
mod with_sprite;

pub use node_change::NodeChange;

// TODO Use some abstraction for EnemyAi, so we don't depend on that
use super::super::ai::EnemyAi;
use super::inventory::Pickup;
use super::sprite::Sprite;
use crate::{Bounds, GridMap, Point, Team};
use log::debug;

use with_sprite::WithSprite;

type NodeConstructionError = String;

#[derive(Clone, Debug)]
pub struct Node {
    grid: GridMap<Piece>,
    name: String,
    active_sprite: Option<usize>,
    enemy_ai: EnemyAi,
    active_team: Team,
}

#[derive(Clone, Debug)]
pub enum Piece {
    AccessPoint,
    Program(Sprite),
    Pickup(Pickup),
}

impl Node {
    fn grid_mut(&mut self) -> &mut GridMap<Piece> {
        &mut self.grid
    }

    fn drop_active_sprite(&mut self) {
        self.active_sprite = None;
    }

    pub fn untapped_sprites_on_active_team(&self) -> usize {
        let enemy_sprites_remaining = self.sprite_keys_for_team(Team::EnemyTeam).len();
        if enemy_sprites_remaining == 0 {
            panic!("No enemies remain! You win!")
        }

        self.filtered_sprite_keys(|_, sprite| {
            sprite.team() == self.active_team() && !sprite.tapped()
        })
        .len()
    }

    pub fn active_sprite_key(&self) -> Option<usize> {
        self.active_sprite
    }

    pub fn deactivate_sprite(&mut self) {
        self.with_active_sprite_mut(|mut sprite| sprite.tap());
        self.active_sprite = None;
    }

    pub fn activate_sprite(&mut self, sprite_key: usize) -> bool {
        let can_activate = self
            .with_sprite(sprite_key, |sprite| {
                sprite.team() == self.active_team() && !sprite.tapped()
            })
            .unwrap_or(false);

        if can_activate {
            self.with_active_sprite_mut(|mut sprite|
                if sprite.moves_taken() != 0 {
                    debug!("Activating new sprite, old sprite had taken {:?} moves and so must be tapped", sprite.moves_taken());
                    sprite.tap();
                });
            self.active_sprite = Some(sprite_key);
        }
        can_activate
    }

    pub(crate) fn grid(&self) -> &GridMap<Piece> {
        &self.grid
    }

    pub fn add_sprite(
        &mut self,
        spr: Sprite,
        pt_vec: Vec<Point>,
    ) -> Result<usize, NodeConstructionError> {
        // Could possibly be optimized with GridMap::put_entries
        let mut pts = pt_vec.into_iter();
        let first_pt = pts.next().ok_or("Sprite needs at least one point!")?;
        let key = self
            .grid
            .put_item(first_pt, Piece::Program(spr))
            .ok_or_else::<NodeConstructionError, _>(|| {
                "Could not add sprite to initial location".into()
            })?;
        for pt in pts {
            if !self.grid.push_front(pt, key) {
                return Err(format!("Could not add sprite to location {:?}", pt));
            }
        }
        Ok(key)
    }

    pub fn add_piece(&mut self, pt: Point, piece: Piece) -> Option<usize> {
        self.grid.put_item(pt, piece)
    }

    pub fn add_money(&mut self, pt: Point, amount: usize) -> Option<usize> {
        self.grid.put_item(pt, Piece::Pickup(Pickup::Mon(amount)))
    }

    pub fn width(&self) -> usize {
        self.grid.width()
    }

    pub fn height(&self) -> usize {
        self.grid.height()
    }

    pub fn bounds(&self) -> Bounds {
        Bounds::of(self.grid.width(), self.grid.height())
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn piece(&self, piece_key: usize) -> Option<&Piece> {
        self.grid.item(piece_key)
    }

    pub fn piece_at(&self, pt: Point) -> Option<&Piece> {
        self.grid.item_at(pt)
    }

    pub fn piece_key_at(&self, pt: Point) -> Option<usize> {
        self.grid.item_key_at(pt)
    }

    pub fn piece_len(&self, piece_key: usize) -> usize {
        self.grid.len_of(piece_key)
    }

    pub fn active_team(&self) -> Team {
        self.active_team
    }

    pub fn change_active_team(&mut self) {
        let active_team = match self.active_team {
            Team::EnemyTeam => Team::PlayerTeam,
            Team::PlayerTeam => Team::EnemyTeam,
        };
        self.active_team = active_team;
        for sprite_key in self.sprite_keys_for_team(active_team) {
            self.with_sprite_mut(sprite_key, |mut sprite| sprite.untap());
        }
    }

    pub fn enemy_ai(&self) -> &EnemyAi {
        &self.enemy_ai
    }

    pub fn pieces(&self) -> Vec<&Piece> {
        self.grid().entries()
    }

    pub fn sprite_keys_for_team(&self, team: Team) -> Vec<usize> {
        self.filtered_sprite_keys(|_, sprite| sprite.team() == team)
    }

    // TODO Make specialized "get sprites for team" function, since that it the primary use case here
    pub fn filtered_sprite_keys<P: Fn(usize, WithSprite) -> bool>(
        &self,
        predicate: P,
    ) -> Vec<usize> {
        self.grid.filtered_keys(|key, _| {
            self.with_sprite(key, |sprite| predicate(key, sprite))
                .unwrap_or(false)
        })
    }
}

impl Piece {
    pub fn is_pickup(&self) -> bool {
        matches!(self, Piece::Pickup(_))
    }
}

impl From<GridMap<Piece>> for Node {
    fn from(grid: GridMap<Piece>) -> Self {
        Node {
            active_sprite: None,
            active_team: Team::PlayerTeam,
            enemy_ai: EnemyAi::Simple,
            grid,
            name: String::from("Node"),
        }
    }
}

impl From<(String, GridMap<Piece>)> for Node {
    fn from((name, grid): (String, GridMap<Piece>)) -> Self {
        Node {
            active_sprite: None,
            active_team: Team::PlayerTeam,
            enemy_ai: EnemyAi::Simple,
            grid,
            name,
        }
    }
}

impl From<(String, GridMap<Piece>, EnemyAi)> for Node {
    fn from((name, grid, enemy_ai): (String, GridMap<Piece>, EnemyAi)) -> Self {
        Node {
            active_sprite: None,
            active_team: Team::PlayerTeam,
            enemy_ai,
            grid,
            name,
        }
    }
}
