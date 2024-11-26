use crate::input_event::{KeyCode, KeyModifiers};
use crate::prelude::*;

#[derive(Debug, Eq, Hash, PartialEq)]
struct KeyCombo(KeyCode, KeyModifiers);

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Submap {
    Node = 0,
}

#[derive(Clone, Debug, Hash, PartialEq)]
pub enum NamedInput {
    Direction(Compass),
    AltDirection(Compass),
    Ready,
    Activate,
    AltActivate,
    MenuFocusNext,
    MenuFocusPrev,
    Undo,
    Help,
    NextMsg,
}

#[derive(Component, Debug)]
pub struct KeyMap {
    submaps: HashMap<Submap, HashMap<KeyCombo, NamedInput>>,
    active_submaps: HashSet<Submap>,
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap {
            submaps: [(
                Submap::Node,
                [
                    (
                        KeyCombo(KeyCode::Char('w'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::North),
                    ),
                    (
                        KeyCombo(KeyCode::Char('k'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::North),
                    ),
                    (
                        KeyCombo(KeyCode::Char('a'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::West),
                    ),
                    (
                        KeyCombo(KeyCode::Char('h'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::West),
                    ),
                    (
                        KeyCombo(KeyCode::Char('s'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::South),
                    ),
                    (
                        KeyCombo(KeyCode::Char('j'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::South),
                    ),
                    (
                        KeyCombo(KeyCode::Char('d'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::East),
                    ),
                    (
                        KeyCombo(KeyCode::Char('l'), KeyModifiers::NONE),
                        NamedInput::Direction(Compass::East),
                    ),
                    (
                        KeyCombo(KeyCode::Char('w'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::North),
                    ),
                    (
                        KeyCombo(KeyCode::Char('k'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::North),
                    ),
                    (
                        KeyCombo(KeyCode::Char('a'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::West),
                    ),
                    (
                        KeyCombo(KeyCode::Char('h'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::West),
                    ),
                    (
                        KeyCombo(KeyCode::Char('s'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::South),
                    ),
                    (
                        KeyCombo(KeyCode::Char('j'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::South),
                    ),
                    (
                        KeyCombo(KeyCode::Char('d'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::East),
                    ),
                    (
                        KeyCombo(KeyCode::Char('l'), KeyModifiers::SHIFT),
                        NamedInput::AltDirection(Compass::East),
                    ),
                    (
                        KeyCombo(KeyCode::Char('-'), KeyModifiers::NONE),
                        NamedInput::Ready,
                    ),
                    (
                        KeyCombo(KeyCode::Char(' '), KeyModifiers::NONE),
                        NamedInput::Activate,
                    ),
                    (
                        KeyCombo(KeyCode::Char(' '), KeyModifiers::CONTROL),
                        NamedInput::AltActivate,
                    ),
                    (
                        KeyCombo(KeyCode::Char('?'), KeyModifiers::NONE),
                        NamedInput::Help,
                    ),
                    (
                        KeyCombo(KeyCode::Tab, KeyModifiers::NONE),
                        NamedInput::MenuFocusNext,
                    ),
                    (
                        KeyCombo(KeyCode::BackTab, KeyModifiers::SHIFT),
                        NamedInput::MenuFocusPrev,
                    ),
                    (
                        KeyCombo(KeyCode::Backspace, KeyModifiers::NONE),
                        NamedInput::Undo,
                    ),
                    (
                        KeyCombo(KeyCode::Enter, KeyModifiers::NONE),
                        NamedInput::NextMsg,
                    ),
                ]
                .into_iter()
                .collect(),
            )]
            .into_iter()
            .collect(),
            active_submaps: HashSet::new(),
        }
    }
}

impl KeyMap {
    pub fn named_input_for_key(
        &self,
        submap: Submap,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Option<NamedInput> {
        if self.active_submaps.contains(&submap) {
            self.submaps
                .get(&submap)?
                .get(&KeyCombo(code, modifiers))
                .cloned()
        } else {
            None
        }
    }

    pub fn is_submap_active(&self, submap: Submap) -> bool {
        self.active_submaps.contains(&submap)
    }

    pub fn activate_submap(&mut self, submap: Submap) {
        self.active_submaps.insert(submap);
    }

    pub fn deactivate_submap(&mut self, submap: Submap) {
        self.active_submaps.remove(&submap);
    }

    pub fn toggle_submap(&mut self, submap: Submap) {
        if !self.active_submaps.remove(&submap) {
            self.active_submaps.insert(submap);
        }
    }
}
