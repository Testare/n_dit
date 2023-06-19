use crossterm::event::{KeyCode, KeyModifiers};

use crate::term::prelude::*;

#[derive(Eq, Hash, PartialEq)]
struct KeyCombo(KeyCode, KeyModifiers);

#[derive(Eq, Hash, PartialEq)]
pub enum Submap {
    Node = 0,
}

#[derive(Clone, Debug, Hash)]
pub enum NamedInput {
    Direction(Compass),
    AltDirection(Compass),
    Ready,
    Activate,
    MenuFocusNext,
    MenuFocusPrev,
    Undo,
    Help,
    NextMsg,
}

#[derive(Component)]
pub struct KeyMap {
    submaps: HashMap<Submap, HashMap<KeyCombo, NamedInput>>,
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
                        KeyCombo(KeyCode::Char('?'), KeyModifiers::NONE),
                        NamedInput::Help,
                    ),
                    (
                        KeyCombo(KeyCode::Tab, KeyModifiers::NONE),
                        NamedInput::MenuFocusNext,
                    ),
                    (
                        KeyCombo(KeyCode::BackTab, KeyModifiers::NONE),
                        NamedInput::MenuFocusPrev,
                    ),
                    (
                        KeyCombo(KeyCode::Tab, KeyModifiers::SHIFT),
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
        self.submaps
            .get(&submap)?
            .get(&KeyCombo(code, modifiers))
            .cloned()
    }
}
