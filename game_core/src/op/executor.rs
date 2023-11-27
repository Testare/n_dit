use std::marker::PhantomData;

use bevy::ecs::schedule::ScheduleLabel;

use super::{Op, OpLoader, OpRegistry, OpRequest};
use crate::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct UnspecifiedSystemSet;

#[derive(Debug)]
pub struct OpExecutorPlugin<E, S = UnspecifiedSystemSet, L: ScheduleLabel + Clone = Update> {
    phantom_data: PhantomData<(E, S)>,
    schedule: L,
    system_set: Option<S>,
}

impl<E, S> Default for OpExecutorPlugin<E, S, Update> {
    fn default() -> Self {
        Self {
            phantom_data: PhantomData,
            schedule: Update,
            system_set: None,
        }
    }
}

impl<E, S, L: ScheduleLabel + Clone> OpExecutorPlugin<E, S, L> {
    pub fn new<S2, L2: ScheduleLabel + Clone>(
        schedule: L2,
        system_set: Option<S2>,
    ) -> OpExecutorPlugin<E, S2, L2> {
        OpExecutorPlugin {
            phantom_data: PhantomData,
            schedule,
            system_set,
        }
    }
}

impl<E, L: ScheduleLabel + Clone, S: SystemSet + Clone> Plugin for OpExecutorPlugin<E, S, L>
where
    E: Resource + Default + std::ops::DerefMut + std::ops::Deref<Target = OpExecutor>,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<E>().init_resource::<OpLoader>();
        if let Some(ref system_set) = self.system_set {
            app.add_systems(
                self.schedule.clone(),
                sys_perform_ops::<E>.in_set(system_set.clone()),
            );
        } else {
            app.add_systems(self.schedule.clone(), sys_perform_ops::<E>);
        }
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
    pub fn request<O: Op>(&mut self, source: Entity, op: O) {
        match self {
            Self::Local(ref mut queue) => queue.push(OpRequest::new(source, op)),
            Self::Network => todo!("TODO network support"),
        }
    }

    pub fn accept_requests<E: Iterator<Item = OpRequest>>(&mut self, events: E) {
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
                // In the future, might only implement X ops per frame
                std::mem::replace(queue, new_queue)
            },
            Self::Network => todo!("TODO Network support"),
        }
    }
}

pub fn sys_perform_ops<E: Resource + std::ops::DerefMut + std::ops::Deref<Target = OpExecutor>>(
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
