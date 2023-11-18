use std::collections::VecDeque;
use std::marker::PhantomData;

use bevy::ecs::system::{StaticSystemParam, SystemId};
use bevy::reflect::TypePath;
use thiserror::Error;

use crate::prelude::*;

#[derive(Debug, Default)]
struct OpPlugin<T: Op + TypePath + FromReflect>(PhantomData<T>);

impl<T: Op + TypePath + FromReflect> Plugin for OpPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_register_op::<T>)
            .add_event::<OpResult<T>>();
    }
}

#[derive(Debug, Default)]
struct OpQueuePlugin<Q> {
    phantom_data: PhantomData<Q>,
}

impl<Q: OpQueue> Plugin for OpQueuePlugin<Q> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Q>()
            .init_resource::<OpLoader<Q>>()
            .add_systems(Update, sys_perform_ops::<Q>);
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
pub struct OpRegistrar<'a, O: Op + TypePath + FromReflect>(&'a mut World, PhantomData<O>);

impl<'a, O: Op + TypePath + FromReflect> OpRegistrar<'a, O> {
    pub fn register_op<M: 'static, S>(&mut self, op_sys: S) -> &mut Self
    where
        S: SystemParamFunction<M, In = O, Out = Result<Metadata, OpError>>,
    {
        let sys_id = self.0.register_system(wrap_op_system(op_sys));
        let mut op_reg = self.0.get_resource_or_insert_with(OpRegistry::default);
        op_reg.add_op_system::<O>(sys_id);
        self
    }
}

#[derive(Debug, Resource)]
struct OpLoader<Q: OpQueue>(Option<Box<dyn Op<Queue = Q>>>);

impl<Q: OpQueue> Default for OpLoader<Q> {
    fn default() -> Self {
        OpLoader(None)
    }
}

fn wrap_op_system<S, M, O>(
    mut op_sys: S,
) -> impl FnMut(ResMut<OpLoader<O::Queue>>, StaticSystemParam<S::Param>, EventWriter<OpResult<O>>)
where
    S: SystemParamFunction<M, In = O, Out = Result<Metadata, OpError>>,
    O: Op + FromReflect,
{
    move |mut dbr, param, mut evw| {
        if let Some(next_op) = dbr.0.take() {
            let source = next_op.into_reflect();
            // let m: O = (*source.clone_value().into();
            let op: O =
                FromReflect::from_reflect(&*source.clone_value()).expect("Unwrap should be good?");
            // It would be nice if we could pass a reference of Op to the system instead, but that isn't working
            let result = op_sys.run(op, param.into_inner());
            let res = OpResult {
                source: *source.downcast().unwrap(),
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
                let sys_id = op_registry.get_op_system(op.reflect_type_path(), op.system_index());
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

fn sys_register_op<O: Op + TypePath + FromReflect>(world: &mut World) {
    let op_reg: OpRegistrar<O> = OpRegistrar(world, PhantomData);
    O::register_systems(op_reg);
}

pub trait Op: std::fmt::Debug + Sync + Send + Reflect + 'static {
    type Queue: OpQueue;
    // type Queue; Multiple queues to be added later
    fn full_sys_index(&self) -> (&str, usize) {
        (self.reflect_type_path(), self.system_index())
    }

    fn system_index(&self) -> usize;
    fn register_systems(registrar: OpRegistrar<Self>)
    where
        Self: Sized + TypePath + FromReflect;
}

pub trait OpQueue:
    core::ops::DerefMut
    + core::ops::Deref<Target = VecDeque<Box<dyn Op<Queue = Self>>>>
    + Resource
    + FromWorld
{
}
impl<
        T: core::ops::DerefMut
            + core::ops::Deref<Target = VecDeque<Box<dyn Op<Queue = Self>>>>
            + Resource
            + FromWorld,
    > OpQueue for T
{
}

#[derive(Debug, Error)]
pub enum OpError {
    /// This error usually means a user or system was trying to perform an OP but it is unsuccessful
    #[error("Invalid op: {0}")]
    InvalidOp(String),
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
    pub source: O,
    #[getset(get = "pub")]
    pub result: Result<Metadata, OpError>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Default, Deref, DerefMut, Resource)]
    pub struct ExampleQueue(VecDeque<Box<dyn Op<Queue = Self>>>);

    #[derive(Debug, Default, Reflect, PartialEq)]
    pub enum ExampleOp {
        #[default]
        ExampleOne,
        ExampleTwo,
    }

    impl Op for ExampleOp {
        type Queue = ExampleQueue;
        fn register_systems(mut registrar: OpRegistrar<Self>) {
            registrar
                .register_op(|In(op): In<ExampleOp>| {
                    println!("Hello from One: {op:?}!");
                    Ok(Metadata::default())
                })
                .register_op(|In(op), mut count: Local<usize>| {
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
            res.push_back(Box::new(ExampleOp::ExampleOne));
            res.push_back(Box::new(ExampleOp::ExampleTwo));
            res.push_back(Box::new(ExampleOp::ExampleTwo));
            res.push_back(Box::new(ExampleOp::ExampleTwo));
        })
        .add_systems(PostUpdate, |mut evr: EventReader<OpResult<ExampleOp>>| {
            let results: Vec<_> = evr.read().collect();
            assert!(
                matches!(
                    results[0],
                    OpResult {
                        source: ExampleOp::ExampleOne,
                        result: Ok(_)
                    }
                ),
                "Result one should match"
            );
            assert!(
                matches!(
                    results[1],
                    OpResult {
                        source: ExampleOp::ExampleTwo,
                        result: Ok(_)
                    }
                ),
                "Result two should match"
            );
            assert!(
                matches!(
                    results[2],
                    OpResult {
                        source: ExampleOp::ExampleTwo,
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
                        source: ExampleOp::ExampleTwo,
                        result: Ok(_)
                    }
                ),
                "Result four should match"
            );
        });
        app.update();
    }
}
