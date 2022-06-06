use core::time::Duration;
use std::{fs::File, io::stdout, panic, time::Instant};

use crossterm::{self, execute};
use game_core::{loader, Card, Curio, GridMap, Node, Pickup, Sprite, Team};
use n_dit::ui::{SuperState, UiAction, UserInput};
use simplelog::{LevelFilter, WriteLogger};

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

    // node.add_curio(Curio::new("あ"), vec![(1, 6), (2, 6), (3, 6)])
    node.add_curio(
        Curio::builder()
            .display("あ")
            .name("Horus")
            .action("Bite")
            .max_size(3)
            .movement_speed(2)
            .build()
            .unwrap(),
        vec![(1, 6), (2, 6), (3, 6)],
    )
    .unwrap();
    node.add_curio(Curio::new("死"), vec![(4, 6), (5, 6), (5, 7)])
        .unwrap();
    node.add_curio(Curio::new("8]"), vec![(3, 3), (3, 4)])
        .unwrap();
    node.add_curio(
        Curio::builder()
            .team(Team::EnemyTeam)
            .display("骨")
            .name("Jackson")
            .action("Bite")
            .max_size(4)
            .movement_speed(7)
            .build()
            .unwrap(),
        vec![(14, 4)],
    )
    .unwrap();
    node.add_curio(Curio::new("<>"), vec![(14, 6)]).unwrap();
    node.add_sprite(
        (15, 7),
        Card {
            name: "Jeremy".to_string(),
        }
        .into(),
    );
    node.add_sprite((6, 1), Pickup::Mon(500).to_sprite());
    node.add_sprite((6, 2), Sprite::AccessPoint);
    /* let action_dictionary_string =
        std::fs::read_to_string("./assets/nightfall/action_dictionary.json").unwrap();
    let dict = serde_json::from_str(action_dictionary_string.as_ref()).unwrap();*/
    let config = loader::Configuration {
        assets_folder: "./assets".to_string(),
    };
    let action_dictionary = loader::load_action_dictionaries(config);
    node.add_action_dictionary(action_dictionary);
    SuperState::from(Some(node))
}

fn game_loop(mut state: SuperState) -> crossterm::Result<()> {
    let mut keep_going = true;
    let mut last_action;
    // state.render()?;
    while keep_going {
        last_action = Instant::now();
        state.render()?;
        for action in get_next_actions(&state, last_action)? {
            if action.is_quit() {
                keep_going = false;
            } else {
                state.apply_action(action).unwrap(); // TODO handle this better?
            }
        }
    }
    Ok(())
}

const TIMEOUT: Duration = Duration::from_millis(500);
const MINIMUM_POLLING: Duration = Duration::from_millis(50);

// TODO Could be implemented as an Iterator<Item=Result<UiAction>>, where no-ops are ignored
// instead of returning None
fn get_next_actions(state: &SuperState, last_update: Instant) -> crossterm::Result<Vec<UiAction>> {
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
            return Ok(vec![UiAction::next()]);
        }
    };
    Ok(UserInput::from_event(event)
        .into_iter()
        .flat_map(|user_input| state.ui_actions_for_input(user_input).into_iter())
        .collect())
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
}
