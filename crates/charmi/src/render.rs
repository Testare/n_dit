pub mod image_sprite;
pub mod material_ch;
pub mod pipeline;
pub mod transform;
pub mod view;

use std::fmt::Debug;

use bevy::app::{App, Plugin, TerminalCtrlCHandlerPlugin};
use bevy::asset::{load_internal_asset, weak_handle, AssetPlugin, Handle};
use bevy::diagnostic::FrameCountPlugin;
use bevy::prelude::{FromWorld, ImagePlugin, Resource, Shader, World};
use bevy::render::globals::GlobalsUniform;
use bevy::render::render_resource::binding_types::{storage_buffer, uniform_buffer};
use bevy::render::render_resource::{
    AsBindGroup, BindGroup, BindGroupLayout, BindGroupLayoutEntries, ShaderStages,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::RenderPlugin;
use bevy::time::TimePlugin;
use bevy::window::{ExitCondition, WindowPlugin};

pub use self::image_sprite::*;
pub use self::material_ch::*;
pub use self::pipeline::*;
pub use self::transform::*;
pub use self::view::*;
use crate::CharmiImage;

const CHARMI_SHADER_HANDLE: Handle<Shader> = weak_handle!("e0af1749-cc9f-47b7-9079-6273492ca90b");
const CHARMI_BOX_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("3dfe1740-d746-469c-9077-d3f688862bae");
const CHARMI_IMAGE_SPRITE_OPAQUE_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("e52a0fe3-e8b1-438b-99e7-1346c629abd2");
const CHARMI_VIEW_CLEAR_HANDLE: Handle<Shader> =
    weak_handle!("ceeae3dc-8d00-4062-bd35-911c86ebac77");

#[derive(Debug)]
pub struct CharmiRenderPlugin;

impl Plugin for CharmiRenderPlugin {
    fn build(&self, app: &mut App) {
        require_plugin(app, TimePlugin); // Used for shader resources
        require_plugin(app, FrameCountPlugin); // Used for shader resources
        require_plugin(app, TerminalCtrlCHandlerPlugin); // Handle CtrlC in terminal
        require_plugin(app, AssetPlugin::default());
        require_plugin(app, RenderPlugin::default());
        require_plugin(app, ImagePlugin::default());
        // TODO open issue with bevy so I don't need this plugin to use the render plugin
        require_plugin(
            app,
            WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                close_when_requested: false,
            },
        );
        app.add_plugins((
            self::view::ViewPlugin,
            self::image_sprite::ImageSpritePlugin,
            self::transform::CharmiTransformPlugin,
            self::pipeline::CharmiRenderPipelinePlugin,
        ));

        load_internal_asset!(
            app,
            CHARMI_SHADER_HANDLE,
            "shader_import/charmi.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CHARMI_BOX_SHADER_HANDLE,
            "shader_import/box.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CHARMI_IMAGE_SPRITE_OPAQUE_SHADER_HANDLE,
            "shader_import/image_sprite.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CHARMI_VIEW_CLEAR_HANDLE,
            "shader_import/view_clear.wgsl",
            Shader::from_wgsl
        )
    }
}

fn require_plugin<P: Plugin>(app: &mut App, plugin: P) {
    if !app.is_plugin_added::<P>() {
        app.add_plugins(plugin);
    }
}

// Should possibly rename to "SpriteBindGroup"
#[derive(Debug, Resource)]
pub struct CharmiGlobalsBindGroup(pub BindGroup);

#[derive(Debug, Resource)]
pub struct CharmiBindGroupLayouts {
    pub view_layout: BindGroupLayout,
    pub sprite_layout: BindGroupLayout,
    pub image_sprite_layout: BindGroupLayout,
}

impl FromWorld for CharmiBindGroupLayouts {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer::<CharmiImage>(false),
                    uniform_buffer::<TransformCh>(true),
                    uniform_buffer::<GlobalsUniform>(false),
                ),
            ),
        );
        let sprite_layout = render_device.create_bind_group_layout(
            None,
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TransformCh>(true),
                    uniform_buffer::<GlobalsUniform>(false),
                ),
            ),
        );
        let image_sprite_layout = CharmiImageSpriteBuffer::bind_group_layout(render_device);
        Self {
            view_layout,
            sprite_layout,
            image_sprite_layout,
        }
    }
}
