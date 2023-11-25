use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;

use super::{OpLoader, OpRegistry, OpRequest, OpV2};
use crate::prelude::*;

#[derive(Debug)]
pub struct OpExecutorPlugin<E, S: ScheduleLabel + Clone = Update> {
    phantom_data: PhantomData<E>,
    schedule: S,
}

impl<E> Default for OpExecutorPlugin<E, Update> {
    fn default() -> Self {
        Self {
            phantom_data: PhantomData,
            schedule: Update,
        }
    }
}

impl<E, K: ScheduleLabel + Clone> Plugin for OpExecutorPlugin<E, K>
where
    E: Resource + Default + std::ops::DerefMut + std::ops::Deref<Target = OpExecutor>,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<E>()
            .init_resource::<OpLoader>()
            .add_systems(self.schedule.clone(), sys_perform_ops::<E>);
    }
}

#[derive(Debug)]
pub enum OpExecutor {
    Local(Vec<OpRequest>),
    Network,
}

impl Default for OpExecutor {
    fn default() -> Self {
        OpExecutor::Local(Vec::new())
    }
}

impl OpExecutor {
    pub fn request<O: OpV2>(&mut self, source: Entity, op: O) {
        match self {
            Self::Local(ref mut queue) => queue.push(OpRequest::new(source, op)),
            Self::Network => todo!("TODO network support"),
        }
    }

    pub fn accept_events<E: Iterator<Item = OpRequest>>(&mut self, events: E) {
        match self {
            Self::Local(ref mut queue) => {
                queue.extend(events);
            },
            Self::Network => todo!("TODO Network support"),
        }
    }

    pub fn take_ops(&mut self) -> Vec<OpRequest> {
        match self {
            Self::Local(ref mut queue) => {
                let new_queue = Vec::new();
                std::mem::replace(queue, new_queue)
            },
            Self::Network => todo!("TODO Network support"),
        }
    }
}

fn sys_perform_ops<E: Resource + std::ops::DerefMut + std::ops::Deref<Target = OpExecutor>>(
    world: &mut World,
) {
    // TODO make resource to get list of Ops to perform
    let ops = world
        .get_resource_mut::<E>()
        .expect("Can't find executor")
        .take_ops();

    let ops = world.get_resource::<OpRegistry>().map(|op_registry| {
        ops.into_iter()
            .map(|op| {
                let sys_id =
                    op_registry.get_op_system(op.op.reflect_type_path(), op.op.system_index());
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
                    .get_resource_mut::<OpLoader>()
                    .expect("OpLoader for OpQueue should be initialized")
                    .0 = Some(op_data);
                if let Err(e) = world.run_system(op_sys) {
                    log::error!("Error running op system [{op_sys:?}]: {e:?}");
                }
            },
        }
    }
}
