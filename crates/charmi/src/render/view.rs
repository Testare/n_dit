use bevy::asset::{Assets, Handle};
use bevy::math::UVec2;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::globals::GlobalsBuffer;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{BindGroup, BindGroupEntries, BufferUsages};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use bevy::render::{Render, RenderApp, RenderSystems};

use crate::{CharmiBindGroupLayouts, CharmiImage, TransformCh, TransformChUniforms};

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ViewCh>::default());
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            rsys_prepare_view_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
    }
}

/// Represents the MainView, the one rendered to the terminal
/// In the future, will probably simply make a "ViewOutput(Box<dyn Write>)"
/// component or something like that
#[derive(Component, Debug)]
pub struct MainView;

/// Represents a Charmi "view"
#[derive(Clone, Component, Debug, ExtractComponent)]
#[require(TransformCh)]
pub struct ViewCh {
    order: usize,
    size: UVec2,
    buffer: Handle<ShaderStorageBuffer>,
}

impl ViewCh {
    pub fn new(order: usize, size: UVec2, ast_buffers: &mut Assets<ShaderStorageBuffer>) -> Self {
        let buffer = CharmiImage::new_empty(size.x, size.y);
        let mut buffer = ShaderStorageBuffer::from(buffer);
        // We need to enable the COPY_SRC usage so we can copy the buffer to the cpu
        buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
        let buffer = ast_buffers.add(buffer);
        Self {
            order,
            size,
            buffer,
        }
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
