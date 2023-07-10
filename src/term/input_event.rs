pub use crossterm::event::{KeyCode, KeyEvent as Key, KeyModifiers, MouseButton, MouseEventKind};
use game_core::prelude::*;

#[derive(Clone, Copy, Deref, DerefMut, Event)]
pub struct CrosstermEvent(pub crossterm::event::Event);

#[derive(Clone, Copy, Deref, DerefMut, Event)]
pub struct MouseEvent(pub crossterm::event::MouseEvent);

#[derive(Clone, Copy, Event, PartialEq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    // pub kind: KeyEventKind,
    // pub state: KeyEventState,
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(
        crossterm::event::KeyEvent {
            code, modifiers, ..
        }: crossterm::event::KeyEvent,
    ) -> Self {
        Self {
            code,
            modifiers,
            // kind,
            // state,
        }
    }
}
