use game_core::op::{Op, OpExecutorResource};
use game_core::player::ForPlayer;

use crate::base_ui::context_menu::ContextAction;

pub fn context_action_from_op<R: OpExecutorResource, O: Op + Clone>(
    name: &str,
    op: O,
) -> ContextAction {
    ContextAction::new(name.to_string(), move |id, world| {
        let for_player = world.get::<ForPlayer>(id).copied();
        if let Some(ForPlayer(player_id)) = for_player {
            world
                .get_resource_mut::<R>()
                .expect("should have initialized resource")
                .request(player_id, op.clone());
        }
    })
}
