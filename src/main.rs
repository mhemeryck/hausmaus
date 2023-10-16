use crossbeam::channel::{bounded, Sender};
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;
use std::path::Path;
use std::thread::{sleep, spawn};
use std::time;

const CHANNEL_SIZE: usize = 16;
const PATH: &str = "/home/mhemeryck/Projects/hausmaus/fixtures/sys/devices/platform/unipi_plc/io_group3/di_3_14/di_value";
const DURATION: time::Duration = time::Duration::from_millis(250);

type DeviceId = u64;

// #[derive(Clone, Debug)]
// enum DeviceType {
//     DigitalInput,
//     DigitalOutput,
//     RelayOutput,
// }

#[derive(Debug)]
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
    FileMonitorErr(Option<char>),
}

fn monitor_file(path: &str, sender: Sender<FileEvent>) -> Result<(), Error> {
    let path = Path::new(path);
    let mut file = File::open(path).map_err(|_e| Error::FileMonitorErr(None))?;
    let mut buf: [u8; 1] = [0; 1];

    loop {
        file.read_exact(&mut buf)
            .map_err(|_e| Error::FileMonitorErr(None))?;
        file.seek(SeekFrom::Start(0))
            .map_err(|_e| Error::FileMonitorErr(None))?;

        let state: State;
        match buf[0] as char {
            '0' => {
                state = State::Off;
            }
            '1' => {
                state = State::On;
            }
            c => {
                return Err(Error::FileMonitorErr(Some(c)));
            }
        };

        let _ = sender.send(FileEvent {
            device_id: 1,
            state,
        });
        sleep(DURATION);
    }
}

fn main() {
    let (s, r) = bounded(CHANNEL_SIZE);

    let handle = spawn(move || monitor_file(PATH, s));
    while let Ok(e) = r.recv() {
        println!("{:?} - {:?}", e.device_id, e.state);
    }

    match handle.join() {
        Ok(Err(e)) => println!("Error in handling sender thread: {:?}", e),
        Err(e) => println!("Found err {:?}", e),
        _ => println!("Finished"),
    };
}
