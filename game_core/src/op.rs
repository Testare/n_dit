use crate::prelude::*;

#[derive(Clone, Debug, Event)]
pub struct Op<O: OpSubtype> {
    pub op: O,
    pub player: Entity,
}

pub trait OpSubtype: Clone + Send + Sync + 'static {
    type Error;

    fn for_p(self, player: Entity) -> Op<Self> {
        Op::new(player, self)
    }
}

#[derive(Clone, Debug, Event, getset::Getters)]
pub struct OpResult<O: OpSubtype> {
    #[getset(get = "pub")]
    pub source: Op<O>,
    #[getset(get = "pub")]
    pub result: Result<Metadata, O::Error>,
}

impl<O: OpSubtype> OpResult<O> {
    pub fn new(source: &Op<O>, result: Result<Metadata, O::Error>) -> Self {
        OpResult {
            source: source.clone(),
            result,
        }
    }
}

impl<O: OpSubtype> Op<O> {
    pub fn new(player: Entity, op: O) -> Self {
        Op { op, player }
    }

    pub fn op(&self) -> &O {
        &self.op
    }

    pub fn player(&self) -> Entity {
        self.player
    }

    pub fn send(self, evw: &mut EventWriter<Op<O>>) {
        evw.send(self)
    }
}
