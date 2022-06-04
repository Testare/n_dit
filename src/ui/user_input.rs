use game_core::{Bounds, Direction, Point};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

#[derive(Clone, Copy, Debug)]
pub enum UserInput {
    Dir(Direction), // Gamepad joystick: Defaults to HJKL and arrow keys. Might need to configure for WASD as well?
    AltDir(Direction), // Gamepad D-pad: Moving while special key is held down
    Activate,       // Gamepad "A": Default binds are "A" or Space
    Back,           // Gamepad "B": Default binds are "U" or Backspace
    Select,         // Gamepad "X" or "Select": Default binds are "S" or Tab
    Menu,           // Gamepad "Y" or "Start": Default binds are "M" or Escape
    Next,           // Gamepad "R": Default binds are "N"
    Previous,       // Gamepad "L": Default binds are "P"
    Click(Point),
    Drag(Point),
    AltClick(Point),
    Resize(Bounds),
    Quit,  // Power button?: Defaults to Q
    Debug, // Only used for debugging, should not be used when not in development mode
}

#[inline]
fn direction_input(is_alt: bool, dir: Direction) -> Option<UserInput> {
    if is_alt {
        Some(UserInput::AltDir(dir))
    } else {
        Some(UserInput::Dir(dir))
    }
}

impl UserInput {
    pub fn from_event(event: Event) -> Option<UserInput> {
        match event {
            Event::Key(KeyEvent { code, modifiers }) => {
                let ctrl = modifiers.contains(KeyModifiers::CONTROL);
                match code {
                    KeyCode::Char('h') | KeyCode::Left => direction_input(ctrl, Direction::West),
                    KeyCode::Char('H') => direction_input(true, Direction::West),
                    KeyCode::Char('k') | KeyCode::Up => direction_input(ctrl, Direction::North),
                    KeyCode::Char('K') => direction_input(true, Direction::North),
                    KeyCode::Char('j') | KeyCode::Down => direction_input(ctrl, Direction::South),
                    KeyCode::Char('J') => direction_input(true, Direction::South),
                    KeyCode::Char('l') | KeyCode::Right => direction_input(ctrl, Direction::East),
                    KeyCode::Char('L') => direction_input(true, Direction::East),
                    KeyCode::Char('u') | KeyCode::Backspace => Some(UserInput::Back),
                    KeyCode::Char('n') => Some(UserInput::Next),
                    KeyCode::Char('p') => Some(UserInput::Previous),
                    KeyCode::Char('q') => Some(UserInput::Quit),
                    KeyCode::Char('-') => Some(UserInput::Debug),
                    KeyCode::Esc => Some(UserInput::Menu),
                    KeyCode::Tab => Some(UserInput::Select),
                    KeyCode::Char(' ') => Some(UserInput::Activate),
                    _ => None,
                }
            }
            Event::Mouse(MouseEvent {
                kind,
                column,
                row,
                modifiers,
            }) => {
                let ctrl = modifiers.contains(KeyModifiers::CONTROL);
                let pt: Point = (column.into(), row.into());
                if let MouseEventKind::Down(button) = kind {
                    if ctrl || button == MouseButton::Right {
                        Some(UserInput::AltClick(pt))
                    } else {
                        Some(UserInput::Click(pt))
                    }
                } else if let MouseEventKind::Drag(button) = kind {
                    if ctrl || button == MouseButton::Right {
                        Some(UserInput::Drag(pt))
                    } else {
                        Some(UserInput::Click(pt))
                    }
                } else {
                    None
                }
            }
            Event::Resize(w, h) => Some(UserInput::Resize(Bounds::from((w, h)))),
        }
    }
}
