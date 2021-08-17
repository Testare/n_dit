use n_dit::{configuration::DrawConfiguration, game::{Sprite, Node}, grid_map::GridMap, ui::Window};
use crossterm::{execute, self, event::{Event, MouseEvent, MouseEventKind, KeyEvent, KeyModifiers, KeyCode} };
use std::io::{stdout, Write};
use n_dit::ui::layout::NodeLayout;

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

    let guy_key = node.add_sprite((1,6), Sprite::new("あ"));
    node.move_sprite((2,6), guy_key.unwrap());
    node.move_sprite((3,6), guy_key.unwrap());
    node.move_sprite((3,7), guy_key.unwrap());

    let guy_key = node.add_sprite((4,6), Sprite::new("死"));
    node.move_sprite((5,6), guy_key.unwrap());
    node.move_sprite((5,7), guy_key.unwrap());

    let guy_key = node.add_sprite((3,3), Sprite::new("8]"));
    node.move_sprite((3,4), guy_key.unwrap());

    node.add_sprite((14,6), Sprite::new("<>"));
    execute!(stdout(), crossterm::terminal::EnterAlternateScreen,
    crossterm::terminal::SetTitle("n_dit"),
    crossterm::event::EnableMouseCapture)?; 
    crossterm::terminal::enable_raw_mode()?;
    // draw('\\', &node, None);
    layout.draw_layout(&node, &DrawConfiguration::default());
    game_loop()?;
    crossterm::terminal::disable_raw_mode()?;
    execute!(stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture)?;
    // draw('/', &node, None);

    Ok(())
}

fn game_loop() -> crossterm::Result<()> {
    let mut x = 1;
    let mut y = 1;
    let mut action = ' ';

    while action != 'q' {
        execute!(stdout(), crossterm::cursor::MoveTo(3+3*x,1+2*y))?;
        let event = crossterm::event::read()?;
        match event {
            Event::Key(KeyEvent {code, modifiers}) => {
                let speed = if modifiers.contains(KeyModifiers::CONTROL) {
                    2
                } else { 1 };
                match code { 
                    KeyCode::Char('h') => if x >= speed {
                        x -= speed;
                    }
                    KeyCode::Char('k') => if y >= speed {
                        y -= speed;
                    }
                    KeyCode::Char('j') => if y < 15 {
                        y += speed;
                    }
                    KeyCode::Char('l') => if x < 70 {
                        x += speed;
                    }
                    KeyCode::Char('-') => {
                        panic!("Last action was {:?}", action);
                    }
                    KeyCode::Char(char_) => {
                        action = char_;
                    }
                    _ => {
                    }
                }
            },
            Event::Mouse(MouseEvent { kind, column, row, modifiers:_ }) => {
                if let MouseEventKind::Down(_) = kind {
                    if column > 2 {
                        x = (column - 2) /3;
                        y = row/2;
                    }
                }
            }
            Event::Resize(_w, _h) => {
            }
        }

        // Draw

    }
    // println!("{:?}", k);
    Ok(())
}

fn draw(border: char, node: &Node, window: Option<Window>) {
    for row in node.draw_node(window, &DrawConfiguration::default()) {
        write!(stdout(), "{0} {1} {0}\n", border, row);
        execute!(stdout(), crossterm::cursor::MoveToColumn(0));
    }
}
