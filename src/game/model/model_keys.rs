pub mod node_change_keys {
    use super::super::{DroppedSquare, SpritePoint};
    use crate::{Pickup, Sprite, Team, Point, Metadata};
    use typed_key::{typed_key, Key};

    pub const TEAM: Key<Team> = typed_key!("team");
    pub const PICKUP: Key<Pickup> = typed_key!("pickup");
    pub const DROPPED_POINT: Key<Point> = typed_key!("droppedPoint");
    pub const CURIO_ACTION_METADATA: Key<Metadata> = typed_key!("actionMetadata");

    pub const DROPPED_SQUARES: Key<Vec<DroppedSquare>> = typed_key!("droppedSquares");
    pub const PREVIOUS_ACTIVE_CURIO: Key<usize> = typed_key!("performingCurio");
    pub const PERFORMING_CURIO: Key<usize> = typed_key!("performingCurio");

    pub const DELETED_SPRITE: Key<(usize, Sprite)> = typed_key!("deletedSprite");
}

pub mod curio_action_keys {
    use typed_key::{typed_key, Key};
    use crate::{Sprite};
    use super::super::{SpritePoint};

    pub const DAMAGES: Key<Vec<SpritePoint>> = typed_key!("damages");
    pub const DELETED_SPRITES: Key<Vec<(usize, Sprite)>> = typed_key!("deletedSprites");
    #[deprecated]
    pub const DELETED_SPRITE: Key<(usize, Sprite)> = typed_key!("deletedSprite");
    pub const TARGET_CURIO: Key<usize> = typed_key!("targetCurio");
    pub const MOVEMENT_SPEED_CHANGE: Key<(isize)> = typed_key!("movementSpeedChange");

}
