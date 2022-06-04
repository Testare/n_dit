mod node_change;
mod with_curio;

pub use node_change::NodeChange;
pub use node_change::SpritePoint; // TODO Move these to better location

use getset::Getters;
use serde::{Deserialize, Serialize};

// TODO Use some abstraction for EnemyAi, so we don't depend on that
use super::super::ai::EnemyAi;
use super::super::error::Result;
use super::curio::Curio;
use super::inventory::{Inventory, Pickup};
use super::keys::node_change_keys;
use crate::{Bounds, GridMap, Metadata, Point, Team};
use log::debug;

use with_curio::WithCurio;

type NodeConstructionError = String;

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct Node {
    grid: GridMap<Sprite>,
    name: String,
    active_curio: Option<usize>,
    enemy_ai: EnemyAi,
    active_team: Team,
    #[get = "pub"]
    inventory: Inventory,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Sprite {
    AccessPoint,
    Curio(Curio),
    Pickup(Pickup),
}

impl Node {
    /// ### SAFETY
    /// Unsafe to return sprites with new keys
    /// See grid_map return_item_with_key
    pub(super) unsafe fn return_sprite_with_key(
        &mut self,
        key: usize,
        pt: Point,
        sprite: Sprite,
    ) -> Option<usize> {
        self.grid_mut().return_item_with_key(key, pt, sprite)
    }

    fn grid_mut(&mut self) -> &mut GridMap<Sprite> {
        &mut self.grid
    }

    fn drop_active_curio(&mut self) {
        self.active_curio = None;
    }

    pub fn untapped_curios_on_active_team(&self) -> usize {
        let enemy_curios_remaining = self.curio_keys_for_team(Team::EnemyTeam).len();
        if enemy_curios_remaining == 0 {
            panic!("No enemies remain! You win!")
        }

        self.filtered_curio_keys(|_, curio| curio.team() == self.active_team() && !curio.tapped())
            .len()
    }

    pub fn active_curio_key(&self) -> Option<usize> {
        self.active_curio
    }

    pub fn deactivate_curio(&mut self) {
        self.with_active_curio_mut(|mut curio| curio.tap());
        self.active_curio = None;
    }

    fn set_active_curio(&mut self, curio_key: Option<usize>) {
        self.active_curio = curio_key
    }

    pub fn activate_curio(&mut self, curio_key: usize) -> bool {
        let can_activate = self
            .with_curio(curio_key, |curio| {
                curio.team() == self.active_team() && !curio.tapped()
            })
            .unwrap_or(false);

        if can_activate {
            self.with_active_curio_mut(|mut curio|
                if curio.moves_taken() != 0 {
                    debug!("Activating new curio, old curio had taken {:?} moves and so must be tapped", curio.moves_taken());
                    curio.tap();
                });
            self.active_curio = Some(curio_key);
        }
        can_activate
    }

    pub fn grid(&self) -> &GridMap<Sprite> {
        &self.grid
    }

    pub fn add_curio(
        &mut self,
        spr: Curio,
        pt_vec: Vec<Point>,
    ) -> std::result::Result<usize, NodeConstructionError> {
        // Could possibly be optimized with GridMap::put_entries
        let mut pts = pt_vec.into_iter();
        let first_pt = pts.next().ok_or("Curio needs at least one point!")?;
        let key = self
            .grid
            .put_item(first_pt, Sprite::Curio(spr))
            .ok_or_else::<NodeConstructionError, _>(|| {
                "Could not add curio to initial location".into()
            })?;
        for pt in pts {
            if !self.grid.push_front(pt, key) {
                return Err(format!("Could not add curio to location {:?}", pt));
            }
        }
        Ok(key)
    }

    pub fn add_sprite(&mut self, pt: Point, sprite: Sprite) -> Option<usize> {
        self.grid.put_item(pt, sprite)
    }

    pub fn add_money(&mut self, pt: Point, amount: usize) -> Option<usize> {
        self.grid.put_item(pt, Sprite::Pickup(Pickup::Mon(amount)))
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

    pub fn sprite(&self, sprite_key: usize) -> Option<&Sprite> {
        self.grid.item(sprite_key)
    }

    pub fn sprite_at(&self, pt: Point) -> Option<&Sprite> {
        self.grid.item_at(pt)
    }

    pub fn sprite_key_at(&self, pt: Point) -> Option<usize> {
        self.grid.item_key_at(pt)
    }

    pub fn sprite_len(&self, sprite_key: usize) -> usize {
        self.grid.len_of(sprite_key)
    }

    pub fn active_team(&self) -> Team {
        self.active_team
    }

    fn set_active_team(&mut self, team: Team) {
        self.active_team = team;
    }

    pub fn change_active_team(&mut self) {
        let active_team = match self.active_team {
            Team::EnemyTeam => Team::PlayerTeam,
            Team::PlayerTeam => Team::EnemyTeam,
        };
        self.active_team = active_team;
        for curio_key in self.curio_keys_for_team(active_team) {
            self.with_curio_mut(curio_key, |mut curio| curio.untap());
        }
    }

    pub fn enemy_ai(&self) -> &EnemyAi {
        &self.enemy_ai
    }

    pub fn sprites(&self) -> Vec<&Sprite> {
        self.grid().entries()
    }

    pub fn curio_keys_for_team(&self, team: Team) -> Vec<usize> {
        self.filtered_curio_keys(|_, curio| curio.team() == team)
    }

    // TODO Make specialized "get curios for team" function, since that it the primary use case here
    pub fn filtered_curio_keys<P: Fn(usize, WithCurio) -> bool>(&self, predicate: P) -> Vec<usize> {
        self.grid.filtered_keys(|key, _| {
            self.with_curio(key, |curio| predicate(key, curio))
                .unwrap_or(false)
        })
    }

    fn default_metadata(&self) -> Result<Metadata> {
        let mut metadata = Metadata::new();
        metadata.put(node_change_keys::TEAM, &self.active_team())?;
        metadata.put_optional(node_change_keys::PERFORMING_CURIO, self.active_curio_key())?;
        Ok(metadata)
    }
}

impl Sprite {
    pub fn is_pickup(&self) -> bool {
        matches!(self, Sprite::Pickup(_))
    }
}

impl From<GridMap<Sprite>> for Node {
    fn from(grid: GridMap<Sprite>) -> Self {
        Node {
            active_curio: None,
            active_team: Team::PlayerTeam,
            enemy_ai: EnemyAi::Simple,
            grid,
            name: String::from("Node"),
            inventory: Inventory::default(),
        }
    }
}

impl From<(String, GridMap<Sprite>)> for Node {
    fn from((name, grid): (String, GridMap<Sprite>)) -> Self {
        Node {
            active_curio: None,
            active_team: Team::PlayerTeam,
            enemy_ai: EnemyAi::Simple,
            grid,
            name,
            inventory: Inventory::default(),
        }
    }
}

impl From<(String, GridMap<Sprite>, EnemyAi)> for Node {
    fn from((name, grid, enemy_ai): (String, GridMap<Sprite>, EnemyAi)) -> Self {
        Node {
            active_curio: None,
            active_team: Team::PlayerTeam,
            enemy_ai,
            grid,
            name,
            inventory: Inventory::default(),
        }
    }
}
