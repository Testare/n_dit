use crate::{Node, Point};
use getset::{CopyGetters, Setters};

#[derive(Debug, CopyGetters, Setters)]
pub struct NodeUiState {
    focus: NodeFocus,
    phase: NodePhase,
    #[get_copy = "pub"]
    #[set = "pub(super)"]
    selected_square: Point,
}

impl NodeUiState {
    pub fn selected_action_index(&self) -> Option<usize> {
        match self.phase {
            NodePhase::FreeSelect {
                selected_action_index,
                ..
            } => selected_action_index,

            NodePhase::MoveSprite {
                selected_action_index,
                ..
            } => selected_action_index,

            NodePhase::SpriteAction {
                selected_action_index,
                ..
            } => Some(selected_action_index),
        }
    }

    pub fn set_default_selected_action(&mut self) {
        // TODO check sprite metadata for last selected action?
        self.set_selected_action_index(0);
    }

    // # Safety
    // Do not call when in sprite action phase. Hope in future to remove this function
    #[deprecated]
    pub unsafe fn clear_selected_action_index(&mut self) {
        match &mut self.phase {
            NodePhase::FreeSelect {
                selected_action_index,
                ..
            } => *selected_action_index = None,
            NodePhase::MoveSprite {
                selected_action_index,
                ..
            } => *selected_action_index = None,

            NodePhase::SpriteAction {
                selected_action_index,
                ..
            } => panic!("can't clear selected action index when in sprite action phase"),
        }
    }

    pub fn set_selected_action_index(&mut self, idx: usize) {
        match &mut self.phase {
            NodePhase::FreeSelect {
                selected_action_index,
                ..
            } => *selected_action_index = Some(idx),
            NodePhase::MoveSprite {
                selected_action_index,
                ..
            } => *selected_action_index = Some(idx),

            NodePhase::SpriteAction {
                selected_action_index,
                ..
            } => *selected_action_index = idx,
        }
    }
}

impl From<&Node> for NodeUiState {
    fn from(node: &Node) -> Self {
        NodeUiState {
            focus: NodeFocus::Grid,
            phase: NodePhase::FreeSelect {
                selected_sprite_key: node.with_sprite_at((0, 0), |sprite| sprite.key()),
                selected_action_index: None,
            },
            selected_square: (0, 0),
        }
    }
}

#[derive(Debug)]
enum NodeFocus {
    Grid,
    ActionMenu,
    // SpriteMenu
}

#[derive(Debug)]
enum NodePhase {
    /* TODO SetUp
    SetUp {
        selected_sprite_index: usize,
        selected_action_index: Option<usize>,
    }, */
    /* TODO Enemy Turn
    EnemyTurn,*/
    FreeSelect {
        selected_sprite_key: Option<usize>,
        selected_action_index: Option<usize>,
    },
    MoveSprite {
        undo_state: Node,
        selected_sprite_key: usize,
        selected_action_index: Option<usize>,
    },
    SpriteAction {
        undo_state: Node,
        selected_sprite: usize,
        selected_action_index: usize,
    },
}
