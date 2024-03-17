use game_core::player::Player;
use game_core::shop::InShop;

use crate::prelude::*;

#[derive(Debug)]
pub struct ShopUiPlugin;

impl Plugin for ShopUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sys_open_shop_ui);
    }
}

pub fn sys_open_shop_ui(players_in_shops: Query<(Entity, &InShop), (With<Player>, Added<InShop>)>) {
    for (id, &InShop(shop_id)) in players_in_shops.iter() {
        log::debug!("TODO (player [{id:?}] entered shop [{shop_id:?}])");
    }
}
