use bevy::asset::{Assets, Handle};
use bevy::math::UVec2;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::globals::GlobalsBuffer;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    BindGroup, BindGroupEntries, BufferUsages, CachedComputePipelineId, ComputePipelineDescriptor,
    PipelineCache,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use bevy::render::sync_world::RenderEntity;
use bevy::render::{Extract, Render, RenderApp, RenderSystems};

use crate::{CharCell, CharmiBindGroupLayouts, CharmiImage, TransformCh, TransformChUniforms};

use super::CharmiPhase;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExtractComponentPlugin::<ViewCh>::default(),));
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ViewClearPipeline>()
            .add_systems(
                Render,
                rsys_prepare_view_bind_groups.in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(ExtractSchedule, rsys_on_view_changed);
    }
}

/// Represents the MainView, the one rendered to the terminal
/// In the future, will probably simply make a "ViewOutput(Box<dyn Write>)"
/// component or something like that
#[derive(Component, Debug)]
pub struct MainView;

/// Represents a Charmi "view"
#[derive(Clone, Component, Debug, ExtractComponent)]
#[require(TransformCh, CharmiPhase)]
pub struct ViewCh {
    order: usize,
    size: UVec2,
    buffer: Handle<ShaderStorageBuffer>,
}

impl ViewCh {
    pub fn resize(&mut self, size: UVec2, ast_buffers: &mut Assets<ShaderStorageBuffer>) {
        self.size = size;
        Self::create_buffer(size, ast_buffers, &self.buffer);
    }

    pub fn new(order: usize, size: UVec2, ast_buffers: &mut Assets<ShaderStorageBuffer>) -> Self {
        let buffer = ast_buffers.reserve_handle();
        Self::create_buffer(size, ast_buffers, &buffer);
        Self {
            order,
            size,
            buffer,
        }
    }

    fn create_buffer(
        size: UVec2,
        ast_buffers: &mut Assets<ShaderStorageBuffer>,
        handle: &Handle<ShaderStorageBuffer>,
    ) {
        let buffer = CharmiImage::new_fill(
            size.x,
            size.y,
            CharCell {
                ch: ' ' as u32,
                fg: 15,
                bg: 0,
                attr: 0,
            },
        );
        let mut buffer = ShaderStorageBuffer::from(buffer);
        // We need to enable the COPY_SRC usage so we can copy the buffer to the cpu
        buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        ast_buffers.insert(handle.id(), buffer);
    }

    pub fn buffer(&self) -> &Handle<ShaderStorageBuffer> {
        &self.buffer
    }

    pub fn buffer_len(&self) -> u32 {
        self.size.x * self.size.y
    }

    pub fn order(&self) -> usize {
        self.order
    }
}

#[derive(Clone, Component, ExtractComponent, Deref)]
pub struct ViewChBindGroup(pub BindGroup);

fn rsys_on_view_changed(
    mut commands: Commands,
    eq_changed_views: Extract<Query<RenderEntity, Changed<ViewCh>>>,
) {
    for id in eq_changed_views.iter() {
        commands.entity(id).remove::<ViewChBindGroup>();
    }
}

fn rsys_prepare_view_bind_groups(
    mut commands: Commands,
    charmi_layouts: Res<CharmiBindGroupLayouts>,
    render_device: Res<RenderDevice>,
    globals_buffer: Res<GlobalsBuffer>,
    transform_uniforms: Res<TransformChUniforms>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
    mut q_views: ParamSet<(
        Query<(Entity, &ViewCh), Without<ViewChBindGroup>>,
        Query<(Entity, &ViewCh)>,
    )>,
) {
    let mut construct_bind_group = |id: Entity, view: &ViewCh| {
        let buffer = buffers.get(view.buffer()).unwrap();
        let transform_binding = transform_uniforms.binding().unwrap();
        let globals_binding = globals_buffer.buffer.binding().unwrap();

        let bind_group = render_device.create_bind_group(
            "Charmi view bind group",
            &charmi_layouts.view_layout,
            &BindGroupEntries::sequential((
                buffer.buffer.as_entire_buffer_binding(),
                transform_binding,
                globals_binding,
            )),
        );
        commands.entity(id).insert(ViewChBindGroup(bind_group));
    };

    if transform_uniforms.buffer_recreated() {
        // Transform uniform
        for (id, view) in q_views.p1().iter() {
            construct_bind_group(id, view);
        }
    } else {
        for (id, view) in q_views.p0().iter() {
            construct_bind_group(id, view);
        }
    }
}

#[derive(Resource)]
pub struct ViewClearPipeline {
    pub pipeline: CachedComputePipelineId,
}

impl FromWorld for ViewClearPipeline {
    fn from_world(world: &mut World) -> Self {
        world.init_resource::<CharmiBindGroupLayouts>();
        let charmi_layouts = world.resource::<CharmiBindGroupLayouts>();

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("View Clear Pipeline".into()),
            layout: vec![charmi_layouts.view_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: super::CHARMI_VIEW_CLEAR_HANDLE,
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });
        ViewClearPipeline { pipeline }
    }
}
