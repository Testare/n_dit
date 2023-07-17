use bevy::asset::Handle;
use bevy::ecs::system::Resource;
use game_core::prelude::*;

use crate::charmie::{CharmieActor, CharmieAnimation};

pub const FX_ACTOR: &'static str = "cq_term/fx.charmia";

#[derive(Resource, Deref, Default)]
pub struct Fx(pub Handle<CharmieActor>);

pub fn sys_init_fx(mut res_fx: ResMut<Fx>, asset_server: Res<AssetServer>) {
    res_fx.0 = asset_server.load(FX_ACTOR);
}
