use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::sync_component::SyncComponentPlugin;
use bevy::render::sync_world::RenderEntity;
use bevy::render::{Extract, RenderApp};

use super::CharmiImageSprite;

#[derive(Clone, Component, Debug, Default, ExtractComponent)]
#[relationship_target(relationship=RenderedBy)]
pub struct RenderLayer(EntityHashSet);

#[derive(Clone, Component, Debug, Deref, DerefMut, ExtractComponent)]
pub struct ViewLayers(pub EntityHashSet);

#[derive(Clone, Component, Debug)]
#[component(immutable)]
pub struct TranslatedRenderLayer {
    entities: EntityHashSet,
}

impl TranslatedRenderLayer {
    pub fn entities(&self) -> &EntityHashSet {
        &self.entities
    }
}

#[derive(Clone, Copy, Component, Debug, Deref, ExtractComponent)]
#[relationship(relationship_target=RenderLayer)]
pub struct RenderedBy(pub Entity);

#[derive(Clone, Copy, Component, Debug)]
pub struct NotRenderedByDefault;

#[derive(Clone, Resource, Debug, Deref, ExtractResource)]
pub struct DefaultRenderLayer {
    #[deref]
    id: Entity,
    id_set: EntityHashSet,
}

#[derive(Debug)]
pub struct RenderLayerPlugin;

impl Plugin for RenderLayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            SyncComponentPlugin::<ViewLayers>::default(),
            SyncComponentPlugin::<RenderLayer>::default(),
        ))
        .add_systems(PostUpdate, sys_add_to_default_render_layer)
        .init_resource::<DefaultRenderLayer>(); // Must be done AFTER SyncComponentPlugin::<RenderLayer>::default()
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(ExtractSchedule, rsys_extract_render_layer);
    }
}

impl FromWorld for DefaultRenderLayer {
    fn from_world(world: &mut World) -> Self {
        let id = world.spawn(RenderLayer::default()).id();
        let id_set = [id].into_iter().collect();
        DefaultRenderLayer { id, id_set }
    }
}

impl DefaultRenderLayer {
    pub fn unwrap<'a: 'b + 'c, 'b: 'c, 'c>(
        &'a self,
        view_layer: Option<&'b ViewLayers>,
    ) -> &'c EntityHashSet {
        let ip: Option<&EntityHashSet> = view_layer.map(|view_layer| &view_layer.0);
        ip.unwrap_or(&self.id_set)
    }
}

fn sys_add_to_default_render_layer(
    mut commands: Commands,
    res_default_layer: Res<DefaultRenderLayer>,
    // TODO not just for CharmiImageSprites
    q_sprites_for_default_layer: Query<
        Entity,
        (
            With<CharmiImageSprite>,
            Without<NotRenderedByDefault>,
            Without<RenderedBy>,
        ),
    >,
) {
    let batch: Vec<_> = q_sprites_for_default_layer
        .iter()
        .map(|id| (id, RenderedBy(**res_default_layer)))
        .collect();

    if !batch.is_empty() {
        commands.try_insert_batch(batch);
    }
}

fn rsys_extract_render_layer(
    mut commands: Commands,
    eres_default_layer: Extract<Res<DefaultRenderLayer>>,
    res_default_layer: Option<Res<DefaultRenderLayer>>,
    eq_render_layers: Extract<Query<(RenderEntity, &RenderLayer), Changed<RenderLayer>>>,
    eq_render_entities: Extract<Query<RenderEntity>>, // TODO visibility flag to cull unrendered entities?
    eq_view_layers: Extract<Query<(RenderEntity, &ViewLayers), Changed<ViewLayers>>>,
) {
    // Not sure if this is necessary
    if res_default_layer.is_none() {
        let default_layer_id: Entity = ***eres_default_layer;
        let Ok(dl_rid) = eq_render_entities.get(default_layer_id) else {
            log::error!("Default layer not translated to render world");
            return;
        };
        let id_set = [dl_rid].into_iter().collect();
        commands.insert_resource(DefaultRenderLayer { id: dl_rid, id_set });
        log::debug!("Default Render Layer Extracted");
    };

    commands.try_insert_batch(
        eq_render_layers
            .iter()
            .map(|(rid, render_layer)| {
                (
                    rid,
                    TranslatedRenderLayer {
                        entities: render_layer
                            .iter()
                            .filter_map(|render_target| eq_render_entities.get(render_target).ok())
                            .collect(),
                    },
                )
            })
            .collect::<Vec<_>>(),
    );

    // TODO split into separate system?
    commands.try_insert_batch(
        eq_view_layers
            .iter()
            .map(|(rid, view_layers)| {
                (
                    rid,
                    ViewLayers(
                        view_layers
                            .iter()
                            .filter_map(|render_target| eq_render_entities.get(*render_target).ok())
                            .collect(),
                    ),
                )
            })
            .collect::<Vec<_>>(),
    );
}
