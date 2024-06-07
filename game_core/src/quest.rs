use serde::{Deserialize, Serialize};

use super::node::NodeId;
use crate::player::{Ncp, Player};
use crate::prelude::*;
use crate::saving::{LoadData, SaveData, SaveSchedule};

#[derive(Debug)]
pub struct QuestPlugin;

mod key {
    use typed_key::*;

    pub const SAVE_QUEST_STATUS: Key<super::QuestStatus> = typed_key!("quest_status");
}

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<QuestStatus>()
            .add_systems(SaveSchedule, sys_save_quest_status);
    }
}
/// Indicates status of nodes and quests
/// Indicates which nodes have been completed. Would love to fit this into a more comprehensive player
/// metadata/save data/progress flag/score system later.
#[derive(Component, Debug, Default, Deserialize, Eq, PartialEq, Reflect, Serialize)]
#[reflect(Component)]
#[serde(transparent)]
pub struct QuestStatus {
    flags: HashMap<String, u32>,
}

impl QuestStatus {
    pub fn record_node_done(&mut self, node_id: &NodeId) {
        *self.flags.entry(node_id.set().to_string()).or_default() |= node_id.num_flag();
    }

    pub fn is_node_done(&self, node_id: &NodeId) -> bool {
        self.flags
            .get(node_id.set())
            .map(|victory_flags_for_set| (victory_flags_for_set & node_id.num_flag()) != 0)
            .unwrap_or(false)
    }
}

pub fn sys_save_quest_status(
    mut res_save_data: ResMut<SaveData>,
    q_player: Query<&QuestStatus, (With<Player>, With<Ncp>)>,
) {
    for quest_status in q_player.iter() {
        res_save_data
            .put(key::SAVE_QUEST_STATUS, quest_status)
            .expect("TODO figure out how to handle save issues");
    }
}

pub fn sys_load_quest_statuss(
    res_save_data: Res<LoadData>,
    mut q_player: Query<&mut QuestStatus, (With<Player>, With<Ncp>)>,
) {
    for mut quest_status in q_player.iter_mut() {
        if let Ok(Some(load_qs)) = res_save_data.get_optional(key::SAVE_QUEST_STATUS) {
            quest_status.set_if_neq(load_qs);
        }
    }
}
