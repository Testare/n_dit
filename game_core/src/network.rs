use super::{error, Informant, game_master::{GameMaster, InformantManager}, GameState, event::Event, GameCommand, error::Error, EventLog};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, TryRecvError, Receiver, Sender};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[derive(Debug)]
// TODO Use TcpStream.non_blocking() instead of receiver thread
pub struct NetworkInformant{
    rx: Receiver<GameCommand>,
    write: TcpStream,
    event_buffer: Vec<Event>,
}

#[derive(Debug)]
pub struct NetworkGameMaster {
    write: TcpStream,
    read: BufReader<TcpStream>,
    reliable_state: GameState,
    reliable_event_log: EventLog,
    informants: InformantManager,
    running: bool,
}

#[derive(Debug)]
pub struct ServerConnectionListener {
    rx: Receiver<TcpStream>,
    tx: Sender<()>,
}

impl ServerConnectionListener {
    pub fn start(port: u16) -> std::io::Result<Self> {
        let (connection_tx, rx) = mpsc::channel();
        let (tx, closerx) = mpsc::channel();
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        listener.set_nonblocking(true)?;
        std::thread::spawn(move || {
            let mut listening = true;
            while listening {
                match listener.accept() {
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection to accept
                    }
                    Ok((stream, _addr)) => {
                        if let Err(e) = connection_tx.send(stream) {
                            log::error!("Error occured while accepting connection: {}", e);
                            listening = false;
                        }
                    }
                    Err(e) => {
                        log::error!("Error occured while listneing to for connection: {}", e);
                        listening = false;
                    }
                }
                if !matches!(closerx.try_recv(), Err(TryRecvError::Empty)) {
                    // Either we lost connection and the artifact was dropped, or it was explicitly closed
                    listening = false;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        Ok(ServerConnectionListener { tx, rx })
    }

    pub fn poll_for_connection(&self) -> Result<TcpStream,TryRecvError> { 
        self.rx.try_recv()
    }
}

/**
 * Message from NetworkInformant to NetworkGameManager
 */
#[derive(Debug, Deserialize, Serialize)]
enum NetInformantMessage {
    Event(GameCommand, Event)
}

impl NetworkInformant {

    pub fn new(stream: TcpStream, state: &GameState) -> Self {
        let (tx, rx) = mpsc::channel();
        let write = stream.try_clone().unwrap();
        send_serialized(&write, state).unwrap();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stream);
            let mut connected = true;
            while connected {
                if let Ok(gc) = receive_serialized(&mut reader) {
                    connected = tx.send(gc).is_ok()
                } else {
                    connected = false
                }
            }
        });
        NetworkInformant{
            rx, 
            write,
            event_buffer: Default::default()
        }
    }
}

impl Informant for NetworkInformant {
    fn tick(&mut self, _game_state: &GameState) -> Vec<GameCommand> {
        // TODO Perhaps I should change this to a loop to get multiple commands?
        let tr = self.rx.try_recv();
        match tr {
            Ok(gc) => {
                vec![gc]
            }
            Err(TryRecvError::Empty) => vec![],
            Err(TryRecvError::Disconnected) => vec![GameCommand::Drop],
        }
    }
    fn collect(&mut self, event: &Event, game_state: &GameState) {
        self.event_buffer.push(event.clone());

    }
    fn fail(&mut self, error: &Error, command: &GameCommand, game_state: &GameState) {
        self.event_buffer.clear();
        // TODO send error 
    }
    fn publish(&mut self, command: &GameCommand, game_state: &GameState) {
        for event in self.event_buffer.drain(..) {
            send_serialized(&self.write, &NetInformantMessage::Event(command.clone(), event)).unwrap()
        }
    }
    fn collect_undo(&mut self, event: &Event, game_state: &GameState, event_log: &EventLog) {

    }
}


impl NetworkGameMaster {

    pub fn informants_testing(&mut self) -> &mut InformantManager {
        &mut self.informants
    }

    pub fn setup_informant<I: Informant + 'static, C: FnOnce(&GameState)-> I>(&mut self, construct_informant: C) {
        self.informants.add_informant(construct_informant(&self.reliable_state));
    }

    pub fn run(&mut self) {
        log::debug!{"Running..."};
        self.running = true;
        while self.running {
            let commands = self.informants.tick(&self.reliable_state);
            for (_player_id, gc) in commands {
                log::debug!("Sending command: {:?}", &gc);
                if gc == GameCommand::Drop {
                    self.running = false;
                }
                send_serialized(&self.write, &gc).unwrap();
            }
            if let Some(message) = receive_serialized_nb(&mut self.read).expect("Unexpected error while listening for inputs") {
                match message {
                    NetInformantMessage::Event(gc, event) => {
                        let id = event.id();
                        let event = event.into_change().apply(id, &mut self.reliable_state).unwrap();
                        self.informants.collect(&event, &self.reliable_state);
                        // TODO Publish after calling all events for the same GC
                        // TODO Collect multiple events at once
                        self.informants.publish(&gc, &self.reliable_state);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn connect(address: &str) -> crate::error::Result<NetworkGameMaster> {
        let stream = TcpStream::connect(address)?;

        log::debug!{"Connection successful to [{}]", address};
        let write = stream.try_clone()?;
        let mut read = BufReader::new(stream);
        let reliable_state = receive_serialized(&mut read)?;
        write.set_nonblocking(true)?;

        Ok(NetworkGameMaster {
            write,
            read,
            reliable_state,
            reliable_event_log: EventLog::default(),
            informants: InformantManager::default(),
            running: false,
        })

    }

}

impl GameMaster for NetworkGameMaster {

}

/** Helper function */
fn send_serialized<T: Serialize>(mut stream: &TcpStream, obj: &T) ->error::Result<()> {
    serde_json::to_writer(stream, obj)?;
    writeln!(&mut stream);
    Ok(())
}

fn receive_serialized<T: DeserializeOwned>(read: &mut BufReader<TcpStream>) -> error::Result<T> {
    let mut buffer = String::new();
    read.read_line(&mut buffer)?;
    Ok(serde_json::from_str(&buffer)?)
}

fn receive_serialized_nb<T: DeserializeOwned>(read: &mut BufReader<TcpStream>) -> error::Result<Option<T>> {
    let mut buffer = String::new();
    match read.read_line(&mut buffer) {
        Ok(_) => {
            Ok(serde_json::from_str(&buffer)?)
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            Ok(None)
        }
        Err(err) => Err(err.into())
    }
}