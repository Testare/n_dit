use super::Node;
use crate::{Direction, Piece, Point, PointSet, Sprite, StandardSpriteAction, Team};
use std::{cmp, num::NonZeroUsize};

const SPRITE_KEY_IS_VALID: &str = "Sprite key is expected to be valid key for node grid";

pub struct WithSpriteMut<'a> {
    node: &'a mut Node,
    sprite_key: usize,
}

pub struct WithSprite<'a> {
    node: &'a Node,
    sprite_key: usize,
}

impl<'a> WithSpriteMut<'a> {
    pub fn head(&self) -> Point {
        self.node
            .grid()
            .head(self.sprite_key)
            .expect(SPRITE_KEY_IS_VALID)
    }

    pub fn size(&self) -> usize {
        self.node.grid().len_of(self.sprite_key)
    }

    pub fn moves(&self) -> usize {
        self.with_sprite(|sprite| sprite.moves())
    }

    pub fn moves_taken(&self) -> usize {
        self.with_sprite(|sprite| sprite.moves_taken())
    }

    pub fn max_size(&self) -> usize {
        self.with_sprite(|sprite| sprite.max_size())
    }

    pub fn increase_max_size(&mut self, increase: usize, bound: Option<NonZeroUsize>) -> usize {
        self.with_sprite_mut(|sprite| {
            // TODO test this logic
            let increased_max_size = sprite.max_size() + increase;
            let bounded_max_size = bound
                .map(|bnd| cmp::min(increased_max_size, bnd.get()))
                .unwrap_or(increased_max_size);
            let final_max_size = cmp::max(bounded_max_size, sprite.max_size());

            sprite.set_max_size(final_max_size);
            final_max_size
        })
    }

    pub fn tapped(&self) -> bool {
        self.with_sprite(|sprite| sprite.tapped())
    }

    pub fn tap(&mut self) {
        self.with_sprite_mut(|sprite| sprite.tap());
        if self.node.active_sprite_key() == Some(self.sprite_key) {
            self.node.drop_active_sprite(); // deactivate_sprite();
        }
    }

    pub fn untap(&mut self) {
        self.with_sprite_mut(|sprite| sprite.untap());
    }

    pub fn team(&mut self) -> Team {
        self.with_sprite(Sprite::team)
    }

    // NOTE maybe this shouldn't be public?
    pub fn actions(&self) -> &Vec<StandardSpriteAction> {
        if let Piece::Program(sprite) = self.node.grid().item(self.sprite_key).unwrap() {
            sprite.actions()
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID);
        }
    }

    /// Consumes self since the sprite might be deleted, and thus the sprite key is no longer valid
    pub fn take_damage(self, dmg: usize) -> Option<Piece> {
        self.node.grid_mut().pop_back_n(self.sprite_key, dmg)
    }

    // TODO evaluate if we should no longer return remaining moves. Only if we don't return anything from GameState::apply_action
    /// Returns remaining moves
    pub fn move_sprite(&mut self, directions: &[Direction]) -> Result<usize, String> {
        if self.moves() == 0 || self.tapped() {
            return Err("Sprite cannot move".to_string());
        }
        let bounds = self.node.bounds();
        let mut remaining_moves = self.moves();
        let max_size = self.max_size();
        let has_no_actions = self.actions().is_empty();

        for dir in directions {
            let head = self.head();
            let next_pt = dir.add_to_point(head, 1, bounds);
            let sucessful_movement = self.node.grid_mut().push_front(next_pt, self.sprite_key);
            if sucessful_movement {
                remaining_moves = self.with_sprite_mut(|sprite| {
                    sprite.took_a_move();
                    sprite.moves()
                })
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

        Ok(remaining_moves)
    }

    fn with_sprite<R, F: FnOnce(&Sprite) -> R>(&self, sprite_op: F) -> R {
        if let Some(Piece::Program(sprite)) = self.node.grid().item(self.sprite_key) {
            sprite_op(sprite)
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID)
        }
    }

    fn with_sprite_mut<R, F: FnOnce(&mut Sprite) -> R>(&mut self, sprite_op: F) -> R {
        if let Some(Piece::Program(sprite)) = self.node.grid_mut().item_mut(self.sprite_key) {
            sprite_op(sprite)
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID)
        }
    }
}

impl<'a> WithSprite<'a> {
    // NOTE maybe this shouldn't be public?
    pub fn actions(&self) -> &Vec<StandardSpriteAction> {
        if let Piece::Program(sprite) = self.node.grid().item(self.sprite_key).unwrap() {
            sprite.actions()
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID);
        }
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
