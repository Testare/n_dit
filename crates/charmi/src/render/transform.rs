use bevy::math::{IVec3, UVec2};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::{BufferUsages, DynamicUniformBuffer, ShaderType};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::{Render, RenderApp, RenderSet};

pub struct CharmiTransformPlugin;

impl Plugin for CharmiTransformPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<TransformCh>::default());
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<TransformChUniforms>()
            .add_systems(
                Render,
                rsys_update_transforms.in_set(RenderSet::PrepareResources),
            );
    }
}

#[derive(Clone, Component, Debug, Default, ExtractComponent, ShaderType)]
#[repr(C)]
pub struct TransformCh {
    pub position: IVec3,
    // TODO BEFOREMERGE separate this out? For things like images or unbounded items a scale is not relevant or should be defaulted
    pub scale: UVec2,
}

/// Render resource -
#[derive(Resource, Deref, DerefMut)]
pub struct TransformChUniforms {
    uniforms: DynamicUniformBuffer<TransformCh>,
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct TransformChOffset(pub u32);

impl FromWorld for TransformChUniforms {
    fn from_world(world: &mut World) -> Self {
        let mut uniforms = DynamicUniformBuffer::default();
        uniforms.set_label(Some("transform_uniforms_buffer"));
        uniforms.add_usages(BufferUsages::STORAGE);
        let device = world.resource::<RenderDevice>();
        let queue = world.resource::<RenderQueue>();
        {
            let mut writer = uniforms
                .get_writer(2, device, queue)
                .expect("should work (Proper error handling TODO)");
            writer.write(&TransformCh::default());
        }

        Self { uniforms }
    }
}

fn rsys_update_transforms(
    mut commands: Commands,
    mut q_transform: Query<(Entity, &TransformCh, Option<&mut TransformChOffset>)>,
    mut res_transform_uniform: ResMut<TransformChUniforms>,
    res_render_device: Res<RenderDevice>,
    res_render_queue: Res<RenderQueue>,
) {
    let count = q_transform.iter().len() + 1;
    res_transform_uniform.clear();
    let mut writer = res_transform_uniform
        .get_writer(count, res_render_device.as_ref(), res_render_queue.as_ref())
        .unwrap();
    writer.write(&TransformCh::default());
    for (id, transform, offset_component) in q_transform.iter_mut() {
        let offset = writer.write(transform);
        if let Some(mut offset_component) = offset_component {
            **offset_component = offset;
        } else {
            commands.entity(id).insert(TransformChOffset(offset));
        }
    }
}
