use bevy::prelude::*;
use crossterm::event::Event as CrosstermEvent;
use crossterm::execute;
use std::io::stdout;
use std::ops::Deref;
use std::panic;
use std::sync::mpsc::{self, Receiver, RecvError, SyncSender, TryRecvError};
use std::sync::Mutex;
use std::time::Duration;

pub struct CharmiePlugin;
struct Stub;

#[derive(Component)]
struct TerminalWindow {
    dimensions: Stub,
}

//Resource
#[derive(Resource)]
struct TermConfig {
    exit_key: char,
}

type Input = char;

#[derive(Resource)]
struct TermEventListener {
    rx: Mutex<Receiver<Input>>,
    cancel: SyncSender<()>,
}

impl Deref for TermEventListener {
    type Target = Mutex<Receiver<Input>>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl Drop for TermEventListener {
    fn drop(&mut self) {
        println!("DROPPING");
        self.cancel.send(());
    }
}

struct InputEvent(Input);

impl Default for TermConfig {
    fn default() -> Self {
        TermConfig { exit_key: 'q' }
    }
}

impl Default for TermEventListener {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let (cancel, cancel_notification) = mpsc::sync_channel(0);
        std::thread::spawn(move || {
            let duration = Duration::from_millis(100);
            loop {
                if !matches!(cancel_notification.try_recv(), Err(TryRecvError::Empty)) {
                    println!("LEZ BREAK");
                    break;
                }
                match crossterm::event::poll(duration) {
                    Ok(false) => {
                        // If there is nothing ot read, we check for cancel notifications.
                        // If it is empty and not disconnected, we continue looping, else we die.
                        print!(".");
                    }
                    Err(e) => {
                        log::error!(
                            "Error occurred in crossterm listening thread while polling: {:?}",
                            e
                        );
                        break;
                    }
                    Ok(true) => {
                        match crossterm::event::read() {
                            Ok(crossterm::event::Event::Key(key_event)) => {
                                if let crossterm::event::KeyCode::Char(input) = key_event.code {
                                    println!("Sending key: {}", input);
                                    match tx.send(input) {
                                        Ok(()) => {}
                                        Err(mpsc::SendError(_)) => {
                                            // Other end is dead, close this thread
                                            break;
                                        }
                                    }
                                }
                            }
                            Ok(_) => {
                                println!("Not a key event");
                            }
                            Err(e) => {
                                println!("Error occurred reading {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
        TermEventListener {
            rx: Mutex::new(rx),
            cancel,
        }
    }
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
        app.add_event::<InputEvent>();
        app.add_event::<CrosstermEvent>();
        app.add_system(term_event_listener);
        app.add_system(exit_key);
        println!("Hello from CharmiePlugin")
    }
}

/// Systems

fn create_terminal_window(mut commands: Commands) {
    let terminal_window =
        TerminalWindow::new().expect("Error occured while creating terminal window");
    commands.init_resource::<TermConfig>();
    commands.init_resource::<TermEventListener>();
    commands.spawn(terminal_window);
    println!("Hello!")
}

fn exit_key(
    term_config: Res<TermConfig>,
    mut inputs: EventReader<InputEvent>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    for input in inputs.iter() {
        if input.0 == term_config.exit_key {
            println!("Exit event");
            exit.send(bevy::app::AppExit);
        }
    }
}

fn term_event_listener(term_listener: Res<TermEventListener>, mut inputs: EventWriter<InputEvent>) {
    let lock = term_listener.try_lock();
    match lock {
        Ok(rx) => {
            loop {
                match rx.try_recv() {
                    Ok(event) => {
                        println!("Event received: {:?}", event);
                        inputs.send(InputEvent(event))
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
