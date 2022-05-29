use super::NodeCt;
use crate::{Direction, GameCommand, Node, Point, PointSet, UiAction, UserInput};
use getset::{CopyGetters, Setters};
use log::debug;

#[derive(Clone, Debug, CopyGetters, Setters)]
pub struct NodeUiState {
    focus: NodeFocus,
    phase: NodePhase,
    #[get_copy = "pub"]
    selected_square: Point,
}

// TODO Visual differences between NodeFocus
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NodeFocus {
    Grid,
    ActionMenu,
}

#[derive(Clone, Debug)]
enum NodePhase {
    /* TODO SetUp
    SetUp {
        selected_sprite_index: usize,
        selected_action_index: Option<usize>,
    }, */
    FreeSelect {
        selected_sprite_key: Option<usize>,
        selected_action_index: Option<usize>,
    },
    MoveSprite {
        selected_sprite_key: usize,
        selected_action_index: Option<usize>,
    },
    SpriteAction {
        selected_sprite_key: usize,
        selected_action_index: usize,
    },
}

impl NodeUiState {
    fn set_selected_square(&mut self, pt: Point, node: &Node) {
        self.selected_square = pt;

        if let NodePhase::FreeSelect {
            selected_sprite_key,
            ..
        } = &mut self.phase
        {
            *selected_sprite_key = node.with_sprite_at(pt, |sprite| sprite.key());
        }
    }
    fn move_selected_square(
        &mut self,
        node: &Node,
        direction: Direction,
        speed: usize,
        range_limit: Option<PointSet>,
    ) {
        let new_pt = direction.add_to_point(self.selected_square(), speed, node.bounds());
        if let Some(point_set) = range_limit {
            if !point_set.contains(new_pt) {
                return;
            }
        }
        self.set_selected_square(new_pt, node);
    }

    fn change_focus(&mut self, node: &Node) -> Result<(), String> {
        match self.focus {
            NodeFocus::ActionMenu => {
                match self.phase {
                    NodePhase::FreeSelect { .. } => self.clear_selected_action_index(),
                    NodePhase::MoveSprite {
                        selected_sprite_key,
                        ..
                    } => {
                        // TODO To consider: What if the selected square is over another sprite and not the active sprite?
                        if node
                            .with_sprite(selected_sprite_key, |sprite| sprite.moves() != 0)
                            .ok_or_else(||"NodePhase is not FreeSelect while the selected_sprite_key is invalid".to_string())?
                        {
                            self.clear_selected_action_index();
                        }
                    }
                    NodePhase::SpriteAction {
                        selected_sprite_key,
                        ..
                    } => {
                        let (moves, head_pt) = node
                            .with_sprite(selected_sprite_key, |sprite| (sprite.moves(), sprite.head()))
                            .ok_or_else(||"NodePhase is not FreeSelect while the selected_sprite_key is invalid".to_string())?;
                        if moves != 0 {
                            self.phase.transition_to_move_sprite(node)?;
                            self.set_selected_square(head_pt, node);
                            self.clear_selected_action_index();
                        }
                    }
                }
                self.focus = NodeFocus::Grid;
            }
            NodeFocus::Grid => {
                let selected_sprite_key = node.active_sprite_key().or_else(|| {
                    node.with_sprite_at(self.selected_square(), |sprite| (sprite.key()))
                });
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

    pub fn ui_actions_for_click_target(
        &self,
        node: &Node,
        click_target: NodeCt,
        alt: bool,
    ) -> Vec<UiAction> {
        match click_target {
            NodeCt::Grid(pt) => {
                // TODO If NodeFocus is not on the grid, change it to the grid
                // TODO If there is a selected action index, clicking on a square should use that action in NodeMove phase
                match self.phase {
                    NodePhase::FreeSelect { .. } => {
                        if alt {
                            let sprite_key_opt: Option<usize> = node.with_sprite_at(pt, |sprite| {
                                if sprite.team() == node.active_team() {
                                    Some(sprite.key())
                                } else {
                                    None
                                }
                            });

                            if let Some(sprite_key) = sprite_key_opt {
                                return vec![UiAction::activate_sprite(sprite_key)];
                            }
                        }
                        vec![UiAction::set_selected_square(pt)]
                    }
                    NodePhase::MoveSprite { .. } => {
                        // Calculate the directions necessary
                        // TODO Rethink logic for if the square clicked on is too far away.
                        // Options:
                        // * Pathfinding to square?
                        // * Move one square, but do conditional checked for blocked paths (I.E. If you click NW, and North is blocked, go West)

                        let sprite_head = node
                            .with_active_sprite(|sprite| sprite.head())
                            .expect("Move sprite state without active sprite");
                        let mut dirs = Vec::new();
                        if sprite_head.1 > pt.1 {
                            dirs.push(Direction::North);
                        }
                        if sprite_head.0 < pt.0 {
                            dirs.push(Direction::East);
                        }
                        if sprite_head.1 < pt.1 {
                            dirs.push(Direction::South);
                        }
                        if sprite_head.0 > pt.0 {
                            dirs.push(Direction::West);
                        }
                        dirs.into_iter()
                            .find_map(|dir| {
                                if node
                                    .with_active_sprite(|sprite| sprite.can_move(dir))
                                    .unwrap()
                                {
                                    // TODO this is probably not the right pattern
                                    Some(UiAction::GameCommand(GameCommand::NodeMoveActiveSprite(
                                        dir,
                                    )))
                                } else {
                                    None
                                }
                            })
                            .into_iter()
                            .collect()
                    }
                    NodePhase::SpriteAction {
                        selected_action_index,
                        ..
                    } => vec![UiAction::perform_sprite_action(selected_action_index, pt)],
                }
            }
            NodeCt::SpriteActionMenu(sprite_action_index) => {
                // TODO alt click -> Potentially activate sprite
                vec![UiAction::set_selected_menu_item(sprite_action_index)]
            }
            _ => {
                unimplemented!("Node click target not implemented yet")
            }
        }
    }

    pub fn ui_action_for_input(&self, node: &Node, user_input: UserInput) -> Vec<UiAction> {
        // TODO Undo
        // TODO Not sure why we don't have node state passed in here
        match self.focus {
            NodeFocus::ActionMenu => {
                match user_input {
                    UserInput::Activate => match self.phase {
                        NodePhase::FreeSelect {
                            selected_sprite_key,
                            ..
                        } => {
                            vec!(
                                    UiAction::activate_sprite(selected_sprite_key.expect("How do we confirm selection when there is no selected sprite key?")),
                                    UiAction::confirm_selection(None),
                                )
                        }
                        _ => vec![UiAction::confirm_selection(None)],
                    },
                    UserInput::Dir(dir) => {
                        if dir.matches(Direction::North | Direction::South) {
                            vec![UiAction::change_selected_menu_item(dir)]
                        } else {
                            UiAction::none()
                        }
                    }
                    UserInput::AltDir(dir) => vec![UiAction::move_selected_square(dir, 1)],
                    UserInput::Select | UserInput::Back => vec![UiAction::change_selection()],
                    _ => UiAction::none(), // TODO Undo
                }
            }
            NodeFocus::Grid => {
                match user_input {
                    UserInput::Dir(dir) => match self.phase {
                        NodePhase::MoveSprite { .. } => vec![UiAction::move_active_sprite(dir)],
                        _ => vec![UiAction::move_selected_square(dir, 1)],
                    },
                    UserInput::AltDir(dir) => {
                        match self.phase {
                            // When moving, alt will move the selected square
                            NodePhase::MoveSprite { .. } => {
                                vec![UiAction::move_selected_square(dir, 1)]
                            }
                            // Otherwise, just increase movement speed
                            _ => vec![UiAction::move_selected_square(dir, 2)],
                        }
                    }
                    UserInput::Activate => match self.phase {
                        NodePhase::SpriteAction {
                            selected_action_index,
                            ..
                        } => vec![UiAction::perform_sprite_action(
                            selected_action_index,
                            self.selected_square(),
                        )],
                        NodePhase::FreeSelect {
                            selected_sprite_key: Some(sprite_key),
                            ..
                        } => {
                            if node
                                .with_sprite(sprite_key, |sprite| sprite.untapped())
                                .unwrap_or(false)
                            {
                                vec![UiAction::activate_sprite(sprite_key)]
                            } else {
                                UiAction::none()
                            }
                        }
                        NodePhase::MoveSprite { .. } => vec![UiAction::deactivate_sprite()], // TODO if node's sprite key at selected square is not selected_sprite_key, activate the new sprite key instead
                        _ => UiAction::none(),
                    },
                    UserInput::Select => vec![UiAction::change_selection()],
                    UserInput::Back => vec![UiAction::undo()],
                    _ => UiAction::none(),
                }
            }
        }
    }

    pub fn apply_action(&mut self, node: &Node, ui_action: &UiAction) -> Result<(), String> {
        match ui_action {
            UiAction::ConfirmSelection(_index_opt) => {
                if self.focus == NodeFocus::ActionMenu {
                    match &mut self.phase {
                        NodePhase::FreeSelect {
                            selected_action_index: Some(selected_action_index),
                            selected_sprite_key: Some(selected_sprite_key),
                        } => {
                            // FIXME Need to active the sprite
                            self.phase = NodePhase::SpriteAction {
                                selected_action_index: *selected_action_index,
                                selected_sprite_key: *selected_sprite_key,
                            };
                        }
                        NodePhase::MoveSprite {
                            selected_action_index: Some(selected_action_index),
                            selected_sprite_key,
                        } => {
                            self.phase = NodePhase::SpriteAction {
                                selected_action_index: *selected_action_index,
                                selected_sprite_key: *selected_sprite_key,
                            };
                        }
                        _ => {}
                    };
                    self.focus = NodeFocus::Grid;
                    Ok(())
                } else {
                    Err("Confirm what selection?".to_string())
                }
            }
            UiAction::SetSelectedMenuItem(idx) => {
                if self.focus != NodeFocus::ActionMenu
                    && matches!(self.phase, NodePhase::SpriteAction { .. })
                {
                    self.change_focus(node)?
                }
                // Might need to think more on this behavior if we're in move/action phase and we're highlighting another piece
                let selected_sprite_key = node
                    .active_sprite_key()
                    .or_else(|| node.with_sprite_at(self.selected_square(), |sprite| sprite.key()))
                    .unwrap();
                let num_actions = node
                    .with_sprite(selected_sprite_key, |sprite| sprite.actions().len()) // TODO method to count actions
                    .unwrap();
                if *idx < num_actions {
                    self.set_selected_action_index(*idx);
                }
                Ok(())
            }
            UiAction::ChangeSelectedMenuItem(dir) => {
                if self.focus == NodeFocus::ActionMenu {
                    let selected_sprite_key = node
                        .active_sprite_key()
                        .or_else(|| {
                            node.with_sprite_at(self.selected_square(), |sprite| sprite.key())
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
            UiAction::ChangeSelection => self.change_focus(node),
            UiAction::MoveSelectedSquare { direction, speed } => {
                let range_limit: Option<PointSet> =
                    self.selected_action_index().and_then(|action_index| {
                        node.with_active_sprite(|sprite| sprite.range_of_action(action_index))
                    });
                debug!("Moving selected square {:?} by {}", direction, speed);
                self.move_selected_square(node, *direction, *speed, range_limit);
                Ok(())
            }
            UiAction::SetSelectedSquare(pt) => {
                self.set_selected_square(*pt, node);
                Ok(())
            }
            UiAction::GameCommand(GameCommand::NodeActivateSprite { .. }) => {
                // TODO We don't know if this action was successful?
                // This means if we try to activate unsuccessfully, selected square will go to
                // active sprite
                // ...But is this a bug or a feature?
                if let Some((moves, actions, head)) = node.with_active_sprite(|sprite| {
                    (sprite.moves(), sprite.actions().len(), sprite.head())
                }) {
                    self.set_selected_square(head, node);
                    if moves != 0 {
                        self.phase.transition_to_move_sprite(node)?;
                    } else if actions != 0 {
                        self.phase.transition_to_sprite_action()?;
                    } else {
                        // TODO guard against this in game, perhaps never untap these sprites
                        panic!("How do we have a sprite with no actions or moves?")
                    }
                }
                Ok(())
            }
            UiAction::GameCommand(GameCommand::NodeDeactivateSprite) => {
                self.phase
                    .transition_to_free_select(self.selected_square, node);
                Ok(())
            }
            UiAction::GameCommand(GameCommand::NodeMoveActiveSprite(_direction)) => {
                if let Some((remaining_moves, head, is_tapped)) = node
                    .with_active_sprite(|sprite| (sprite.moves(), sprite.head(), sprite.tapped()))
                {
                    self.set_selected_square(head, node);
                    if remaining_moves == 0 && !is_tapped && self.selected_action_index().is_none()
                    {
                        // Sprite is still active, must still have some moves
                        self.set_default_selected_action();
                        self.phase.transition_to_sprite_action()?;
                    }
                } else {
                    // TODO fix this bug hat applies to sprites without actions
                    // self.set_selected_square(self.selected_square() + directions);
                    self.phase
                        .transition_to_free_select(self.selected_square, node);
                }

                Ok(())
            }
            UiAction::GameCommand(GameCommand::NodeTakeAction { .. }) => {
                if node.active_sprite_key().is_none() {
                    self.phase
                        .transition_to_free_select(self.selected_square, node);
                    self.clear_selected_action_index();
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

    pub fn clear_selected_action_index(&mut self) {
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
                log::warn!("clear_selected_action_index() called while we were NodePhase::SpriteAction, which is a noop")
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

impl NodePhase {
    fn transition_to_free_select(&mut self, selected_square: Point, node: &Node) {
        let selected_sprite_key = node.with_sprite_at(selected_square, |sprite| sprite.key());
        *self = NodePhase::FreeSelect {
            selected_sprite_key,
            selected_action_index: None,
        };
    }

    fn transition_to_move_sprite(&mut self, node: &Node) -> Result<(), String> {
        if matches!(self, NodePhase::MoveSprite { .. }) {
            Ok(())
        } else {
            *self = match self {
                NodePhase::FreeSelect {
                    selected_action_index,
                    ..
                } => Ok::<_, String>(NodePhase::MoveSprite {
                    selected_action_index: *selected_action_index,
                    selected_sprite_key: node.active_sprite_key().unwrap(),
                }),
                NodePhase::SpriteAction {
                    selected_sprite_key,
                    ..
                } => Ok::<_, String>(NodePhase::MoveSprite {
                    selected_sprite_key: *selected_sprite_key,
                    selected_action_index: None,
                }),
                _ => panic!(
                    "Unreachable arm case hit when transitioning to MoveSprite phase in NodeUi"
                ),
            }?;
            Ok(())
        }
    }

    fn transition_to_sprite_action(&mut self) -> Result<(), String> {
        *self = match self {
            NodePhase::FreeSelect {
                selected_action_index: Some(selected_action_index),
                selected_sprite_key: Some(selected_sprite_key),
            } => Ok::<_, String>(NodePhase::SpriteAction {
                selected_action_index: *selected_action_index,
                selected_sprite_key: *selected_sprite_key,
            }),
            NodePhase::MoveSprite {
                selected_sprite_key,
                selected_action_index,
            } => {
                // TODO check if sprite has actions available, else go to
                Ok::<_, String>(NodePhase::SpriteAction {
                    selected_sprite_key: *selected_sprite_key,
                    selected_action_index: selected_action_index.unwrap_or(0),
                })
            }
            _ => unimplemented!("Implement!"),
        }?;
        Ok(())
    }
}
