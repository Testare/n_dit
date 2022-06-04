pub mod node_change_keys {
    use crate::{Metadata, Pickup, Point, Team};
    use typed_key::{typed_key, Key};

    pub const TEAM: Key<Team> = typed_key!("team");
    pub const PICKUP: Key<Pickup> = typed_key!("pickup");
    pub const DROPPED_POINT: Key<Point> = typed_key!("droppedPoint");
    pub const CURIO_ACTION_METADATA: Key<Metadata> = typed_key!("actionMetadata");
    pub const PERFORMING_CURIO: Key<usize> = typed_key!("performingCurio");
}

pub mod curio_action_keys {
    use super::super::SpritePoint;
    use crate::Sprite;
    use typed_key::{typed_key, Key};

    pub const DAMAGES: Key<Vec<SpritePoint>> = typed_key!("damages");
    pub const DELETED_SPRITES: Key<Vec<(usize, Sprite)>> = typed_key!("deletedSprites");

    pub const TARGET_CURIO: Key<usize> = typed_key!("targetCurio");
    pub const MAX_SIZE_CHANGE: Key<(usize, usize)> = typed_key!("movementSpeedChange");
}
