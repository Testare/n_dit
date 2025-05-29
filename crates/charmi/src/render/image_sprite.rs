use bevy::asset::{Assets, Handle};
use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::{
    AsBindGroup, BindGroup, CachedComputePipelineId, ComputePass, ComputePipelineDescriptor,
    PipelineCache,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::ShaderStorageBuffer;
use bevy::render::{Render, RenderApp, RenderSet};

use crate::{
    CharmiBindGroupLayouts, CharmiFunction, CharmiFunctionError, CharmiFunctionId, CharmiFunctions,
    CharmiImage, CharmiPhase, CharmiPhaseItem, TransformCh, ViewCh,
};

#[derive(Debug)]
pub struct ImageSpritePlugin;

impl Plugin for ImageSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<CharmiImageSpriteBuffer>::default())
            .add_systems(PostUpdate, sys_image_sprite_update_buffer);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<DrawImageFunction>().add_systems(
            Render,
            (
                rsys_queue_charmi_image_sprites.in_set(RenderSet::Queue),
                rsys_prepare_image_sprite_bind_groups.in_set(RenderSet::PrepareBindGroups),
            ),
        );
    }
}

#[derive(Clone, Component, Debug, Deref, DerefMut, ExtractComponent, AsBindGroup, TypePath)]
pub struct CharmiImageSprite {
    pub image: CharmiImage,
}

#[derive(Clone, Component, Debug, Deref, DerefMut, ExtractComponent, AsBindGroup, TypePath)]
pub(crate) struct CharmiImageSpriteBuffer {
    #[storage(0, visibility(compute), read_only)]
    pub buffer: Handle<ShaderStorageBuffer>,
}

#[derive(Clone, Component, Debug, Deref, DerefMut, ExtractComponent)]
struct CharmiImageSpriteBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Clone, Debug, Resource)]
pub struct DrawImageFunction {
    function_id: CharmiFunctionId,
    pipeline: CachedComputePipelineId,
}

impl DrawImageFunction {
    pub fn function_id(&self) -> CharmiFunctionId {
        self.function_id
    }
}

impl FromWorld for DrawImageFunction {
    fn from_world(world: &mut World) -> Self {
        world.init_resource::<CharmiBindGroupLayouts>();
        let charmi_layouts = world.resource::<CharmiBindGroupLayouts>();

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("DrawImageFunction pipeline".into()),
            layout: vec![
                charmi_layouts.view_layout.clone(),
                charmi_layouts.sprite_layout.clone(),
                charmi_layouts.image_sprite_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader: super::CHARMI_IMAGE_SPRITE_OPAQUE_SHADER_HANDLE,
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });

        let function = Self {
            function_id: CharmiFunctionId::new(),
            pipeline,
        };
        let mut charmi_functions = world.get_resource_or_init::<CharmiFunctions>();
        charmi_functions.insert(function.function_id, Box::new(function.clone()));
        function
    }
}

impl CharmiFunction for DrawImageFunction {
    fn draw(
        &self,
        world: &World,
        pass: &mut ComputePass,
        view: Entity,
        item: &CharmiPhaseItem,
    ) -> Result<(), CharmiFunctionError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = self.pipeline;
        let Some(image_sprite_bind_group) =
            world.entity(item.id).get::<CharmiImageSpriteBindGroup>()
        else {
            return Err(CharmiFunctionError);
        };
        let Some(view_ch) = world.entity(view).get::<ViewCh>() else {
            return Err(CharmiFunctionError);
        };
        let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline) else {
            return Err(CharmiFunctionError);
        };
        pass.set_bind_group(2, &image_sprite_bind_group.bind_group, &[]);
        pass.set_pipeline(init_pipeline);
        // TODO calculate clip and dispatch workgroups based on item size?
        pass.dispatch_workgroups(view_ch.buffer_len(), 1, 1);
        Ok(())
    }
}

fn sys_image_sprite_update_buffer(
    mut commands: Commands,
    mut ast_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    q_image_sprite: Query<
        (Entity, &CharmiImageSprite, Option<&CharmiImageSpriteBuffer>),
        Changed<CharmiImageSprite>,
    >,
) {
    for (id, image_sprite, buffer) in q_image_sprite.iter() {
        if let Some(buffer) = buffer {
            let Some(buffer) = ast_buffers.get_mut(&**buffer) else {
                log::error!("Cannot find ShaderStorageBuffer asset for {id:?}");
                continue;
            };
            buffer.set_data(image_sprite.image.clone());
        } else {
            let buffer = ShaderStorageBuffer::from(image_sprite.image.clone());
            let buffer = ast_buffers.add(buffer);
            commands
                .entity(id)
                .insert(CharmiImageSpriteBuffer { buffer });
        }
    }
}

fn rsys_queue_charmi_image_sprites(
    q_charmi_material_rect: Query<(Entity, &TransformCh), With<CharmiImageSpriteBindGroup>>,
    mut res_charmi_phase: ResMut<CharmiPhase>,
    res_draw_image: Res<DrawImageFunction>,
) {
    for (id, transform) in q_charmi_material_rect.iter() {
        res_charmi_phase.push(CharmiPhaseItem {
            z: transform.position.z,
            asset_id: None,
            function: res_draw_image.function_id(),
            id,
        });
    }
}

fn rsys_prepare_image_sprite_bind_groups(
    mut commands: Commands,
    charmi_layouts: Res<CharmiBindGroupLayouts>,
    render_device: Res<RenderDevice>,
    mut q_image_sprite: Query<
        (
            Entity,
            &CharmiImageSpriteBuffer,
            Option<&mut CharmiImageSpriteBindGroup>,
        ),
        Changed<CharmiImageSpriteBuffer>,
    >,
    sprite_param: StaticSystemParam<<CharmiImageSprite as AsBindGroup>::Param>,
) {
    let mut sprite_param = sprite_param.into_inner();
    for (id, image_sprite_buffer, bind_group_component) in q_image_sprite.iter_mut() {
        let bind_group = image_sprite_buffer
            .as_bind_group(
                &charmi_layouts.image_sprite_layout,
                &render_device,
                &mut sprite_param,
            )
            .expect("Error creating bind group")
            .bind_group;

        if let Some(mut bind_group_component) = bind_group_component {
            bind_group_component.bind_group = bind_group;
        } else {
            commands
                .entity(id)
                .insert(CharmiImageSpriteBindGroup { bind_group });
        }
    }
}
