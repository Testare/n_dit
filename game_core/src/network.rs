use super::{Informant, game_master::{GameMaster, InformantManager}, GameState, event::Event, GameCommand, error::Error, EventLog};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, TryRecvError, Receiver, Sender};
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

#[derive(Debug)]
// TODO Use TcpStream.non_blocking() instead of receiver thread
pub struct NetworkInformant(Receiver<GameCommand>, TcpStream);

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

    // Might not be necessary: Dropping this should implicitly close connection
    pub fn close(&self) {
        self.tx.send(());
    }
}


impl NetworkInformant {

    pub fn new(mut stream: TcpStream, state: &GameState) -> Self {
        let (tx, rx) = mpsc::channel();
        let writer = stream.try_clone().unwrap();
        serde_json::to_writer(&writer, state).unwrap();
        writeln!(&writer);
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stream);
            let mut connected = true;
            while connected {
                let mut command_str = String::new();
                if reader.read_line(&mut command_str).is_ok() {
                    if let Ok(gc) = serde_json::from_str(&command_str) {
                        connected = tx.send(gc).is_ok();
                        continue;
                    } // else Should we inform user command is malformed?
                }
                connected = false;
            }
        });
        NetworkInformant(rx, writer)
    }
}

impl Informant for NetworkInformant {
    fn tick(&mut self, _game_state: &GameState) -> Option<GameCommand>{
        let tr = self.0.try_recv();
        match tr {
            Ok(gc) => {
                Some(gc)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(GameCommand::Drop),
        }
    }
    fn collect(&mut self, event: &Event, game_state: &GameState) {

    }
    fn fail(&mut self, error: &Error, command: &GameCommand, game_state: &GameState) {

    }
    fn publish(&mut self, command: &GameCommand, game_state: &GameState) {

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
                serde_json::to_writer(&self.write, &gc).unwrap();
                writeln!(&self.write).unwrap();
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn connect(address: &str) -> crate::error::Result<NetworkGameMaster> {
        let stream = TcpStream::connect(address)?;

        log::debug!{"Connection successful"};
        let write = stream.try_clone()?;
        let mut read = BufReader::new(stream);
        let mut state_str = String::new();
        read.read_line(&mut state_str)?;
        log::debug!{"Received state: {}", state_str};

        let reliable_state = serde_json::from_str(&state_str).unwrap();

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