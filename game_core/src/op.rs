use crate::prelude::*;

#[derive(Clone, Debug, Event)]
pub struct Op<O> {
    pub op: O,
    pub player: Entity,
}

pub trait OpSubtype: Clone {
    type Error;

    fn for_player(self, player: Entity) -> Op<Self> {
        Op::new(player, self)
    }
}

#[derive(Clone, Debug, Event, getset::Getters)]
pub struct OpResult<O: OpSubtype> {
    #[getset(get = "pub")]
    source: Op<O>,
    #[getset(get = "pub")]
    result: Result<Metadata, O::Error>,
}

impl<O: OpSubtype> OpResult<O> {
    pub fn new(source: &Op<O>, result: Result<Metadata, O::Error>) -> Self {
        OpResult {
            source: source.clone(),
            result,
        }
    }
}

impl<O> Op<O> {
    pub fn new(player: Entity, op: O) -> Self {
        Op { op, player }
    }

    pub fn op(&self) -> &O {
        &self.op
    }

    pub fn player(&self) -> Entity {
        self.player
    }
}
