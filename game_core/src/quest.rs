use super::node::NodeId;
use crate::prelude::*;

#[derive(Debug)]
pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<QuestStatus>();
    }
}
/// Indicates status of nodes and quests
/// Indicates which nodes have been completed.
/// Would love to fit this into a more comprehensive player
/// metadata/save data/progress flag/score system later.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
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
