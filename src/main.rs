use core::time::Duration;
use crossterm::{
    self,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    execute,
};
use n_dit::{
    game::{Node, Piece, Sprite},
    grid_map::GridMap,
    ui::{SuperState, UiAction},
    Direction,
};
use std::io::stdout;

fn main() -> crossterm::Result<()> {
    let mut node = Node::from(GridMap::from(vec![
        vec![
            false, false, false, false, false, true, false, false, false, false, false,
        ],
        vec![
            false, false, false, false, true, true, true, false, false, false, false,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            false, false, true, true, true, true, true, true, true, false, false,
        ],
        vec![
            false, true, true, true, true, true, true, true, true, true, false,
        ],
        vec![
            true, true, true, true, true, false, true, true, true, true, true,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            false, false, false, false, false, true, false, false, false, false, false,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            true, true, true, true, false, false, false, true, true, true, true,
        ],
        vec![
            true, true, true, true, true, false, true, true, true, true, true,
        ],
        vec![
            false, true, true, true, true, true, true, true, true, true, false,
        ],
        vec![
            false, false, true, true, true, true, true, true, true, false, false,
        ],
        vec![
            false, false, false, true, true, true, true, true, false, false, false,
        ],
        vec![
            false, false, false, false, true, true, true, false, false, false, false,
        ],
        vec![
            false, false, false, false, false, true, false, false, false, false, false,
        ],
    ]));
    let guy_key = node.add_sprite((1, 6), Sprite::new("あ"));
    node.move_sprite((2, 6), guy_key.unwrap());
    node.move_sprite((3, 6), guy_key.unwrap());
    node.move_sprite((3, 7), guy_key.unwrap());

    let guy_key = node.add_sprite((4, 6), Sprite::new("死"));
    node.move_sprite((5, 6), guy_key.unwrap());
    node.move_sprite((5, 7), guy_key.unwrap());

    let guy_key = node.add_sprite((3, 3), Sprite::new("8]"));
    node.move_sprite((3, 4), guy_key.unwrap());

    node.add_sprite((14, 6), Sprite::new("<>"));
    node.add_piece((6, 1), Piece::Mon(500));
    node.add_piece((6, 2), Piece::AccessPoint);
    let state = SuperState::from(Some(node));
    execute!(
        stdout(),
        crossterm::cursor::Hide,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::terminal::SetTitle("n_dit"),
        crossterm::event::EnableMouseCapture
    )?;
    crossterm::terminal::enable_raw_mode()?;
    game_loop(state)?;
    crossterm::terminal::disable_raw_mode()?;
    execute!(
        stdout(),
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}

fn game_loop(mut state: SuperState) -> crossterm::Result<()> {
    let mut keep_going = true;
    state.render()?;
    while keep_going {
        if let Some(action) = get_next_action(&state)? {
            if action.is_quit() {
                keep_going = false;
            } else {
                state.apply_action(action).unwrap();
                //layout.render(&state)?;
                state.render()?;
            }
        }
    }
    Ok(())
}

const TIMEOUT: Duration = Duration::from_millis(500);

// TODO Could be implemented as an Iterator<Item=Result<UiAction>>, where no-ops are ignored
// instead of returning None
fn get_next_action(state: &SuperState) -> crossterm::Result<Option<UiAction>> {
    let event = if state.game_state().waiting_on_player_input() {
        // TODO better handling for when keys are held down/ clearing input queue
        crossterm::event::read()?
    } else {
        if crossterm::event::poll(TIMEOUT)? {
            crossterm::event::read()?
        } else {
            return Ok(Some(UiAction::next()));
        }
    };
    let action = match event {
        Event::Key(KeyEvent { code, modifiers }) => {
            let speed = if modifiers.contains(KeyModifiers::CONTROL) {
                2
            } else {
                1
            };
            match code {
                KeyCode::Char('h') => Some(UiAction::move_selected_square(Direction::West, speed)),
                KeyCode::Char('k') => Some(UiAction::move_selected_square(Direction::North, speed)),
                KeyCode::Char('j') => Some(UiAction::move_selected_square(Direction::South, speed)),
                KeyCode::Char('l') => Some(UiAction::move_selected_square(Direction::East, speed)),
                KeyCode::Char('q') => Some(UiAction::quit()),
                KeyCode::Char('-') => panic!("State report: TODO"),
                _ => None,
            }
        }
        Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: _,
        }) => {
            if let MouseEventKind::Down(_) = kind {
                state.action_for_char_pt((column.into(), row.into()))
            } else {
                None
            }
        }
        Event::Resize(w, h) => Some(UiAction::set_terminal_size(w, h)),
    };
    Ok(action)
}
