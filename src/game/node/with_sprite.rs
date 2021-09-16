use super::Node;
use crate::{Piece, Point, PointSet, Sprite};

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

    fn head(&self) -> Point {
        self.node
            .grid()
            .head(self.sprite_key)
            .expect(Self::SPRITE_KEY_IS_VALID)
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
