use bevy::prelude::*;
use crossterm::event::Event as CrosstermEvent;
use crossterm::execute;
use std::io::stdout;
use std::panic;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Mutex;
use std::time::Duration;

pub struct CharmiePlugin;
struct Stub;

#[derive(Component)]
struct TerminalWindow {
    dimensions: Stub,
}

#[derive(Resource)]
struct TermConfig {
    exit_key: char,
}

#[derive(Deref, Resource)]
struct TermEventListener {
    rx: Mutex<Receiver<CrosstermEvent>>
}

impl TerminalWindow {
    fn new() -> std::io::Result<TerminalWindow> {
        Self::reset_terminal_on_panic();
        Self::set_terminal_state()?;
        Ok(TerminalWindow { dimensions: Stub })
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

    fn reset_terminal_on_panic() {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            log::error!(
                "Panic occurred\n{:#?}\n\nAttempting to reset terminal",
                panic_info
            );

            match Self::reset_terminal_state() {
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
}


impl Default for TermConfig {
    fn default() -> Self {
        TermConfig { exit_key: 'q' }
    }
}

impl Default for TermEventListener {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let duration = Duration::from_millis(100);
            loop {
                match crossterm::event::poll(duration) {
                    Ok(false) => {}
                    Err(e) => {
                        log::error!(
                            "Error occurred in crossterm listening thread while polling: {:?}",
                            e
                        );
                        break;
                    }
                    Ok(true) => {
                        match crossterm::event::read() {
                            Ok(event) => {
                                match tx.send(event) {
                                    Ok(()) => {}
                                    Err(mpsc::SendError(_)) => {
                                        // Other end is dead, close this thread
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Error occurred reading crossterm events {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
        TermEventListener {
            rx: Mutex::new(rx)
        }
    }
}

impl Drop for TerminalWindow {
    fn drop(&mut self) {
        match Self::reset_terminal_state() {
            Ok(()) => {
                log::info!("Successfully reset terminal from Drop")
            }
            Err(e) => {
                log::error!("Failure resetting terminal from Drop: {:#?}", e)
            }
        }
    }
}

impl Plugin for CharmiePlugin {
    fn build(&self, app: &mut App) {
        // TODO atty check
        app.add_startup_system(create_terminal_window);
        app.add_event::<CrosstermEvent>();
        app.add_system(term_event_listener);
        app.add_system(exit_key);
    }
}

/// Systems

fn create_terminal_window(mut commands: Commands) {
    let terminal_window =
        TerminalWindow::new().expect("Error occured while creating terminal window");
    commands.init_resource::<TermConfig>();
    commands.init_resource::<TermEventListener>();
    commands.spawn(terminal_window);
}

fn exit_key(
    term_config: Res<TermConfig>,
    mut inputs: EventReader<CrosstermEvent>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    for input in inputs.iter() {
        if let CrosstermEvent::Key(crossterm::event::KeyEvent { code, .. }) = input {
            if *code == crossterm::event::KeyCode::Char(term_config.exit_key) {
                exit.send(bevy::app::AppExit);
            }
        }
    }
}

fn term_event_listener(
    term_listener: Res<TermEventListener>,
    mut inputs: EventWriter<CrosstermEvent>,
) {
    let lock = term_listener.try_lock();
    match lock {
        Ok(rx) => {
            loop {
                match rx.try_recv() {
                    Ok(event) => {
                        inputs.send(event);
                    }
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        log::error!("Thread sending input events unexpected closed");
                        break;
                        // TODO attempt error recovery here
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Error with mutex in term_event_listener system: {:?}", e)
        }
    }
}
