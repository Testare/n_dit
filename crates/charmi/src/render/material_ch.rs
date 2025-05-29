use std::marker::PhantomData;

use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets};
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroup, BindGroupLayout, CachedComputePipelineId,
    ComputePass, ComputePipelineDescriptor, PipelineCache, ShaderRef,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::{Render, RenderApp, RenderSet};

use crate::{
    CharmiBindGroupLayouts, CharmiFunction, CharmiFunctionError, CharmiFunctionId, CharmiFunctions,
    CharmiPhase, CharmiPhaseItem, TransformCh, ViewCh,
};

// TODO BEFOREMERGE rename to MaterialSpriteCh
#[derive(Clone, Component, Debug, ExtractComponent)]
pub struct MaterialPaneCh<M: MaterialCh>(pub Handle<M>);

pub trait MaterialCh: Sized + Asset + Clone + AsBindGroup {
    fn shader() -> ShaderRef;
}

#[derive(Clone, Copy, Debug)]
pub struct MaterialChPlugin<M: MaterialCh>(PhantomData<M>);

impl<M: MaterialCh> Default for MaterialChPlugin<M> {
    fn default() -> Self {
        MaterialChPlugin(PhantomData::<M>)
    }
}

impl<M: MaterialCh> Plugin for MaterialChPlugin<M> {
    fn build(&self, app: &mut App) {
        app.init_asset::<M>().add_plugins((
            RenderAssetPlugin::<MaterialChBindGroup<M>>::default(),
            ExtractComponentPlugin::<MaterialPaneCh<M>>::default(),
        ));
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<DrawMaterialFunction<M>>()
            .init_resource::<MaterialChPipeline<M>>()
            .add_systems(
                Render,
                rsys_queue_charmi_material_sprites::<M>.in_set(RenderSet::Queue),
            );
    }
}

#[derive(Resource)]
pub struct MaterialChPipeline<M: MaterialCh> {
    pub material_layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
    _phantom_data: PhantomData<M>,
}

impl<M: MaterialCh> FromWorld for MaterialChPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        world.init_resource::<CharmiBindGroupLayouts>();
        let render_device = world.resource::<RenderDevice>();
        let charmi_layouts = world.resource::<CharmiBindGroupLayouts>();
        let material_layout = M::bind_group_layout(render_device);

        let shader = match M::shader() {
            ShaderRef::Default => world.load_asset(""), // TODO BEFOREMERGE better default, probably just grey boxes
            ShaderRef::Path(path) => world.load_asset(path),
            ShaderRef::Handle(handle) => handle.clone(),
        };
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(format!("MaterialChPipeline[{}]", M::type_path()).into()),
            layout: vec![
                charmi_layouts.view_layout.clone(),
                charmi_layouts.sprite_layout.clone(),
                material_layout.clone(),
            ],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: Vec::new(),
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });
        MaterialChPipeline {
            material_layout,
            pipeline,
            _phantom_data: default(),
        }
    }
}

#[derive(Debug, Deref)]
pub struct MaterialChBindGroup<M: MaterialCh>(#[deref] BindGroup, PhantomData<M>);

impl<M: MaterialCh> RenderAsset for MaterialChBindGroup<M> {
    type SourceAsset = M;

    type Param = (SRes<MaterialChPipeline<M>>, SRes<RenderDevice>, M::Param);

    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _asset_id: AssetId<Self::SourceAsset>,
        param: &mut SystemParamItem<Self::Param>,
    ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        let (pipeline, render_device, ref mut mat_param) = param;

        match source_asset.as_bind_group(
            &pipeline.material_layout,
            render_device.as_ref(),
            mat_param,
        ) {
            Ok(prepared_bind_group) => Ok(Self(prepared_bind_group.bind_group, PhantomData::<M>)),
            Err(AsBindGroupError::RetryNextUpdate) => {
                Err(PrepareAssetError::RetryNextUpdate(source_asset))
            },
            Err(e) => Err(PrepareAssetError::AsBindGroupError(e)),
        }
    }
}

fn rsys_queue_charmi_material_sprites<M: MaterialCh>(
    q_charmi_material_rect: Query<(Entity, &MaterialPaneCh<M>, &TransformCh)>,
    mut res_charmi_phase: ResMut<CharmiPhase>,
    res_material: Res<DrawMaterialFunction<M>>,
) {
    for (id, rect, transform) in q_charmi_material_rect.iter() {
        res_charmi_phase.push(CharmiPhaseItem {
            z: transform.position.z,
            asset_id: Some(rect.0.id().untyped()),
            function: res_material.function_id(),
            id,
        });
    }
}

// Should probably be moved to material_ch.rs
#[derive(Clone, Debug, Resource)]
pub struct DrawMaterialFunction<M: MaterialCh> {
    function_id: CharmiFunctionId,
    phantom: PhantomData<M>,
}

impl<M: MaterialCh> DrawMaterialFunction<M> {
    pub fn function_id(&self) -> CharmiFunctionId {
        self.function_id
    }
}

impl<M: MaterialCh> FromWorld for DrawMaterialFunction<M> {
    fn from_world(world: &mut World) -> Self {
        let mut charmi_functions = world.get_resource_or_init::<CharmiFunctions>();
        let function_id = CharmiFunctionId::new();
        let function = Self {
            function_id,
            phantom: PhantomData::<M>,
        };
        charmi_functions.insert(function_id, Box::new(function.clone()));
        function
    }
}

impl<M: MaterialCh> CharmiFunction for DrawMaterialFunction<M> {
    fn draw(
        &self,
        world: &World,
        pass: &mut ComputePass,
        view: Entity,
        item: &CharmiPhaseItem,
    ) -> Result<(), CharmiFunctionError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<MaterialChPipeline<M>>();
        let rast_mat_bind_group = world.resource::<RenderAssets<MaterialChBindGroup<M>>>();

        let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) else {
            return Ok(());
        };
        let Some(view_ch) = world.entity(view).get::<ViewCh>() else {
            return Err(CharmiFunctionError);
        };
        let Some(Ok(asset_id)) = item
            .asset_id
            .as_ref()
            .map(|asset_id| asset_id.try_typed::<M>())
        else {
            return Err(CharmiFunctionError);
        };
        let Some(mat_bind_group) = rast_mat_bind_group.get(asset_id) else {
            return Err(CharmiFunctionError);
        };
        pass.set_bind_group(2, &**mat_bind_group, &[]);
        pass.set_pipeline(init_pipeline);
        // TODO dispatch workgroups based on item size?
        pass.dispatch_workgroups(view_ch.buffer_len(), 1, 1);
        Ok(())
    }
}
