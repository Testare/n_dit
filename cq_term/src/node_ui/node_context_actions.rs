use std::ops::Deref;

use game_core::node::NodeOp;
use game_core::op::CoreOps;
use game_core::player::ForPlayer;
use getset::CopyGetters;

use super::node_ui_op::UiOps;
use super::{NodeUiOp, SelectedEntity};
use crate::base_ui::context_menu::ContextAction;
use crate::linkage::base_ui_game_core;
use crate::prelude::*;

#[derive(Resource, CopyGetters)]
pub struct NodeContextActions {
    #[get_copy = "pub"]
    unload_selected_access_point: Entity,
    #[get_copy = "pub"]
    clear_selected_action: Entity,
}

impl FromWorld for NodeContextActions {
    fn from_world(world: &mut World) -> Self {
        let unload_selected_access_point = world
            .spawn((
                Name::new("Unload access point CA"),
                ContextAction::new("Unload access point", |id, world| {
                    world.get(id).copied().and_then(|ForPlayer(player)| {
                        let access_point_id = (*world.get::<SelectedEntity>(player)?.deref())?;
                        let node_op = NodeOp::UnloadAccessPoint { access_point_id };
                        world.resource_mut::<CoreOps>().request(player, node_op);
                        Some(())
                    });
                }),
            ))
            .id();
        let clear_selected_action = world
            .spawn((
                Name::new("Clear selected action CA"),
                base_ui_game_core::context_action_from_op::<UiOps, _>(
                    "Clear action selection",
                    NodeUiOp::SetSelectedAction(None),
                ),
            ))
            .id();

        Self {
            unload_selected_access_point,
            clear_selected_action,
        }
    }
}
