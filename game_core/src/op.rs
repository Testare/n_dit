mod executor;

use std::marker::PhantomData;

use bevy::ecs::system::{StaticSystemParam, SystemId};
use bevy::reflect::TypePath;
pub use executor::{OpExecutor, OpExecutorPlugin, OpExecutorResource};
use thiserror::Error;

use crate::prelude::*;

// TODO refactor most of this into a module "op_sys" which will be more generic,
// then the more game-specific stuff defined somewhere else
#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct CoreOps(OpExecutor);

pub trait OpErrorUtils {
    type Error;
    fn critical(self) -> Self::Error;
    fn invalid(self) -> Self::Error;
}

impl OpErrorUtils for &str {
    type Error = OpError;
    fn critical(self) -> OpError {
        OpError::OpFailureCritical(anyhow::anyhow!(self.to_string()))
    }
    fn invalid(self) -> Self::Error {
        OpError::InvalidOp(self.to_string())
    }
}

impl<T, E: std::error::Error + Send + Sync + 'static> OpErrorUtils for Result<T, E> {
    type Error = Result<T, OpError>;
    fn critical(self) -> Result<T, OpError> {
        self.map_err(|e| OpError::OpFailureCritical(anyhow::Error::from(e)))
    }

    fn invalid(self) -> Result<T, OpError> {
        self.map_err(|e| OpError::InvalidOp(e.to_string()))
    }
}

/// A type alias for Result<Metadata, OpError>, different
/// from OpResult because OpResult also contains a copy of the
/// Op.
pub type OpImplResult = Result<Metadata, OpError>;

#[derive(Debug)]
pub struct OpPlugin<T: Op + TypePath + FromReflect>(PhantomData<T>);

impl<T: Op + TypePath + FromReflect> Default for OpPlugin<T> {
    fn default() -> Self {
        OpPlugin(default())
    }
}

impl<T: Op + TypePath + FromReflect> Plugin for OpPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_register_op::<T>)
            .add_event::<OpResult<T>>();
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
    pub fn register_op<M: 'static, S>(&mut self, opsys: S) -> &mut Self
    where
        S: SystemParamFunction<M, In = (Entity, O), Out = Result<Metadata, OpError>>,
    {
        let sys_id = self.0.register_system(wrap_op_system(opsys));
        let mut op_reg = self.0.get_resource_or_insert_with(OpRegistry::default);
        op_reg.add_op_system::<O>(sys_id);
        self
    }
}

#[derive(Debug, Default, Resource)]
struct OpLoader(Option<OpRequest>);

fn wrap_op_system<S, M, O>(
    mut op_sys: S,
) -> impl FnMut(ResMut<OpLoader>, StaticSystemParam<S::Param>, EventWriter<OpResult<O>>)
where
    S: SystemParamFunction<M, In = (Entity, O), Out = Result<Metadata, OpError>>,
    O: Op + FromReflect,
{
    move |mut op_loader, param, mut evw| {
        if let Some(op_request) = op_loader.0.take() {
            let OpRequest { op, source, .. } = op_request;
            let reflect_op = op.into_reflect();
            let op: O = FromReflect::from_reflect(&*reflect_op.clone_value())
                .expect("Unwrap should be good?");
            // It would be nice if we could pass a reference of Op to the system instead, but that isn't working
            let result = op_sys.run((source, op), param.into_inner());
            let res = OpResult {
                source,
                op: *reflect_op.downcast().unwrap(),
                result,
            };
            evw.send(res);
        }
    }
}

fn sys_register_op<O: Op + TypePath + FromReflect>(world: &mut World) {
    let op_reg: OpRegistrar<O> = OpRegistrar(world, PhantomData);
    O::register_systems(op_reg);
}

#[derive(Debug)]
pub struct OpRequest {
    source: Entity,
    op: Box<dyn Op>,
}

impl OpRequest {
    fn new<O: Op>(source: Entity, op: O) -> Self {
        OpRequest {
            source,
            op: Box::new(op),
        }
    }
}

pub trait Op: std::fmt::Debug + Sync + Send + Reflect + 'static {
    fn system_index(&self) -> usize;
    fn register_systems(registrar: OpRegistrar<Self>)
    where
        Self: Sized + TypePath + FromReflect;

    fn to_request(self, source: Entity) -> OpRequest
    where
        Self: Sized,
    {
        OpRequest {
            source,
            op: Box::new(self),
        }
    }
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

#[derive(Debug, Event, getset::CopyGetters, getset::Getters)]
pub struct OpResult<O> {
    #[getset(get_copy = "pub")]
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
    pub struct ExampleExecutor(OpExecutor);

    #[derive(Debug, Default, Reflect, PartialEq)]
    pub enum ExampleOp {
        #[default]
        ExampleOne,
        ExampleTwo,
    }

    impl Op for ExampleOp {
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
            OpExecutorPlugin::<ExampleExecutor>::default(),
            OpPlugin::<ExampleOp>::default(),
        ))
        .add_systems(Startup, |mut ops: ResMut<ExampleExecutor>| {
            ops.request(Entity::PLACEHOLDER, ExampleOp::ExampleOne);
            ops.request(Entity::PLACEHOLDER, ExampleOp::ExampleTwo);
            ops.request(Entity::PLACEHOLDER, ExampleOp::ExampleTwo);
            ops.request(Entity::PLACEHOLDER, ExampleOp::ExampleTwo);
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
