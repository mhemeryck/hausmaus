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

fn monitor_file(sender: Sender<FileEvent>) {
    // let mut state = State::On;

    let path = Path::new(PATH);
    let mut file = File::open(path).expect("Could not read path");
    let mut buf: [u8; 1] = [0; 1];

    loop {
        file.read_exact(&mut buf)
            .expect("Could not read first byte of path");
        file.seek(SeekFrom::Start(0)).unwrap();

        let state = match buf[0] as char {
            '0' => State::Off,
            '1' => State::On,
            _ => panic!("Can't read this!"),
        };

        // // toggle state
        // state = match state {
        //     State::On => State::Off,
        //     State::Off => State::On,
        // };

        let file_event = FileEvent {
            device_id: 1,
            state,
        };

        let _ = sender.send(file_event);
        sleep(DURATION);
    }
}

fn main() {
    let (s, r) = bounded(CHANNEL_SIZE);

    let handle = spawn(move || monitor_file(s));
    while let Ok(e) = r.recv() {
        println!("{:?}", e);
    }

    handle.join().expect("Could not run file monitor job");
}
