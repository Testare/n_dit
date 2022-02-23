use crate::{Direction, Node, NodeRestorePoint, Point, PointSet, UiAction, UserInput};
use getset::{CopyGetters, Setters};
use std::rc::Rc;

#[derive(Debug, CopyGetters, Setters)]
pub struct NodeUiState {
    focus: NodeFocus,
    phase: NodePhase,
    #[get_copy = "pub"]
    #[set = "pub(super)"]
    selected_square: Point,
}

impl NodeUiState {
    #[deprecated]
    pub fn set_selected_sprite_key_if_phase_is_right(&mut self, sprite_key: Option<usize>) {
        if let NodePhase::FreeSelect {
            selected_sprite_key,
            ..
        } = &mut self.phase
        {
            *selected_sprite_key = sprite_key;
        }
    }

    pub fn ui_action_for_input(&self, user_input: UserInput) -> Option<UiAction> {
        // TODO Undo
        return match self.focus {
            NodeFocus::ActionMenu => {
                match user_input {
                    UserInput::Activate => Some(UiAction::ConfirmSelection),
                    UserInput::Dir(dir) => {
                        if dir.matches(Direction::North | Direction::South) {
                            Some(UiAction::change_selected_menu_item(dir))
                        } else {
                            None
                        }
                    }
                    UserInput::AltDir(dir) => Some(UiAction::move_selected_square(dir, 1)),
                    UserInput::Select => Some(UiAction::ChangeSelection),
                    _ => None, // TODO Undo
                }
            }
            NodeFocus::Grid => {
                match user_input {
                    UserInput::Dir(dir) => match self.phase {
                        NodePhase::MoveSprite { .. } => Some(UiAction::move_active_sprite(dir)),
                        _ => Some(UiAction::move_selected_square(dir, 1)),
                    },
                    UserInput::AltDir(dir) => {
                        match self.phase {
                            // When moving, alt will move the selected square
                            NodePhase::MoveSprite { .. } => {
                                Some(UiAction::move_selected_square(dir, 1))
                            }
                            // Otherwise, just increase movement speed
                            _ => Some(UiAction::move_selected_square(dir, 2)),
                        }
                    }

                    UserInput::Activate => match self.phase {
                        NodePhase::SpriteAction { .. } => Some(UiAction::PerformSpriteAction),
                        NodePhase::FreeSelect {
                            selected_sprite_key: Some(sprite_key),
                            ..
                        }
                        | NodePhase::MoveSprite {
                            selected_sprite_key: sprite_key,
                            ..
                        } => Some(UiAction::ActivateSprite(sprite_key)),
                        _ => None,
                    },

                    UserInput::Select => Some(UiAction::ChangeSelection),
                    _ => None,
                }
            }
        };
    }

    pub fn apply_action(&mut self, node: &mut Node, ui_action: UiAction) -> Result<(), String> {
        match ui_action {
            UiAction::ConfirmSelection => {
                if self.focus == NodeFocus::ActionMenu {
                    match &mut self.phase {
                        NodePhase::FreeSelect {
                            selected_action_index: Some(selected_action_index),
                            selected_sprite_key: Some(selected_sprite_key),
                        } => {
                            // Need to active the sprite
                            self.phase = NodePhase::SpriteAction {
                                selected_action_index: *selected_action_index,
                                selected_sprite_key: *selected_sprite_key,
                                undo_state: Rc::new(node.create_restore_point()),
                            };
                        }
                        NodePhase::MoveSprite {
                            selected_action_index: Some(selected_action_index),
                            selected_sprite_key,
                            undo_state,
                        } => {
                            self.phase = NodePhase::SpriteAction {
                                selected_action_index: *selected_action_index,
                                selected_sprite_key: *selected_sprite_key,
                                undo_state: undo_state.clone(),
                            };
                        }
                        _ => {}
                    };
                    //*/
                    /*
                    match self.phase {
                        NodePhase::FreeSelect{..} | NodePhase::MoveSprite {..} => {
                            self.phase.transition_to_sprite_action(node)?;
                        }
                        _ => {}
                    }*/
                    self.focus = NodeFocus::Grid;
                    Ok(())
                } else {
                    Err("Confirm what selection?".to_string())
                }
            }
            UiAction::ChangeSelectedMenuItem(dir) => {
                if self.focus == NodeFocus::ActionMenu {
                    let selected_sprite_key = node
                        .active_sprite_key()
                        .or_else(|| {
                            node.with_sprite_at(self.selected_square(), |sprite| (sprite.key()))
                        })
                        .unwrap();
                    if let Some(action_index) = self.selected_action_index() {
                        let num_actions = node
                            .with_sprite(selected_sprite_key, |sprite| sprite.actions().len())
                            .unwrap();
                        self.set_selected_action_index(match dir {
                            Direction::North => (action_index + num_actions - 1) % num_actions,
                            Direction::South => (action_index + 1) % num_actions,
                            _ => action_index,
                        })
                    }
                }
                Ok(())
            }
            UiAction::ChangeSelection => {
                let selected_sprite_key = node.active_sprite_key().or_else(|| {
                    node.with_sprite_at(self.selected_square(), |sprite| (sprite.key()))
                });
                match self.focus {
                    NodeFocus::ActionMenu => {
                        if 0 != node
                            .with_sprite(selected_sprite_key.unwrap(), |sprite| sprite.moves())
                            .unwrap_or(0)
                        {
                            unsafe {
                                self.clear_selected_action_index();
                            }
                        }
                        self.focus = NodeFocus::Grid;
                    }
                    NodeFocus::Grid => {
                        if selected_sprite_key.is_some() {
                            if self.selected_action_index() == None {
                                self.set_default_selected_action();
                            }
                            self.focus = NodeFocus::ActionMenu;
                        }
                    }
                }
                Ok(())
            }
            UiAction::MoveSelectedSquare { direction, speed } => {
                let range_limit: Option<PointSet> =
                    self.selected_action_index().and_then(|action_index| {
                        node.with_active_sprite(|sprite| sprite.range_of_action(action_index))
                    });
                // FIXME IMMEDIATELY
                // self.move_selected_square(direction, speed, range_limit);
                Ok(())
            }
            UiAction::SetSelectedSquare(pt) => {
                self.set_selected_square(pt);
                Ok(())
            }
            UiAction::ActivateSprite(sprite_key) => {
                match &mut self.phase {
                    NodePhase::FreeSelect {
                        selected_sprite_key,
                        selected_action_index,
                    } => {
                        if node.activate_sprite(sprite_key) {
                            // Should make this a function
                            if let Some((moves, actions)) = node.with_active_sprite(|sprite| {
                                (sprite.moves(), sprite.actions().len())
                            }) {
                                if moves != 0 {
                                    self.phase.transition_to_move_sprite(node);
                                } else if actions != 0 {
                                    self.phase.transition_to_sprite_action(node);
                                } else {
                                    // TODO guard against this in game
                                    panic!("How do we have a sprite with no actions or moves?")
                                }
                            }
                            Ok(())
                        } else {
                            Ok(())
                        }
                    }
                    _ => {
                        node.deactivate_sprite();
                        self.phase
                            .transition_to_free_select(self.selected_square, node);
                        Ok(())
                    }
                }
            }
            UiAction::MoveActiveSprite(dir) => {
                let (remaining_moves, head, is_tapped) = node
                    .with_active_sprite_mut(|mut sprite| {
                        (
                            sprite.move_sprite(vec![dir]),
                            sprite.head(),
                            sprite.tapped(),
                        )
                    })
                    .ok_or("No active sprite".to_string())?;

                self.set_selected_square(head);

                if remaining_moves? == 0 && !is_tapped && self.selected_action_index().is_none() {
                    // Sprite is still active, must still have some moves
                    self.set_default_selected_action();
                    self.phase.transition_to_sprite_action(node)?;
                }
                Ok(())
            }
            UiAction::PerformSpriteAction => {
                if let Some(action_index) = self.selected_action_index() {
                    let result = node.perform_sprite_action(action_index, self.selected_square());
                    if result.is_some() {
                        self.phase
                            .transition_to_free_select(self.selected_square, node);
                        unsafe {
                            self.clear_selected_action_index();
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

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
            NodePhase::SpriteAction { .. } => {
                panic!("can't clear selected action index when in sprite action phase")
            }
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

#[derive(Debug, Eq, PartialEq)]
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
        undo_state: Rc<NodeRestorePoint>,
        selected_sprite_key: usize,
        selected_action_index: Option<usize>,
    },
    SpriteAction {
        undo_state: Rc<NodeRestorePoint>,
        selected_sprite_key: usize,
        selected_action_index: usize,
    },
}

impl NodePhase {
    fn undo_state(&self) -> Option<&NodeRestorePoint> {
        match self {
            NodePhase::MoveSprite { undo_state, .. }
            | NodePhase::SpriteAction { undo_state, .. } => Some(undo_state),
            _ => None,
        }
    }

    fn transition_to_free_select(&mut self, selected_square: Point, node: &Node) {
        let selected_sprite_key = node.with_sprite_at(selected_square, |sprite| sprite.key());
        *self = NodePhase::FreeSelect {
            selected_sprite_key,
            selected_action_index: None,
        };
    }

    fn transition_to_move_sprite(&mut self, node: &Node) -> Result<(), String> {
        *self = match self {
            NodePhase::FreeSelect {
                selected_action_index,
                selected_sprite_key: Some(selected_sprite_key),
            } => Ok::<_, String>(NodePhase::MoveSprite {
                selected_action_index: selected_action_index.clone(),
                selected_sprite_key: *selected_sprite_key,
                undo_state: Rc::new(node.create_restore_point()),
            }),
            NodePhase::SpriteAction {
                selected_sprite_key,
                undo_state,
                ..
            } => Ok::<_, String>(NodePhase::MoveSprite {
                selected_sprite_key: *selected_sprite_key,
                selected_action_index: None,
                undo_state: undo_state.clone(),
            }),
            _ => unimplemented!("Implement!"),
        }?;
        Ok(())
    }

    fn transition_to_sprite_action(&mut self, node: &Node) -> Result<(), String> {
        *self = match self {
            NodePhase::FreeSelect {
                selected_action_index: Some(selected_action_index),
                selected_sprite_key: Some(selected_sprite_key),
            } => Ok::<_, String>(NodePhase::SpriteAction {
                selected_action_index: *selected_action_index,
                selected_sprite_key: *selected_sprite_key,
                undo_state: Rc::new(node.create_restore_point()),
            }),
            NodePhase::MoveSprite {
                selected_sprite_key,
                selected_action_index,
                undo_state,
            } => {
                // TODO check if sprite has actions available, else go to
                Ok::<_, String>(NodePhase::SpriteAction {
                    selected_sprite_key: *selected_sprite_key,
                    selected_action_index: selected_action_index.unwrap_or(0),
                    undo_state: undo_state.clone(),
                })
            }
            _ => unimplemented!("Implement!"),
        }?;
        Ok(())
    }
}
