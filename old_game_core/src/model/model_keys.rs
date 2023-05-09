pub mod node_change_keys {
    use crate::{Metadata, Point, Team};
    use super::super::{inventory::{Pickup, CardId}};
    use typed_key::{typed_key, Key};
    use std::collections::HashMap;

    pub const TEAM: Key<Team> = typed_key!("team");
    pub const PICKUP: Key<Pickup> = typed_key!("pickup");
    pub const DROPPED_POINT: Key<Point> = typed_key!("droppedPoint");
    pub const CURIO_ACTION_METADATA: Key<Metadata> = typed_key!("actionMetadata");
    pub const PERFORMING_CURIO: Key<usize> = typed_key!("performingCurio");
    pub const REPLACED_CARD: Key<CardId> = typed_key!("replacedCard");
    pub const ACCESS_POINT_MAP: Key<HashMap<usize, (Point, Option<CardId>)>> = typed_key!("accessPointMap");
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
