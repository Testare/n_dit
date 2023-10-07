#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::single_match
)]
pub mod base_ui;
pub mod configuration;
mod fx;
pub mod input_event;
mod key_map;
pub mod layout;
pub mod node_ui;
mod render;
// Temporary module to run the game while we get it set up
pub mod animation;
pub mod demo;

use game_core::NDitCoreSet;
pub use key_map::{KeyMap, Submap};

pub mod prelude {
    pub use bevy_query_ext::prelude::*;
    pub use game_core::prelude::*;

    pub use super::input_event::{CrosstermEvent, KeyEvent, MouseEvent};
}

use std::io::stdout;
use std::panic;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Mutex;
use std::time::Duration;

use charmi::{CharmiLoader, CharmiaLoader, CharmieActor, CharmieAnimation};
use crossterm::execute;
use input_event::{sys_mouse_tty, CrosstermEvent, MouseEventTty};
use prelude::*;

use self::configuration::DrawConfiguration;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum TerminalFocusMode {
    #[default]
    Node,
}

pub struct CharmiePlugin;

#[derive(Debug, Resource, getset::Setters, getset::Getters)]
pub struct TerminalWindow {
    #[getset(get = "pub", set = "pub")]
    render_target: Option<Entity>,
    #[getset(get = "pub", set)]
    size: UVec2,
}

#[derive(Resource)]
struct TermConfig {
    exit_key: char,
}

#[derive(Deref, Resource)]
struct TermEventListener {
    rx: Mutex<Receiver<crossterm::event::Event>>,
}

impl TerminalWindow {
    pub fn width(&self) -> usize {
        self.size.x as usize
    }

    pub fn height(&self) -> usize {
        self.size.y as usize
    }

    pub fn scroll_x(&self) -> usize {
        0 // TODO Move to Node logic
    }
    pub fn scroll_y(&self) -> usize {
        0
    }

    fn new() -> std::io::Result<TerminalWindow> {
        Self::reset_terminal_on_panic();
        Self::set_terminal_state()?;
        let (size_width, size_height) = crossterm::terminal::size()?;
        Ok(TerminalWindow {
            render_target: None,
            size: UVec2 {
                x: size_width as u32,
                y: size_height as u32,
            },
        })
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
            crossterm::terminal::SetTitle("Common Quest"),
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
                },
                Err(e) => {
                    log::error!("Failure resetting terminal: {:#?}", e)
                },
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
                    Ok(false) => {},
                    Err(e) => {
                        log::error!(
                            "Error occurred in crossterm listening thread while polling: {:?}",
                            e
                        );
                        break;
                    },
                    Ok(true) => {
                        match crossterm::event::read() {
                            Ok(event) => {
                                match tx.send(event) {
                                    Ok(()) => {},
                                    Err(mpsc::SendError(_)) => {
                                        // Other end is dead, close this thread
                                        break;
                                    },
                                }
                            },
                            Err(e) => {
                                log::error!("Error occurred reading crossterm events {:?}", e);
                                break;
                            },
                        }
                    },
                }
            }
        });
        TermEventListener { rx: Mutex::new(rx) }
    }
}

impl Default for TerminalWindow {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Drop for TerminalWindow {
    fn drop(&mut self) {
        match Self::reset_terminal_state() {
            Ok(()) => {
                log::info!("Successfully reset terminal from Drop")
            },
            Err(e) => {
                log::error!("Failure resetting terminal from Drop: {:#?}", e)
            },
        }
    }
}

impl Plugin for CharmiePlugin {
    fn build(&self, app: &mut App) {
        // TODO atty check
        app.init_resource::<TermConfig>()
            .init_resource::<TermEventListener>()
            .init_resource::<TerminalWindow>()
            .init_resource::<fx::Fx>()
            .init_resource::<DrawConfiguration>()
            .add_asset::<CharmieAnimation>()
            .add_asset::<CharmieActor>()
            .add_state::<TerminalFocusMode>()
            .add_asset_loader(CharmiaLoader)
            .add_asset_loader(CharmiLoader)
            .add_plugins((
                base_ui::BaseUiPlugin,
                layout::TaffyTuiLayoutPlugin,
                node_ui::NodeUiPlugin,
                render::RenderTtyPlugin,
            ))
            .add_event::<CrosstermEvent>()
            .add_event::<KeyEvent>()
            .add_event::<MouseEvent>()
            .add_event::<MouseEventTty>()
            .add_systems(PreUpdate, sys_mouse_tty)
            .add_systems(Startup, fx::sys_init_fx)
            .add_systems(First, term_event_listener.in_set(NDitCoreSet::RawInputs))
            .add_systems(Update, terminal_size_adjustment)
            .add_systems(Last, exit_key);
    }
}

/// Systems

fn exit_key(
    term_config: Res<TermConfig>,
    mut ev_key: EventReader<KeyEvent>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    for input_event::KeyEvent { code, .. } in ev_key.iter() {
        if *code == input_event::KeyCode::Char(term_config.exit_key) {
            exit.send(bevy::app::AppExit);
        }
    }
}

/// Writes out crossterm events
/// KeyEvent and MouseEvent are written as their own events,
/// the rest are written as crossterm::event::Event's (but we
/// provide "CrosstermEvent" as a convenient way to refer to them)
fn term_event_listener(
    term_listener: Res<TermEventListener>,
    mut ev_crossterm: EventWriter<CrosstermEvent>,
    mut ev_mouse: EventWriter<MouseEvent>,
    mut ev_key: EventWriter<KeyEvent>,
) {
    let lock = term_listener.try_lock();
    match lock {
        Ok(rx) => {
            loop {
                match rx.try_recv() {
                    Ok(crossterm::event::Event::Mouse(mouse_event)) => {
                        ev_mouse.send(MouseEvent(mouse_event));
                    },
                    Ok(crossterm::event::Event::Key(key_event)) => {
                        ev_key.send(key_event.into());
                    },
                    Ok(event) => {
                        ev_crossterm.send(CrosstermEvent(event));
                    },
                    Err(TryRecvError::Empty) => {
                        break;
                    },
                    Err(TryRecvError::Disconnected) => {
                        log::error!("Thread sending input events unexpected closed");
                        break;
                        // TODO attempt error recovery here
                    },
                }
            }
        },
        Err(e) => {
            log::error!("Error with mutex in term_event_listener system: {:?}", e)
        },
    }
}

fn terminal_size_adjustment(
    mut inputs: EventReader<CrosstermEvent>,
    mut window: ResMut<TerminalWindow>,
) {
    for input in inputs.iter() {
        if let crossterm::event::Event::Resize(width, height) = **input {
            window.set_size(UVec2 {
                x: width as u32,
                y: height as u32,
            });
        }
    }
}
