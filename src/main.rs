use crossterm::{
    self,
    event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    execute,
};
use n_dit::{
    game::{Node, Piece, Sprite},
    grid_map::GridMap,
    ui::{layout::NodeLayout, SuperState},
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

    let layout = NodeLayout::default();

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
        crossterm::terminal::EnterAlternateScreen,
        crossterm::terminal::SetTitle("n_dit"),
        crossterm::event::EnableMouseCapture
    )?;
    crossterm::terminal::enable_raw_mode()?;
    // draw('\\', &node, None);
    // layout.draw_layout(&node, &DrawConfiguration::default())?;
    layout.render(&state)?;
    game_loop(state)?;
    crossterm::terminal::disable_raw_mode()?;
    execute!(
        stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    // draw('/', &node, None);

    Ok(())
}

fn game_loop(mut state: SuperState) -> crossterm::Result<()> {
    let mut action = ' ';
    let layout = NodeLayout::default();

    while action != 'q' {
        // execute!(stdout(), crossterm::cursor::MoveTo(11+3*x,4+2*y))?;
        let event = crossterm::event::read()?;
        match event {
            Event::Key(KeyEvent { code, modifiers }) => {
                let speed = if modifiers.contains(KeyModifiers::CONTROL) {
                    2
                } else {
                    1
                };
                match code {
                    KeyCode::Char('h') => state.move_selected_square(Direction::West, speed),
                    KeyCode::Char('k') => state.move_selected_square(Direction::North, speed),
                    KeyCode::Char('j') => state.move_selected_square(Direction::South, speed),
                    KeyCode::Char('l') => state.move_selected_square(Direction::East, speed),
                    KeyCode::Char('-') => {
                        panic!("Last action was {:?}", action);
                    }
                    KeyCode::Char(char_) => {
                        action = char_;
                    }
                    _ => {}
                }
            }
            Event::Mouse(MouseEvent {
                kind,
                column: _column,
                row: _row,
                modifiers: _,
            }) => {
                if let MouseEventKind::Down(_) = kind {
                    /*if column > 2 {
                        x = (column - 2) /3;
                        y = row/2;
                    }*/
                    // TODO square click
                }
            }
            Event::Resize(_w, _h) => {}
        }

        layout.render(&state)?;
    }
    Ok(())
}
