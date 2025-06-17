use std::collections::HashMap;
use std::fmt::Debug;

use bevy::asset::UntypedAssetId;
use bevy::prelude::*;
use bevy::render::globals::GlobalsBuffer;
use bevy::render::render_graph::{
    self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel,
};
use bevy::render::render_resource::{
    BindGroupEntries, ComputePass, ComputePassDescriptor, PipelineCache,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::{define_atomic_id, Render, RenderApp, RenderSystems};

use crate::{
    CharmiBindGroupLayouts, CharmiGlobalsBindGroup, TransformChOffset, TransformChUniforms, ViewCh,
    ViewChBindGroup,
};

use super::ViewClearPipeline;

pub struct CharmiRenderPipelinePlugin;
impl Plugin for CharmiRenderPipelinePlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<CharmiBindGroupLayouts>()
            .init_resource::<CharmiPhase>()
            .add_systems(
                Render,
                (
                    (rsys_prepare_charmi_globals_bind_group.run_if(
                        |transforms: Res<TransformChUniforms>| transforms.buffer_recreated(),
                    ),)
                        .in_set(RenderSystems::PrepareBindGroups),
                    rsys_clear_charmi_phase.in_set(RenderSystems::Cleanup),
                    rsys_sort_charmi_phase.in_set(RenderSystems::PhaseSort),
                ),
            );

        let world_mut = render_app.world_mut();
        let main_pass_ch_node = MainPassChNode::from_world(world_mut);
        let mut render_graph = world_mut.resource_mut::<RenderGraph>();
        render_graph.add_node(NodeCharmi::MainPassCh, main_pass_ch_node);
    }
}

#[allow(unused_variables, reason = "better default names when implemented")]
pub trait CharmiFunction: Send + Sync {
    fn draw(
        &self,
        world: &World,
        pass: &mut ComputePass,
        view: Entity,
        item: &CharmiPhaseItem,
    ) -> Result<(), CharmiFunctionError>;

    fn prepare(&mut self, world: &World) {}
}

// TODO better errors (replace with Bevy's Error type?)
#[derive(Debug)]
pub struct CharmiFunctionError;

define_atomic_id!(CharmiFunctionId);

#[derive(Default, Deref, DerefMut, Resource)]
pub struct CharmiFunctions(HashMap<CharmiFunctionId, Box<dyn CharmiFunction>>);

impl Debug for CharmiFunctions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CharmiFunctions")
            .field(&self.0.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut, Resource)]
pub struct CharmiPhase(Vec<CharmiPhaseItem>);

#[derive(Debug)]
pub struct CharmiPhaseItem {
    pub z: i32,
    pub asset_id: Option<UntypedAssetId>,
    pub function: CharmiFunctionId,
    pub id: Entity,
}

/// The node that will execute the compute shader
struct MainPassChNode {
    views: QueryState<(
        Entity,
        &'static ViewCh,
        &'static ViewChBindGroup,
        &'static CharmiPhase,
    )>,
}

impl FromWorld for MainPassChNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            views: world.query(),
        }
    }
}

impl render_graph::Node for MainPassChNode {
    fn update(&mut self, world: &mut World) {
        self.views.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let globals_bind_group = world.resource::<CharmiGlobalsBindGroup>();
        // let charmi_phase = world.resource::<CharmiPhase>();
        let charmi_functions = world.resource::<CharmiFunctions>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let view_clear_pipeline = world.resource::<ViewClearPipeline>();
        // TODO sort with view
        for (view_id, view, view_bind_group, charmi_phase) in self.views.iter_manual(world) {
            let view_offset = world
                .get::<TransformChOffset>(view_id)
                .map(|o| **o)
                .unwrap_or(0);

            if let Some(view_clear_pipeline) =
                pipeline_cache.get_compute_pipeline(view_clear_pipeline.pipeline)
            {
                // TODO Possibly make view clear into its own phase/node
                let mut clear_pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("Charmi compute pass"),
                            ..default()
                        });

                clear_pass.set_bind_group(0, &**view_bind_group, &[view_offset]);
                clear_pass.set_pipeline(view_clear_pipeline);
                clear_pass.dispatch_workgroups(view.buffer_len(), 1, 1);
            }

            for charmi_phase_item in charmi_phase.iter() {
                let Some(function) = charmi_functions.get(&charmi_phase_item.function) else {
                    continue;
                };
                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("Charmi compute pass"),
                            ..default()
                        });

                let sprite_offset = world
                    .get::<TransformChOffset>(charmi_phase_item.id)
                    .map(|o| **o)
                    .unwrap_or(0);
                pass.set_bind_group(0, &**view_bind_group, &[view_offset]);
                pass.set_bind_group(1, &globals_bind_group.0, &[sprite_offset]);
                if let Err(e) = function.draw(world, &mut pass, view_id, charmi_phase_item) {
                    log::error!("Charmi function error {e:?}");
                }
            }
        }
        Ok(())
    }
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
enum NodeCharmi {
    MainPassCh,
}

fn rsys_prepare_charmi_globals_bind_group(
    mut commands: Commands,
    charmi_layouts: Res<CharmiBindGroupLayouts>,
    render_device: Res<RenderDevice>,
    transform_uniforms: Res<TransformChUniforms>,
    globals_buffer: Res<GlobalsBuffer>,
) {
    let transform_binding = transform_uniforms.binding().unwrap();
    let globals_binding = globals_buffer.buffer.binding().unwrap();
    let bind_group = render_device.create_bind_group(
        "Charmi sprite bind group",
        &charmi_layouts.sprite_layout,
        &BindGroupEntries::sequential((transform_binding, globals_binding)),
    );
    commands.insert_resource(CharmiGlobalsBindGroup(bind_group));
}

fn rsys_clear_charmi_phase(
    mut res_charmi_phase: ResMut<CharmiPhase>,
    mut q_charmi_phase: Query<&mut CharmiPhase>,
) {
    for mut charmi_phase in q_charmi_phase.iter_mut() {
        charmi_phase.clear();
    }
    res_charmi_phase.clear();
}

fn rsys_sort_charmi_phase(
    mut res_charmi_phase: ResMut<CharmiPhase>,
    mut q_charmi_phase: Query<&mut CharmiPhase>,
) {
    for mut charmi_phase in q_charmi_phase.iter_mut() {
        charmi_phase.sort_by_key(|phase| phase.z);
    }
    res_charmi_phase.sort_by_key(|phase| phase.z);
}
