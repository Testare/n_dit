use std::sync::Arc;

use super::super::super::error::{ErrorMsg as _, Result};
use super::super::super::Metadata;
use super::super::curio_action::CurioAction;
use super::super::keys::node_change_keys as keys;
use super::Node;
use super::SpritePoint;
use crate::{Bounds, Curio, Direction, GridMap, Point, PointSet, Sprite, Team};
use std::{cmp, collections::HashSet, num::NonZeroUsize, ops::Deref, ops::DerefMut};

const CURIO_KEY_IS_VALID: &str = "Curio key is expected to be valid key for node grid";

pub struct WithCurioGeneric<N: Deref<Target = Node>> {
    node: N,
    curio_key: usize,
}

pub type WithCurio<'a> = WithCurioGeneric<&'a Node>;
pub type WithCurioMut<'a> = WithCurioGeneric<&'a mut Node>;

impl<N: Deref<Target = Node>> WithCurioGeneric<N> {
    /// Returns internal curio data struct
    fn curio(&self) -> &Curio {
        if let Sprite::Curio(curio) = self
            .node
            .grid()
            .item(self.curio_key)
            .expect(CURIO_KEY_IS_VALID)
        {
            curio
        } else {
            panic!("{}", CURIO_KEY_IS_VALID);
        }
    }

    pub fn action_count(&self) -> usize {
        self.action_names().len()
    }

    /// List of actions the curio can take
    pub fn actions(&self) -> Result<Vec<Arc<CurioAction>>> {
        self.action_names()
            .iter()
            .map(|action| {
                self.node
                    .action_dictionary()
                    .get(action)
                    .ok_or_else(|| format!("Sprite action [{}] missing from dictionary", action).fail_critical_msg())
            })
            .collect()
    }

    pub fn action_names(&self) -> &Vec<String> {
        if let Sprite::Curio(curio) = self.node.grid().item(self.curio_key).unwrap() {
            curio.actions()
        } else {
            panic!("{}", CURIO_KEY_IS_VALID);
        }
    }

    pub fn action(&self, key: &str) -> Option<Arc<CurioAction>> {
        if self.action_names().contains(&key.to_string()) {
            self.node.action_dictionary.get(key)
        } else {
            None
        }
    }

    pub fn indexed_action(&self, index: usize) -> Option<Arc<CurioAction>> {
        let action_name = self.action_names().get(index)?;
        self.action(action_name)
    }

    /// Tests if the curio can go in the specified direction, if it is a legal move.
    ///
    /// Does not check if the curio has moves left or is tapped.
    pub fn can_move(self, dir: Direction) -> bool {
        if let Some(next_pt) = self.head() + dir {
            let grid = self.node.grid();
            if grid.square_is_free(next_pt) {
                return true;
            }
            if grid.item_key_at(next_pt) == Some(self.curio_key) {
                return true;
            }
            // Might involve more involved checks in the future
            if matches!(grid.item_at(next_pt), Some(Sprite::Pickup(_))) {
                return true;
            }
        }
        false
    }

    pub fn size(&self) -> usize {
        self.node.grid().len_of(self.curio_key)
    }

    pub fn max_size(&self) -> usize {
        self.curio().max_size()
    }

    pub fn head(&self) -> Point {
        self.node
            .grid()
            .head(self.curio_key)
            .expect(CURIO_KEY_IS_VALID)
    }

    pub fn moves(&self) -> usize {
        self.curio().moves()
    }

    pub fn key(&self) -> usize {
        self.curio_key
    }

    pub fn tapped(&self) -> bool {
        self.curio().tapped()
    }

    pub fn untapped(&self) -> bool {
        self.curio().untapped()
    }

    pub fn team(&self) -> Team {
        self.curio().team()
    }

    pub fn range_of_action(&self, action_index: usize) -> Option<PointSet> {
        let pt = self.head();
        let bounds = self.node.bounds();
        let range = self.indexed_action(action_index)?.range()?.get();
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

    // TODO move to WithCurio
    // TODO Take pick-ups into account
    pub fn possible_moves(&self) -> PointSet {
        let curio = self.curio();
        if curio.moves() == 0 || curio.tapped() {
            return PointSet::default();
        }
        let bounds = self.node.bounds();
        fn possible_moves_recur(
            point: Point,
            hash_set: HashSet<Point>,
            moves: usize,
            bounds: &Bounds,
            curio_key: usize,
            grid: &GridMap<Sprite>,
        ) -> HashSet<Point> {
            if moves == 0 {
                hash_set
            } else {
                Direction::EVERY_DIRECTION
                    .iter()
                    .fold(hash_set, |mut set, dir| {
                        let next_pt = dir.add_to_point(point, 1, *bounds);
                        if grid.square_is_free(next_pt)
                            || grid.item_key_at(next_pt) == Some(curio_key)
                        {
                            set.insert(next_pt);
                            possible_moves_recur(next_pt, set, moves - 1, bounds, curio_key, grid)
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
        PointSet::Pts(possible_moves_recur(
            head,
            point_set,
            moves,
            &bounds,
            self.curio_key,
            self.node.grid(),
        ))
    }
}

impl<N: DerefMut<Target = Node>> WithCurioGeneric<N> {
    /// Internal function to get mutable access to the curio data
    fn curio_mut(&mut self) -> &mut Curio {
        if let Some(Sprite::Curio(curio)) = self.node.grid_mut().item_mut(self.curio_key) {
            curio
        } else {
            panic!("{}", CURIO_KEY_IS_VALID)
        }
    }

    /// Number of moves the curio has taken since it was last untapped.
    pub fn moves_taken(&self) -> usize {
        self.curio().moves_taken()
    }

    /// Increases the max size of the curio
    pub fn set_max_size(&mut self, max_size: usize) {
        self.curio_mut().set_max_size(max_size);
    }

    /// Increases the max size of the curio
    pub fn increase_max_size(&mut self, increase: usize, bound: Option<NonZeroUsize>) -> usize {
        let curio = self.curio_mut();
        // TODO test this logic
        let increased_max_size = curio.max_size() + increase;
        let bounded_max_size = bound
            .map(|bnd| cmp::min(increased_max_size, bnd.get()))
            .unwrap_or(increased_max_size);
        let final_max_size = cmp::max(bounded_max_size, curio.max_size());
        curio.set_max_size(final_max_size);
        final_max_size
    }

    /// The curio finishes its action for this turn.
    pub fn tap(&mut self) {
        self.curio_mut().tap();
        if self.node.active_curio_key() == Some(self.curio_key) {
            self.node.drop_active_curio(); // deactivate_curio();
        }
    }

    /// Makes the curio able to take a turn again (move and take action)
    pub fn untap(&mut self) {
        self.curio_mut().untap();
    }

    /// Consumes self since the curio might be deleted, and thus the curio key is no longer valid
    pub fn take_damage(mut self, dmg: usize) -> (Vec<SpritePoint>, Option<Sprite>) {
        let grid_mut = self.node.grid_mut();
        let remaining_square_len = grid_mut.len_of(self.curio_key).saturating_sub(dmg);
        let dropped_squares: Vec<SpritePoint> = grid_mut
            .square_iter(self.curio_key)
            .skip(remaining_square_len)
            .map(|sqr| SpritePoint(self.curio_key, sqr.location()))
            .collect();
        (dropped_squares, grid_mut.pop_back_n(self.curio_key, dmg))
    }

    pub fn go(&mut self, direction: Direction) -> Result<Metadata> {
        if self.moves() == 0 || self.tapped() {
            return "Curio cannot move".invalid();
        }
        let mut metadata = self.node.default_metadata()?;

        let at_max_size = self.size() >= self.max_size();
        let next_pt = direction.add_to_point(self.head(), 1, self.node.bounds());
        let grid_mut = self.node.grid_mut();

        match grid_mut.item_at(next_pt) {
            Some(Sprite::AccessPoint) => "Move curio collision with access point".fail_critical(),
            Some(Sprite::Pickup(_)) => {
                let key = grid_mut.item_key_at(next_pt).unwrap();
                if let Some(Sprite::Pickup(pickup)) = grid_mut.pop_front(key) {
                    metadata.put(keys::PICKUP, &pickup)?;
                    self.node.inventory.pick_up(pickup);
                    Ok(())
                } else {
                    "Something weird happened, is pop-up taking more than one location?"
                        .fail_critical()
                }
            }
            Some(Sprite::Curio(_)) => {
                let key = grid_mut.item_key_at(next_pt).unwrap();
                if key != self.curio_key {
                    "Cannot move curio into another curio".invalid()
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }?;

        let grid_mut = self.node.grid_mut();

        let sucessful_movement = grid_mut.push_front(next_pt, self.curio_key);
        if sucessful_movement {
            if at_max_size {
                let dropped_point = grid_mut.back(self.curio_key).unwrap();
                metadata.put(keys::DROPPED_POINT, &dropped_point)?;
                grid_mut.pop_back(self.curio_key);
            }
            let curio = self.curio_mut();
            curio.took_a_move();
            Ok(metadata)
        } else {
            format!("Unable to move curio {:?}", direction).invalid()
        }
    }

    pub(crate) fn grow_back(&mut self, pt: Point) -> bool {
        self.node.grid_mut().push_back(pt, self.curio_key)
    }

    pub(super) fn drop_front(&mut self) -> bool {
        self.node.grid_mut().pop_front(self.curio_key).is_some()
    }
}

impl Node {
    pub fn with_active_curio<F, R, O>(&self, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithCurio<'brand>) -> O,
        O: Into<Option<R>>,
    {
        self.active_curio_key()
            .and_then(|key| self.with_curio(key, f))
    }

    pub(crate) fn with_active_curio_mut<F, R, O>(&mut self, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithCurioMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        self.active_curio_key()
            .and_then(|key| self.with_curio_mut(key, f))
    }

    pub fn with_curio<F, R, O>(&self, curio_key: usize, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithCurio<'brand>) -> O,
        O: Into<Option<R>>,
    {
        if let Some(Sprite::Curio(..)) = self.grid().item(curio_key) {
            let with_curio_mut = WithCurio {
                node: self,
                curio_key,
            };
            f(with_curio_mut).into()
        } else {
            None
        }
    }

    pub(crate) fn with_curio_mut<F, R, O>(&mut self, curio_key: usize, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithCurioMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        if let Some(Sprite::Curio(..)) = self.grid().item(curio_key) {
            let with_curio_mut = WithCurioMut {
                node: self,
                curio_key,
            };
            f(with_curio_mut).into()
        } else {
            None
        }
    }

    pub fn with_curio_at<F, R, O>(&self, pt: Point, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithCurio<'brand>) -> O,
        O: Into<Option<R>>,
    {
        let curio_key = self.grid().item_key_at(pt)?;
        self.with_curio(curio_key, f)
    }

    pub(crate) fn with_curio_at_mut<F, R, O>(&mut self, pt: Point, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithCurioMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        let curio_key = self.grid().item_key_at(pt)?;
        self.with_curio_mut(curio_key, f)
    }
}
