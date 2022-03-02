use core::time::Duration;
use crossterm::{self, execute};
use n_dit::{
    game::{Node, Piece, Sprite},
    grid_map::GridMap,
    ui::{SuperState, UiAction, UserInput},
    Team,
};
use simplelog::{LevelFilter, WriteLogger};
use std::{fs::File, io::stdout};

fn main() -> crossterm::Result<()> {
    setup_logging();
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

    node.add_sprite(Sprite::new("あ"), vec![(1, 6), (2, 6), (3, 6)])
        .unwrap();
    node.add_sprite(Sprite::new("死"), vec![(4, 6), (5, 6), (5, 7)])
        .unwrap();
    node.add_sprite(Sprite::new("8]"), vec![(3, 3), (3, 4)])
        .unwrap();
    node.add_sprite(
        Sprite::builder()
            .team(Team::EnemyTeam)
            .display("骨")
            .name("Jackson")
            .max_size(4)
            .movement_speed(1)
            .build()
            .unwrap(),
        vec![(14, 4)],
    )
    .unwrap();
    node.add_sprite(Sprite::new("<>"), vec![(14, 6)]).unwrap();
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
        // TODO Allow "rapid next" mode for a shorter timeout, and then
        if crossterm::event::poll(TIMEOUT)? {
            crossterm::event::read()?
        } else {
            return Ok(Some(UiAction::next()));
        }
    };
    Ok(UserInput::from_event(event).and_then(|user_input| state.ui_action_for_input(user_input)))
}

// Can set up more advanced CLI support in the future with clap
fn setup_logging() {
    if std::env::args().any(|arg| arg == "--debug") {
        // Should I do something in the future to make this append style instead of recreate file?
        WriteLogger::init(
            LevelFilter::Debug,
            simplelog::Config::default(),
            File::create("debug.log").unwrap(),
        )
        .unwrap()
    }
    // WriteLogger::init(LevelFilter::Debug, simplelog::Config::default(), File::create("debug.log").unwrap()).unwrap()
}
