use bevy::asset::Handle;
use bevy::ecs::system::Resource;
use charmi::CharmieActor;
use game_core::prelude::*;

pub const FX_ACTOR: &str = "cq_term/fx.charmia";
pub const PICKUP_SOUND: &str = "tmp/audio/mixkit-coins-sound-2003.wav";
pub const CARD_SOUND: &str = "tmp/audio/mixkit-poker-card-flick-2002.wav";

#[derive(Resource, Default)]
pub struct Fx {
    pub charmia: Handle<CharmieActor>,
    pub pickup_sound: Handle<AudioSource>,
    pub card_sound: Handle<AudioSource>,
}

pub fn sys_init_fx(mut res_fx: ResMut<Fx>, asset_server: Res<AssetServer>) {
    res_fx.charmia = asset_server.load(FX_ACTOR);
    res_fx.pickup_sound = asset_server.load(PICKUP_SOUND);
    res_fx.card_sound = asset_server.load(CARD_SOUND);
}
