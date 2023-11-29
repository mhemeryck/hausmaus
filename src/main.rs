use crossbeam::channel::Receiver;
use crossbeam::channel::{bounded, Sender};
// use serde::{Deserialize, Serialize};
// use serde_yaml;
// use std::collections::HashMap;
use crossbeam_channel::{select, tick};
use futures::stream::FilterMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;
// use std::path::Path;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::{self, Duration};

const CHANNEL_SIZE: usize = 4;
const PATH: &str = "/home/mhemeryck/Projects/hausmaus/fixtures/sys/devices/platform/unipi_plc/io_group3/di_3_14/di_value"; // TODO: as config
const OUTPUT_PATH : &str = "/home/mhemeryck/Projects/hausmaus/fixtures/sys/devices/platform/unipi_plc/io_group1/do_1_01/do_value"; // TODO: as config
const DURATION: time::Duration = time::Duration::from_millis(250);
// const CONFIG_FILE: &str = "config.yaml";
const DEVICE_ID: DeviceId = 1; // TODO: to be determined from crawling file system
const OUTPUT_DEVICE_ID: DeviceId = 2; // TODO: to be determined from file system

type DeviceId = u64;

// #[derive(Clone, Debug)]
// enum DeviceType {
//     DigitalInput,
//     DigitalOutput,
//     RelayOutput,
// }

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    Off,
    On,
}

#[derive(Debug)]
struct FileEvent {
    device_id: DeviceId,
    state: State,
}

#[derive(Debug)]
enum Error {
    FileReadErr,
    FileWriteErr,
    FileUnexpectedContentErr,
}

// #[derive(Debug, Deserialize)]
// struct Config {
//     push_buttons: Vec<PushButton>,
//     lights: Vec<Light>,
//     rules: Vec<Rule>,
// }

// #[derive(Debug, Deserialize)]
// struct PushButton {
//     name: String,
//     device: String,
// }

// #[derive(Debug, Deserialize)]
// struct Light {
//     name: String,
//     device: String,
// }

// #[derive(Debug, Deserialize)]
// struct Rule {
//     name: String,
//     trigger: String,
//     action: String,
// }

#[derive(Debug)]
enum Event {
    Toggle,
}

struct PushButton {
    filename: String,
}

impl PushButton {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }

    pub fn process(
        self,
        event_rx: Receiver<Event>,
        event_tx: Sender<FileEvent>,
    ) -> Result<JoinHandle<()>, Error> {
        let ticker = tick(Duration::from_millis(250));

        // file business
        let mut readfile = File::open(&self.filename).map_err(|_e| Error::FileReadErr)?;
        let mut readbuf: [u8; 1] = [0; 1];

        let mut prev_state: Option<State> = None;

        let handle = spawn(move || loop {
            select! {
            recv(event_rx) -> _msg => {
                let mut writefile = File::open(PATH).unwrap();
                let mut writebuf: [u8; 1] = [0; 1];
                // file.read_exact(&mut buf).map_err(|_| Error::FileWriteErr)?;
                writefile.read_exact(&mut writebuf).unwrap();

                let state = match writebuf[0] as char {
                    '0' => Some(State::Off),
                    '1' => Some(State::On),
                    _ => None,
                };

                let state = state.unwrap();

                let mut outfile = File::create(&self.filename).unwrap();
                match state {
                    State::On => {
                        println!("Convert file from on to off");
                        outfile.write(b"0").unwrap();
                    }
                    State::Off => {
                        println!("Convert file from off to on");
                        outfile.write(b"1").unwrap();
                    }
                }
            },

            recv(ticker) -> _msg => {
                    readfile.read_exact(&mut readbuf).unwrap();
                    readfile.seek(SeekFrom::Start(0)).unwrap();

                    let state = match readbuf[0] as char {
                        '0' => Some(State::Off),
                        '1' => Some(State::On),
                        _ => None,
                    };

                    let state = state.unwrap();
                    match (prev_state, state) {
                        (Some(State::On), State::Off) | (Some(State::Off), State::On) => {
                            println!("File update during tick!");
                            event_tx
                                .send(FileEvent {
                                    device_id: DEVICE_ID,
                                    state,
                                }).unwrap();
                        }
                        _ => {}
                    }
                    prev_state = Some(state);

                },
            }
        });
        Ok(handle)
    }
}

#[derive(Debug)]
enum DeviceEvent {
    TurnedOn,
    TurnedOff,
    // On(u64),
    // Off(u64),
}

enum DeviceCommand {
    On,
    Off,
    Toggle,
}

trait Device {
    fn new(path: &'static str) -> Self;

    fn monitor(
        self,
        sender: Sender<DeviceEvent>,
        receiver: Receiver<DeviceCommand>,
    ) -> JoinHandle<()>;
}

struct DigitalInput {
    path: &'static str,
}

struct DigitalOutput {
    path: &'static str,
    state: Option<State>,
}

impl Device for DigitalOutput {
    fn new(path: &'static str) -> DigitalOutput {
        DigitalOutput { path, state: None }
    }

    fn monitor(
        self,
        sender: Sender<DeviceEvent>,
        receiver: Receiver<DeviceCommand>,
    ) -> JoinHandle<()> {
        let ticker = tick(DURATION);

        spawn(move || loop {
            select! {
                recv(receiver) -> msg => {
                    match msg {
                        Ok(DeviceCommand::On) => println!("Turning on ..."),
                        Ok(DeviceCommand::Off) => println!("Turning off ..."),
                        Ok(DeviceCommand::Toggle) => println!("Toggling ..."),
                        Err(_) => todo!(),
                    }
                },
                recv(ticker) -> _msg => {
                    println!("Time is ticking away ...");
                    sender.send(DeviceEvent::TurnedOn).unwrap();
                }
            }
        })
    }
}

// File state mapper allows wraps the sysfs operations
struct FileStateMapper {
    path: &'static str,
    state: Option<State>,
}

impl FileStateMapper {
    fn new(path: &'static str) -> Self {
        FileStateMapper { path, state: None }
    }

    fn monitor(
        mut self,
        sender: Sender<DeviceEvent>,
        receiver: Receiver<DeviceCommand>,
    ) -> JoinHandle<()> {
        let ticker = tick(DURATION);

        spawn(move || loop {
            select! {
                recv(receiver) -> msg => match msg {
                    Ok(cmd) =>  self.handle(cmd).unwrap(),
                    Err(_e) => (),
                },
                recv(ticker) -> _msg => {
                    self.read().unwrap();
                    match self.state {
                        Some(State::On) => sender.send(DeviceEvent::TurnedOn).unwrap(),
                        Some(State::Off) => sender.send(DeviceEvent::TurnedOff).unwrap(),
                        _ => ()

                    }
                }
            }
        })
    }

    fn handle(&mut self, cmd: DeviceCommand) -> Result<(), Error> {
        match cmd {
            DeviceCommand::On => self.write(State::On),
            DeviceCommand::Off => self.write(State::Off),
            DeviceCommand::Toggle => {
                println!("{:?}", self.state);
                match self.state {
                    Some(State::Off) => self.write(State::On),
                    Some(State::On) => self.write(State::Off),
                    None => {
                        self.read()?;
                        match self.state {
                            None => Err(Error::FileReadErr),
                            _ => self.handle(cmd),
                        }
                    }
                }
            }
        }
    }

    // Read the current state
    fn read(&mut self) -> Result<(), Error> {
        let mut file = File::open(self.path).map_err(|_e| Error::FileReadErr)?;
        let mut buf: [u8; 1] = [0; 1];

        // Read current value
        file.read_exact(&mut buf).map_err(|_| Error::FileReadErr)?;

        self.state = match buf[0] as char {
            '0' => Some(State::Off),
            '1' => Some(State::On),
            _ => None,
        };

        // Err in case we couldn't read the value
        self.state.ok_or_else(|| Error::FileUnexpectedContentErr)?;

        Ok(())
    }

    fn write(&mut self, state: State) -> Result<(), Error> {
        match (self.state, state) {
            (None, _) => {
                // No previously known state, so read it
                self.read()?;
                // Read it again now
                match self.state {
                    // Still nothing, break up
                    None => Err(Error::FileReadErr),
                    // Read again
                    _ => self.write(state),
                }
            }

            // Flip, where required
            (Some(prev), next) if prev != next => self.write_new_state(next),
            // Already the correct state, so nothing to do
            (Some(_), _) => Ok(()),
        }
    }

    fn write_new_state(&mut self, state: State) -> Result<(), Error> {
        let mut file = File::create(self.path).map_err(|_| Error::FileWriteErr)?;
        match state {
            State::On => {
                file.write(b"1").map_err(|_| Error::FileWriteErr)?;
                Ok(())
            }
            State::Off => {
                file.write(b"0").map_err(|_| Error::FileWriteErr)?;
                Ok(())
            }
        }
    }
}

// fn monitor_file(path: &str, sender: Sender<FileEvent>) -> Result<(), Error> {
//     let path = Path::new(path);
//     let mut file = File::open(path).map_err(|_e| Error::FileMonitorErr)?;
//     let mut buf: [u8; 1] = [0; 1];

//     let mut prev_state: Option<State> = None;

//     loop {
//         sleep(DURATION);
//     }
// }

// fn toggle_light(path: &str) -> Result<(), Error> {
//     let mut file = File::open(path).map_err(|_| Error::FileWriteErr)?;
//     let mut buf: [u8; 1] = [0; 1];

//     file.read_exact(&mut buf).map_err(|_| Error::FileWriteErr)?;

//     let state = match buf[0] as char {
//         '0' => Some(State::Off),
//         '1' => Some(State::On),
//         _ => None,
//     };

//     let state = state.ok_or(Error::FileWriteErr)?;

//     let mut file = File::create(path).map_err(|_| Error::FileWriteErr)?;
//     match state {
//         State::On => {
//             println!("Convert file from on to off");
//             file.write(b"0").map_err(|_| Error::FileWriteErr)?;
//         }
//         State::Off => {
//             println!("Convert file from off to on");
//             file.write(b"1").map_err(|_| Error::FileWriteErr)?;
//         }
//     }

//     Ok(())
// }

fn main() {
    let (s_command, r_command) = bounded(CHANNEL_SIZE);
    let (s_event, r_event) = bounded(CHANNEL_SIZE);

    let mut handles = Vec::new();
    let fm = FileStateMapper::new(OUTPUT_PATH);

    let handle = fm.monitor(s_event, r_command);
    handles.push(handle);

    let handle = spawn(move || {
        while let Ok(e) = r_event.recv() {
            println!("received event from file state mapper: {:?}", e);
        }
    });
    handles.push(handle);

    let handle = spawn(move || {
        for _ in 0..=10 {
            s_command.send(DeviceCommand::Toggle).unwrap();
            sleep(time::Duration::from_secs(1));
        }
    });
    handles.push(handle);

    for handle in handles {
        handle.join().unwrap();
    }
}

// fn main() {
//     // // YAML config
//     // let config = read_to_string(CONFIG_FILE).unwrap();
//     // let deserialized_config: Config = serde_yaml::from_str(&config).unwrap();
//     // println!("Found config {:?}", deserialized_config);

//     // // TODO: crawl for devices
//     // let mut config_map = HashMap::new();
//     // config_map.insert(DEVICE_ID, "di_3_14");

//     // File monitor thread
//     let mut handles = Vec::new();

//     let push_button = PushButton::new(PATH);

//     let (tx, rx) = bounded(CHANNEL_SIZE);
//     let (s, r) = bounded(CHANNEL_SIZE);
//     let handle = push_button.process(r, tx);
//     handles.push(handle.unwrap());

//     let handle = spawn(move || {
//         while let Ok(e) = rx.recv() {
//             if e.device_id == DEVICE_ID {
//                 println!("toggle path: {:?}", e);
//                 // toggle_light(OUTPUT_PATH).unwrap();
//             }
//         }
//     });
//     handles.push(handle);
//     for _ in 0..10 {
//         s.send(Event::Toggle).unwrap();
//         sleep(Duration::from_secs(1));
//     }

//     for handle in handles {
//         handle.join().unwrap();
//     }

//     // let (s, r) = bounded(CHANNEL_SIZE);
//     // // let (s1, r1) = bounded(CHANNEL_SIZE);

//     // // Main automation thread
//     // let handle = spawn(move || {
//     //     while let Ok(e) = r.recv() {
//     //         println!("{:?}", e);
//     //         if e.device_id == DEVICE_ID {
//     //             println!("toggle path: {:?}", e);
//     //             toggle_light(OUTPUT_PATH).unwrap();
//     //         }
//     //     }
//     //     Ok(())
//     // });
//     // handles.push(handle);

//     // // while let Ok(e) = r.recv() {
//     // //     println!("{:?} - {:?}", e.device_id, e.state);
//     // //     // TODO: rewrap before sending
//     // //     s1.send(e).unwrap();
//     // // }

//     // // handle2.join().unwrap();
//     // // match handle.join() {
//     // //     Ok(Err(e)) => println!("Error in handling sender thread: {:?}", e),
//     // //     Err(e) => println!("Found err {:?}", e),
//     // //     _ => println!("Finished"),
//     // // };
// }
