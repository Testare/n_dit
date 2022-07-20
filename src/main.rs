use std::{fs::File, io::stdout, panic};

use crossterm::{self, execute};
use game_core::{
    error, loader, node_from_def, AuthorityGameMaster, GameCommand, GameState, Inventory,
    NetworkGameMaster, NodeDef, Pickup,
};
use n_dit::ui::CrosstermInformant;
use simplelog::{LevelFilter, WriteLogger};

fn main() -> error::Result<()> {
    setup_logging();
    reset_terminal_on_panic(); // If the game panics, we want to bring the terminal back to a normal state
    set_terminal_state()?;

    let state = load_state();
    if std::env::args().any(|arg| arg == "--connect") {
        let mut gm = NetworkGameMaster::connect("localhost:62062")?;
        log::debug!("Setting up informant");
        gm.setup_informant(CrosstermInformant::new);
        log::debug!("Starting to run");
        gm.run();
    } else if std::env::args().any(|arg| arg == "--charmie") {
        n_dit::charmie_ui::start_with_charmie()
    } else {
        let mut gm = AuthorityGameMaster::from(state);
        gm.listen_for_connections(62062).unwrap();
        gm.setup_informant(CrosstermInformant::new);
        gm.run();
    }
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

fn load_state() -> GameState {
    let config = loader::Configuration {
        assets_folder: "./assets".to_string(),
    };
    let node_def = &loader::load_asset_dictionary::<NodeDef>(&config).unwrap()["Node0"];
    let curio_dict = loader::load_asset_dictionary(&config).unwrap();
    let action_dict = loader::load_asset_dictionary(&config).unwrap();
    let node = node_from_def(node_def, debug_inventory(), curio_dict, action_dict).unwrap();
    GameState::from(Some(node))
}

fn debug_inventory() -> Inventory {
    let mut inventory = Inventory::default();
    inventory.pick_up(Pickup::Card("Hack".to_string()));
    inventory.pick_up(Pickup::Card("Hack".to_string()));
    inventory.pick_up(Pickup::Card("Hack 3.0".to_string()));
    inventory.pick_up(Pickup::Card("Andy".to_string()));
    inventory.pick_up(Pickup::Card("Slingshot".to_string()));
    inventory
}

// Can set up more advanced CLI support in the future with clap
fn setup_logging() {
    if std::env::args().any(|arg| arg == "--debug") {
        let file = if std::env::args().any(|arg| arg == "--connect") {
            "debug.connect.log"
        } else {
            "debug.log"
        };
        // Should I do something in the future to make this append style instead of recreate file?
        WriteLogger::init(
            LevelFilter::Debug,
            simplelog::Config::default(),
            File::create(file).unwrap(),
        )
        .unwrap()
    }
}
