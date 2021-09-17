use super::{Node};
use crate::{Direction, Piece, Point, PointSet, Sprite, StandardSpriteAction};

const SPRITE_KEY_IS_VALID: &'static str = "Sprite key is expected to be valid key for node grid";

pub struct WithSpriteMut<'a> {
    node: &'a mut Node,
    sprite_key: usize,
}

pub struct WithSprite<'a> {
    node: &'a Node,
    sprite_key: usize,
}

impl<'a> WithSpriteMut<'a> {
    const SPRITE_KEY_IS_VALID: &'static str =
        "Sprite key is expected to be valid key for node grid";

    pub fn head(&self) -> Point {
        self.node
            .grid()
            .head(self.sprite_key)
            .expect(Self::SPRITE_KEY_IS_VALID)
    }

    pub fn size(&self) -> usize {
        self.node.grid().len_of(self.sprite_key)
    }

    pub fn moves(&self) -> usize {
        self.with_sprite(|sprite|sprite.moves())
    }
    
    pub fn max_size(&self) -> usize {
        self.with_sprite(|sprite|sprite.max_size())
    }

    pub fn tap(&mut self) {
        self.with_sprite_mut(|sprite|sprite.tap());
        if self.node.active_sprite_key() == Some(self.sprite_key) {
            self.node.drop_active_sprite();// deactivate_sprite();
        }
    }

    // NOTE maybe this shouldn't be public?
    pub fn actions(&self) -> &Vec<StandardSpriteAction> {
        if let Piece::Program(sprite) = self.node.grid().item(self.sprite_key).unwrap() {
            sprite.actions()
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID);
        }
    }

    /// Returns remaining moves
    pub fn move_sprite(&mut self, directions: Vec<Direction>) -> Result<usize, String> {
        if self.node.with_sprite(self.sprite_key, |sprite| sprite.moves() == 0 || sprite.tapped()).expect(SPRITE_KEY_IS_VALID) {
            return Err("Sprite cannot move".to_string());
        }
        let bounds = self.node.bounds();
        let mut size = self.size();
        let mut remaining_moves = self.moves();
        let max_size = self.max_size();
        let has_no_actions = self.actions().is_empty();

        for dir in directions {
            let head = self.head();
            let next_pt = dir.add_to_point(head, 1, bounds);
            let sucessful_movement = self.node.grid_mut().push_front(next_pt, self.sprite_key);
            if sucessful_movement {
                size += 1;
                remaining_moves = self
                    .with_sprite_mut(|sprite| {
                        sprite.took_a_move();
                        sprite.moves()
                    })
            }
            if remaining_moves == 0 {
                break;
            }
        }
        // Tap if there are no remaining moves or actions
        if has_no_actions {
            self.tap();
        }
        self.node.grid_mut()
            .pop_back_n(self.sprite_key, size.checked_sub(max_size).unwrap_or(0));

        Ok(remaining_moves)
    }


    fn with_sprite<R, F: FnOnce(&Sprite) -> R>(
        &self,
        sprite_op: F,
    ) -> R {
        if let Some(Piece::Program(sprite)) = self.node.grid().item(self.sprite_key) {
            sprite_op(sprite).into()
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID)
        }
    }

    fn with_sprite_mut<R, F: FnOnce(&mut Sprite) -> R>(
        &mut self,
        sprite_op: F,
    ) -> R {
        if let Some(Piece::Program(sprite)) = self.node.grid_mut().item_mut(self.sprite_key) {
            sprite_op(sprite).into()
        } else {
            panic!("{}", SPRITE_KEY_IS_VALID)
        }
    }
}

impl<'a> WithSprite<'a> {
    const SPRITE_KEY_IS_VALID: &'static str =
        "Sprite key is expected to be valid key for node grid";

    pub fn head(&self) -> Point {
        self.node
            .grid()
            .head(self.sprite_key)
            .expect(Self::SPRITE_KEY_IS_VALID)
    }

    fn sprite(&self) -> &Sprite {
        if let Piece::Program(sprite) = self
            .node
            .grid()
            .item(self.sprite_key)
            .expect(Self::SPRITE_KEY_IS_VALID)
        {
            sprite
        } else {
            panic!("{}", Self::SPRITE_KEY_IS_VALID);
        }
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
    pub fn with_active_sprite_mut_wrapped<F, R, O>(&mut self, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSpriteMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        self.active_sprite_key()
            .and_then(|key| self.with_sprite_mut_wrapped(key, f))
    }

    pub fn with_active_sprite_wrapped<F, R, O>(&self, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSprite<'brand>) -> O,
        O: Into<Option<R>>,
    {
        self.active_sprite_key()
            .and_then(|key| self.with_sprite_wrapped(key, f))
    }

    pub fn with_sprite_mut_wrapped<F, R, O>(&mut self, sprite_key: usize, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSpriteMut<'brand>) -> O,
        O: Into<Option<R>>,
    {
        let with_sprite_mut = WithSpriteMut {
            node: self,
            sprite_key,
        };
        f(with_sprite_mut).into()
    }

    pub fn with_sprite_wrapped<F, R, O>(&self, sprite_key: usize, f: F) -> Option<R>
    where
        for<'brand> F: FnOnce(WithSprite<'brand>) -> O,
        O: Into<Option<R>>,
    {
        let with_sprite_mut = WithSprite {
            node: self,
            sprite_key,
        };
        f(with_sprite_mut).into()
    }
}
