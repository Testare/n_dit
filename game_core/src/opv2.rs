use std::collections::VecDeque;
use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::{StaticSystemParam, SystemId};
use bevy::reflect::TypePath;
use thiserror::Error;

use crate::prelude::*;

// TODO refactor most of this into a module "op_sys" which will be more generic, 
// then the more game-specific queues defined somewhere else
#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct PrimeOpQueue(VecDeque<OpRequest<Self>>);

pub trait OpErrorUtils<T> {
    fn critical(self) -> Result<T, OpError>;
    fn invalid(self) -> Result<T, OpError>;
}

impl <T> OpErrorUtils<T> for &str {
    fn critical(self) -> Result<T, OpError> {
        Err(OpError::OpFailureCritical(anyhow::anyhow!(self.to_string())))
    }
    fn invalid(self) -> Result<T, OpError> {
        Err(OpError::InvalidOp(self.to_string()))
    }
}

impl <T, E: std::error::Error + Send + Sync + 'static> OpErrorUtils<T> for Result<T, E> {
    fn critical(self) -> Result<T, OpError> {
        self.map_err(|e|OpError::OpFailureCritical(anyhow::Error::from(e)))
    }

    fn invalid(self) -> Result<T, OpError> {
        self.map_err(|e|OpError::InvalidOp(e.to_string()))
    }
}


/// A type alias for Result<Metadata, OpError>, different 
/// from OpResult because OpResult also contains a copy of the 
/// Op.
pub type OpSysResult = Result<Metadata, OpError>;

#[derive(Debug)]
pub struct OpPlugin<T: OpV2 + TypePath + FromReflect>(PhantomData<T>);

impl <T: OpV2 + TypePath + FromReflect> Default for OpPlugin<T> {
    fn default() -> Self {
        OpPlugin(default())
    }
}

impl<T: OpV2 + TypePath + FromReflect> Plugin for OpPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_register_op::<T>)
            .add_event::<OpResult<T>>();
    }
}


#[derive(Debug)]
pub struct OpQueuePlugin<Q, S: ScheduleLabel + Clone = Update> {
    phantom_data: PhantomData<Q>,
    schedule: S
}

impl <Q> Default for OpQueuePlugin<Q, Update> {
    fn default() -> Self {
        OpQueuePlugin { phantom_data: PhantomData, schedule: Update }
    }
}

impl<Q: OpQueue, K: ScheduleLabel + Clone> Plugin for OpQueuePlugin<Q, K> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Q>()
            .init_resource::<OpLoader<Q>>()
            .add_systems(self.schedule.clone(), sys_perform_ops::<Q>);
    }
}

#[derive(Default, Resource)]
struct OpRegistry(HashMap<&'static str, Vec<SystemId>>);

impl OpRegistry {
    fn add_op_system<T: TypePath>(&mut self, system_id: SystemId) {
        self.0.entry(T::type_path()).or_default().push(system_id);
    }

    fn get_op_system(&self, type_path: &str, index: usize) -> Option<SystemId> {
        self.0
            .get(type_path)
            .and_then(|system_ids| system_ids.get(index).copied())
    }
}

#[derive(Debug)]
pub struct OpRegistrar<'a, O: OpV2 + TypePath + FromReflect>(&'a mut World, PhantomData<O>);

impl<'a, O: OpV2 + TypePath + FromReflect> OpRegistrar<'a, O> {
    pub fn register_op<M: 'static, S>(&mut self, op_sys: S) -> &mut Self
    where
        S: SystemParamFunction<M, In = (Entity, O), Out = Result<Metadata, OpError>>,
    {
        let sys_id = self.0.register_system(wrap_op_system(op_sys));
        let mut op_reg = self.0.get_resource_or_insert_with(OpRegistry::default);
        op_reg.add_op_system::<O>(sys_id);
        self
    }
}

#[derive(Debug, Resource)]
struct OpLoader<Q: OpQueue>(Option<OpRequest<Q>>);

impl<Q: OpQueue> Default for OpLoader<Q> {
    fn default() -> Self {
        OpLoader(None)
    }
}

fn wrap_op_system<S, M, O>(
    mut op_sys: S,
) -> impl FnMut(ResMut<OpLoader<O::Queue>>, StaticSystemParam<S::Param>, EventWriter<OpResult<O>>)
where
    S: SystemParamFunction<M, In = (Entity, O), Out = Result<Metadata, OpError>>,
    O: OpV2 + FromReflect,
{
    move |mut op_loader, param, mut evw| {
        if let Some(op_request) = op_loader.0.take() {
            let OpRequest { op, source } = op_request;
            let reflect_op = op.into_reflect();
            let op: O =
                FromReflect::from_reflect(&*reflect_op.clone_value()).expect("Unwrap should be good?");
            // It would be nice if we could pass a reference of Op to the system instead, but that isn't working
            let result = op_sys.run((source.clone(), op), param.into_inner());
            let res = OpResult {
                source,
                op: *reflect_op.downcast().unwrap(),
                result,
            };
            evw.send(res);
        }
    }
}

fn sys_perform_ops<Q: OpQueue>(world: &mut World) {
    // TODO make resource to get list of Ops to perform
    let mut ops_queue = world
        .get_resource_mut::<Q>()
        .expect("OpQueue not initialized");
    if ops_queue.is_empty() {
        return;
    }
    let ops: Vec<_> = ops_queue.drain(..).collect();

    let ops = world.get_resource::<OpRegistry>().map(|op_registry| {
        ops.into_iter()
            .map(|op| {
                let sys_id = op_registry.get_op_system(op.op.reflect_type_path(), op.op.system_index());
                (op, sys_id)
            })
            .collect::<Vec<_>>()
    });
    for (op_data, op_sys) in ops.unwrap_or_default().into_iter() {
        match op_sys {
            None => {
                log::warn!("Op has no registered system: {op_data:?}")
            },
            Some(op_sys) => {
                world
                    .get_resource_mut::<OpLoader<Q>>()
                    .expect("OpLoader for OpQueue should be initialized")
                    .0 = Some(op_data);
                if let Err(e) = world.run_system(op_sys) {
                    log::error!("Error running op system [{op_sys:?}]: {e:?}");
                }
            },
        }
    }
}

fn sys_register_op<O: OpV2 + TypePath + FromReflect>(world: &mut World) {
    let op_reg: OpRegistrar<O> = OpRegistrar(world, PhantomData);
    O::register_systems(op_reg);
}

#[derive(Debug)]
pub struct OpRequest<Q: OpQueue + std::fmt::Debug + ?Sized > {
    source: Entity,
    op: Box<dyn OpV2<Queue = Q>>
}

pub trait OpV2: std::fmt::Debug + Sync + Send + Reflect + 'static {
    type Queue: OpQueue;
    // type Queue; Multiple queues to be added later

    fn system_index(&self) -> usize;
    fn register_systems(registrar: OpRegistrar<Self>)
    where
        Self: Sized + TypePath + FromReflect;

    fn to_request(self, source: Entity) -> OpRequest<Self::Queue> 
        where Self: Sized {
        OpRequest { 
            source, 
            op: Box::new(self)
        }
    }
}

pub trait OpQueue:
    core::ops::DerefMut
    + std::fmt::Debug
    + core::ops::Deref<Target = VecDeque<OpRequest<Self>>>
    + Resource
    + FromWorld
{
}
impl<
        T: core::ops::DerefMut
            // + core::ops::Deref<Target = VecDeque<Box<dyn OpV2<Queue = Self>>>>
            + core::ops::Deref<Target = VecDeque<OpRequest<Self>>>
            + Resource
            + std::fmt::Debug
            + FromWorld,
    > OpQueue for T
{
}

#[derive(Debug, Error)]
pub enum OpError {
    /// This error usually means a user or system was trying to perform an OP but it is unsuccessful
    #[error("Invalid op: {0}")]
    InvalidOp(String),
    /// The op doesn't match the system it was called for.
    #[error("Dev error: System called for op that does not match")]
    MismatchedOpSystem,
    /// This error means we encountered an expected error while performing the op but we can continue running
    #[error("Encountered error [{1:3}] running op: {0}")]
    OpFailureRecoverable(#[source] anyhow::Error, usize),
    /// This error means we encoutered a failure we are not able to recover from, panic
    #[error("Encountered a fatal error running op: {0}")]
    OpFailureCritical(#[from] anyhow::Error),
}

impl From<String> for OpError {
    fn from(value: String) -> Self {
        Self::InvalidOp(value)
    }
}

impl From<&str> for OpError {
    fn from(value: &str) -> Self {
        Self::InvalidOp(String::from(value))
    }
}

#[derive(Debug, Event, getset::Getters)]
pub struct OpResult<O> {
    #[getset(get = "pub")]
    pub source: Entity,
    #[getset(get = "pub")]
    pub op: O,
    #[getset(get = "pub")]
    pub result: Result<Metadata, OpError>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default, Deref, DerefMut, Resource)]
    pub struct ExampleQueue(VecDeque<OpRequest<Self>>);

    #[derive(Debug, Default, Reflect, PartialEq)]
    pub enum ExampleOp {
        #[default]
        ExampleOne,
        ExampleTwo,
    }

    impl OpV2 for ExampleOp {
        type Queue = ExampleQueue;
        fn register_systems(mut registrar: OpRegistrar<Self>) {
            registrar
                .register_op(|In((_, op))| {
                    println!("Hello from One: {op:?}!");
                    Ok(Metadata::default())
                })
                .register_op(|In((_, op)), mut count: Local<usize>| {
                    *count += 1;
                    let modulo = *count % 2;
                    println!("Hello from Two: {op:?} count: {count:?}! Module {modulo}");
                    if modulo == 0 {
                        Err("Bad luck, getting an even number")?;
                    }
                    Ok(Metadata::default())
                });
        }

        fn system_index(&self) -> usize {
            match self {
                Self::ExampleOne => 0,
                Self::ExampleTwo => 1,
            }
        }
    }

    #[test]
    fn test_op_system() {
        let mut app = App::new();
        app.add_plugins((
            OpQueuePlugin::<ExampleQueue>::default(),
            OpPlugin::<ExampleOp>::default(),
        ))
        .add_systems(Startup, |mut res: ResMut<ExampleQueue>| {
            res.push_back(ExampleOp::ExampleOne.to_request(Entity::PLACEHOLDER));
            res.push_back(ExampleOp::ExampleTwo.to_request(Entity::PLACEHOLDER));
            res.push_back(ExampleOp::ExampleTwo.to_request(Entity::PLACEHOLDER));
            res.push_back(ExampleOp::ExampleTwo.to_request(Entity::PLACEHOLDER));
        })
        .add_systems(PostUpdate, |mut evr: EventReader<OpResult<ExampleOp>>| {
            let results: Vec<_> = evr.read().collect();
            assert!(
                matches!(
                    results[0],
                    OpResult {
                        source: Entity::PLACEHOLDER,
                        op: ExampleOp::ExampleOne,
                        result: Ok(_)
                    }
                ),
                "Result one should match"
            );
            assert!(
                matches!(
                    results[1],
                    OpResult {
                        source: Entity::PLACEHOLDER,
                        op: ExampleOp::ExampleTwo,
                        result: Ok(_)
                    }
                ),
                "Result two should match"
            );
            assert!(
                matches!(
                    results[2],
                    OpResult {
                        source: Entity::PLACEHOLDER,
                        op: ExampleOp::ExampleTwo,
                        result: Err(OpError::InvalidOp(_))
                    }
                ),
                "Result three should match"
            );
            assert_eq!(
                results[2].result().as_ref().unwrap_err().to_string(),
                "Invalid op: Bad luck, getting an even number"
            );
            assert!(
                matches!(
                    results[3],
                    OpResult {
                        source: Entity::PLACEHOLDER,
                        op: ExampleOp::ExampleTwo,
                        result: Ok(_)
                    }
                ),
                "Result four should match"
            );
        });
        app.update();
    }
}
