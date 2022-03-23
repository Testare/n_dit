use super::Node;
use crate::{GridMap, Bounds, Direction, Pickup, Piece, Point, PointSet, Sprite, StandardSpriteAction, Team};
use std::{cmp, num::NonZeroUsize, ops::Deref, ops::DerefMut, collections::HashSet};

const SPRITE_KEY_IS_VALID: &str = "Sprite key is expected to be valid key for node grid";

pub struct WithSpriteGeneric<N: Deref<Target = Node>> {
    node: N,
    sprite_key: usize,
}

pub type WithSprite<'a> = WithSpriteGeneric<&'a Node>;
pub type WithSpriteMut<'a> = WithSpriteGeneric<&'a mut Node>;

impl<N: Deref<Target = Node>> WithSpriteGeneric<N> {
    /// Returns internal sprite data struct
    fn sprite(&self) -> &Sprite {
        if let Piece::Program(sprite) = self
            .node
            .grid()
            .item(self.sprite_key)
            .expect(SPRITE_KEY_IS_VALID)
        {
            sprite
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID);
        }
    }

    // NOTE maybe this shouldn't be public?
    /// List of actions the sprite can take
    pub fn actions(&self) -> &Vec<StandardSpriteAction> {
        if let Piece::Program(sprite) = self.node.grid().item(self.sprite_key).unwrap() {
            sprite.actions()
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID);
        }
    }

    /// Tests if the sprite can go in the specified direction, if it is a legal move.
    ///
    /// Does not check if the sprite has moves left or is tapped.
    pub fn can_move(self, dir: Direction) -> bool {
        if let Some(next_pt) = self.head() + dir {
            let grid = self.node.grid();
            if grid.square_is_free(next_pt) {
                return true;
            }
            if grid.item_key_at(next_pt) == Some(self.sprite_key) {
                return true;
            }
            // Might involve more involved checks in the future
            if matches!(grid.item_at(next_pt), Some(Piece::Pickup(_))) {
                return true;
            }
        }
        false
    }


    pub fn size(&self) -> usize {
        self.node.grid().len_of(self.sprite_key)
    }

    pub fn max_size(&self) -> usize {
        self.sprite().max_size()
    }

    pub fn head(&self) -> Point {
        self.node
            .grid()
            .head(self.sprite_key)
            .expect(SPRITE_KEY_IS_VALID)
    }

    pub fn moves(&self) -> usize {
        self.sprite().moves()
    }

    pub fn key(&self) -> usize {
        self.sprite_key
    }

    pub fn tapped(&self) -> bool {
        self.sprite().tapped()
    }

    pub fn untapped(&self) -> bool {
        self.sprite().untapped()
    }

    pub fn team(&self) -> Team {
        self.sprite().team()
    }

    pub fn range_of_action(&self, action_index: usize) -> Option<PointSet> {
        let pt = self.head();
        let bounds = self.node.bounds();
        let range = self
            .sprite()
            .actions()
            .get(action_index)?
            .unwrap()
            .range()?
            .get();
        Some(PointSet::range_of_pt(pt, range, bounds))
        // TODO remove closed squares unless they're a target.
        /* TODO Perhaps changes PointSet to "TargetSet", with difference
        // treatment of different points. For example, actions that target enemies
        // should highlight enemies, or actions that target closed squares. This would probably be best done after we have
        caching for the action range. In the case of actions that target closed squares, we might
        even need to draw square borders. A lot of complication might arise here.

        This actually sounds a lot like UI specific action, perhaps "target_points" should be
        a separate function.*/
    }

    // TODO move to WithSprite
    // TODO Take pick-ups into account
    pub fn possible_moves(&self) -> PointSet {
        let sprite = self.sprite();
        if sprite.moves() == 0 || sprite.tapped() {
            return PointSet::default();
        }
        let bounds = self.node.bounds();
        fn possible_moves_recur(
            point: Point,
            hash_set: HashSet<Point>,
            moves: usize,
            bounds: &Bounds,
            sprite_key: usize,
            grid: &GridMap<Piece>,
        ) -> HashSet<Point> {
            if moves == 0 {
                hash_set
            } else {
                Direction::EVERY_DIRECTION
                    .iter()
                    .fold(hash_set, |mut set, dir| {
                        let next_pt = dir.add_to_point(point, 1, *bounds);
                        if grid.square_is_free(next_pt)
                            || grid.item_key_at(next_pt) == Some(sprite_key)
                        {
                            set.insert(next_pt);
                            possible_moves_recur(
                                next_pt,
                                set,
                                moves - 1,
                                bounds,
                                sprite_key,
                                grid,
                            )
                        } else {
                            set
                        }
                    })
            }
        }
        let head = self.head();
        let moves = self.moves();
        let mut point_set = HashSet::new();
        point_set.insert(head);
        PointSet::Pts(possible_moves_recur(head, point_set, moves, &bounds, self.sprite_key, self.node.grid()))
    }

}

impl<N: DerefMut<Target = Node>> WithSpriteGeneric<N> {
    /// Internal function to get mutable access to the sprite data
    fn sprite_mut(&mut self) -> &mut Sprite {
        if let Some(Piece::Program(sprite)) = self.node.grid_mut().item_mut(self.sprite_key) {
            sprite
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID)
        }
    }

    /// Number of moves the sprite has taken since it was last untapped.
    pub fn moves_taken(&self) -> usize {
        self.sprite().moves_taken()
    }

    /// Increases the max size of the sprite
    pub fn increase_max_size(&mut self, increase: usize, bound: Option<NonZeroUsize>) -> usize {
        let sprite = self.sprite_mut();
        // TODO test this logic
        let increased_max_size = sprite.max_size() + increase;
        let bounded_max_size = bound
            .map(|bnd| cmp::min(increased_max_size, bnd.get()))
            .unwrap_or(increased_max_size);
        let final_max_size = cmp::max(bounded_max_size, sprite.max_size());
        sprite.set_max_size(final_max_size);
        final_max_size
    }

    /// The sprite finishes its action for this turn.
    pub fn tap(&mut self) {
        self.sprite_mut().tap();
        if self.node.active_sprite_key() == Some(self.sprite_key) {
            self.node.drop_active_sprite(); // deactivate_sprite();
        }
    }

    /// Makes the sprite able to take a turn again (move and take action)
    pub fn untap(&mut self) {
        self.sprite_mut().untap();
    }

    /// Consumes self since the sprite might be deleted, and thus the sprite key is no longer valid
    pub fn take_damage(mut self, dmg: usize) -> Option<Piece> {
        self.node.grid_mut().pop_back_n(self.sprite_key, dmg)
    }

    /// Returns remaining moves
    pub fn move_sprite(&mut self, directions: &[Direction]) -> Result<Vec<Pickup>, String> {
        if self.moves() == 0 || self.tapped() {
            return Err("Sprite cannot move".to_string());
        }
        let bounds = self.node.bounds();
        let mut remaining_moves = self.moves();
        let max_size = self.max_size();
        let has_no_actions = self.actions().is_empty();
        let mut results = Vec::new();

        for dir in directions {
            let head = self.head();
            let next_pt = dir.add_to_point(head, 1, bounds);
            let grid_mut = self.node.grid_mut();
            if grid_mut
                .item_at(next_pt)
                .map(|piece| piece.is_pickup())
                .unwrap_or(false)
            {
                let key = grid_mut.item_key_at(next_pt).unwrap();
                if let Some(Piece::Pickup(pickup)) = grid_mut.pop_front(key) {
                    results.push(pickup);
                } else {
                    log::error!("Error when moving sprite [{:?}] from [{:?}] to [{:?}], trying to pick up [{:?}]", 
                        self.sprite_key,
                        head,
                        next_pt,
                        key
                    );
                    panic!("Error picking up pickup");
                }
            }
            let sucessful_movement = self.node.grid_mut().push_front(next_pt, self.sprite_key);
            if sucessful_movement {
                let sprite = self.sprite_mut();
                sprite.took_a_move();
                remaining_moves = sprite.moves();
            }
            if remaining_moves == 0 {
                break;
            }
        }

        let size = self.size();
        // Tap if there are no remaining moves or actions
        if has_no_actions {
            self.tap();
        }
        self.node
            .grid_mut()
            .pop_back_n(self.sprite_key, size.saturating_sub(max_size));

        Ok(results)
    }
}

impl Node {
    pub fn with_active_sprite<F, R, O>(&self, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSprite<'brand>) -> O,
        O: Into<Option<R>>,
    {
        self.active_sprite_key()
            .and_then(|key| self.with_sprite(key, f))
    }

    pub fn with_active_sprite_mut<F, R, O>(&mut self, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSpriteMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        self.active_sprite_key()
            .and_then(|key| self.with_sprite_mut(key, f))
    }

    pub fn with_sprite<F, R, O>(&self, sprite_key: usize, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSprite<'brand>) -> O,
        O: Into<Option<R>>,
    {
        if let Some(Piece::Program(..)) = self.grid().item(sprite_key) {
            let with_sprite_mut = WithSprite {
                node: self,
                sprite_key,
            };
            f(with_sprite_mut).into()
        } else {
            None
        }
    }

    pub fn with_sprite_mut<F, R, O>(&mut self, sprite_key: usize, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSpriteMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        if let Some(Piece::Program(..)) = self.grid().item(sprite_key) {
            let with_sprite_mut = WithSpriteMut {
                node: self,
                sprite_key,
            };
            f(with_sprite_mut).into()
        } else {
            None
        }
    }

    pub fn with_sprite_at<F, R, O>(&self, pt: Point, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSprite<'brand>) -> O,
        O: Into<Option<R>>,
    {
        let sprite_key = self.grid().item_key_at(pt)?;
        self.with_sprite(sprite_key, f)
    }

    pub fn with_sprite_at_mut<F, R, O>(&mut self, pt: Point, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSpriteMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        let sprite_key = self.grid().item_key_at(pt)?;
        self.with_sprite_mut(sprite_key, f)
    }
}
