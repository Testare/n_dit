use bevy::{
    asset::io::embedded::GetAssetServer, render::render_resource::{AsBindGroup, ShaderRef}
};
use charmi::{MaterialCh, MaterialChPlugin};
use game_core::op::tutorial::TutorialIndication;

use crate::prelude::*;

#[derive(Debug)]
pub struct TutorialUiPlugin;

impl Plugin for TutorialUiPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(MaterialChPlugin::<TutorialIndicator>::default())
            // .init_resource::<TheTutorialIndicatorHandle>()
            // app.add_systems(PreUpdate, sys_add_tutorial_commands_to_player)
            app.add_systems(Update, (
                    sys_add_tutorial_ui_flags_on_tutorial_entered, 
                   sys_react_to_tutorial_indication_events))
            ;
    }
}

// This should not be a resource - There can be multiple players in a tutorial indicator
#[derive(Resource)]
struct TheTutorialIndicatorHandle(Handle<TutorialIndicator>);

impl FromWorld for TheTutorialIndicatorHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_asset_server();
        Self(asset_server.add(TutorialIndicator::default()))
    }
}

#[derive(AsBindGroup, Asset, Clone, Debug, Default, TypePath)]
struct TutorialIndicator {
    enabled: bool,
    pos: UVec2,
    size: UVec2,
}

impl MaterialCh for TutorialIndicator {
    fn shader() -> ShaderRef {
        "shaders/tutorial_indicator.wgsl".into()
    }
}

/// TODO: When a player enters a tutorial node, attach `TutorialId(name, tutorial_entity)` to the
/// relevant UI entities (access points, curios, etc.) so that `tutorial_indicate` yarn commands
/// can resolve them at runtime. The tutorial entity comes from `InTutorial` on the player.
/// This should probably live in NF rather than here.
fn sys_add_tutorial_ui_flags_on_tutorial_entered() {
}

fn sys_react_to_tutorial_indication_events(
    mut evr: EventReader<TutorialIndication>,
) {
    for TutorialIndication(id) in evr.read() {
        // Currently a stub
        log::debug!("Received tutorial indication for {id:?}")
    }
}

