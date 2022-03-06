use core::time::Duration;
use crossterm::{self, execute};
use n_dit::{
    game::{Card, Node, Pickup, Piece, Sprite},
    grid_map::GridMap,
    ui::{SuperState, UiAction, UserInput},
    Team,
};
use simplelog::{LevelFilter, WriteLogger};
use std::{fs::File, io::stdout, panic, time::Instant};

fn main() -> crossterm::Result<()> {
    setup_logging();
    let state = load_state();
    reset_terminal_on_panic(); // If the game panics, we want to bring the terminal back to a normal state
    set_terminal_state()?;
    game_loop(state)?;
    reset_terminal_state()?;
    Ok(())
}

fn reset_terminal_on_panic() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        log::error!(
            "Panic occurred\n{:#?}\n\nAttempting to reset terminal",
            panic_info
        );

        match reset_terminal_state() {
            Ok(()) => {
                log::info!("Successfully reset terminal")
            }
            Err(e) => {
                log::error!("Failure resetting terminal: {:#?}", e)
            }
        }
        default_hook(panic_info)
    }))
}

fn reset_terminal_state() -> std::io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    execute!(
        stdout(),
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    Ok(())
}

fn set_terminal_state() -> std::io::Result<()> {
    execute!(
        stdout(),
        crossterm::cursor::Hide,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::terminal::SetTitle("n_dit"),
        crossterm::event::EnableMouseCapture
    )?;
    crossterm::terminal::enable_raw_mode()?;
    Ok(())
}

fn load_state() -> SuperState {
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
    node.add_piece(
        (15, 7),
        Card {
            name: "Jeremy".to_string(),
        }
        .into(),
    );
    node.add_piece((6, 1), Pickup::Mon(500).to_piece());
    node.add_piece((6, 2), Piece::AccessPoint);
    SuperState::from(Some(node))
}

fn game_loop(mut state: SuperState) -> crossterm::Result<()> {
    let mut keep_going = true;
    let mut last_action = Instant::now();
    state.render()?;
    while keep_going {
        if let Some(action) = get_next_action(&state, last_action)? {
            if action.is_quit() {
                keep_going = false;
            } else {
                state.apply_action(action).unwrap();
                //layout.render(&state)?;
                state.render()?;
                last_action = Instant::now();
            }
        }
    }
    Ok(())
}

const TIMEOUT: Duration = Duration::from_millis(500);
const MINIMUM_POLLING: Duration = Duration::from_millis(50);

// TODO Could be implemented as an Iterator<Item=Result<UiAction>>, where no-ops are ignored
// instead of returning None
fn get_next_action(
    state: &SuperState,
    last_update: Instant,
) -> crossterm::Result<Option<UiAction>> {
    let event = if state.game_state().waiting_on_player_input() {
        // TODO better handling for when keys are held down/ clearing input queue
        crossterm::event::read()?
    } else {
        let now = Instant::now();
        let update_time = last_update + TIMEOUT;
        // TODO Allow adjustable animation speed, and "skip animation" button
        if now + MINIMUM_POLLING < update_time && crossterm::event::poll(update_time - now)? {
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
