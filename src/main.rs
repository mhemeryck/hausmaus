use crossbeam::channel::{bounded, Sender};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::io::Read;
use std::io::SeekFrom;
use std::path::Path;
use std::thread::{sleep, spawn};
use std::time;

const CHANNEL_SIZE: usize = 4;
const PATH: &str = "/home/mhemeryck/Projects/hausmaus/fixtures/sys/devices/platform/unipi_plc/io_group3/di_3_14/di_value";
const DURATION: time::Duration = time::Duration::from_millis(250);
const CONFIG_FILE: &str = "config.yaml";
const DEVICE_ID: DeviceId = 1; // TODO: to be determined from crawling file system

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
    FileMonitorErr,
}

#[derive(Debug, Deserialize)]
struct Config {
    push_buttons: Vec<PushButton>,
    lights: Vec<Light>,
    rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
struct PushButton {
    name: String,
    device: String,
}

#[derive(Debug, Deserialize)]
struct Light {
    name: String,
    device: String,
}

#[derive(Debug, Deserialize)]
struct Rule {
    name: String,
    trigger: String,
    action: String,
}

fn monitor_file(path: &str, sender: Sender<FileEvent>) -> Result<(), Error> {
    let path = Path::new(path);
    let mut file = File::open(path).map_err(|_e| Error::FileMonitorErr)?;
    let mut buf: [u8; 1] = [0; 1];

    loop {
        file.read_exact(&mut buf)
            .map_err(|_e| Error::FileMonitorErr)?;
        file.seek(SeekFrom::Start(0))
            .map_err(|_e| Error::FileMonitorErr)?;

        let state = match buf[0] as char {
            '0' => Some(State::Off),
            '1' => Some(State::On),
            _ => None,
        };

        let state = state.ok_or(Error::FileMonitorErr)?;
        sender
            .send(FileEvent {
                device_id: DEVICE_ID,
                state,
            })
            .map_err(|_e| Error::FileMonitorErr)?;
        sleep(DURATION);
    }
}

fn main() {
    // YAML config
    let config = read_to_string(CONFIG_FILE).unwrap();
    let deserialized_config: Config = serde_yaml::from_str(&config).unwrap();
    println!("Found config {:?}", deserialized_config);

    // TODO: crawl for devices
    let mut config_map = HashMap::new();
    config_map.insert(DEVICE_ID, "di_3_14");

    // File monitor thread
    let (s, r) = bounded(CHANNEL_SIZE);
    let (s1, r1) = bounded(CHANNEL_SIZE);

    let mut handles = Vec::new();
    let handle = spawn(move || monitor_file(PATH, s));
    handles.push(handle);
    let handle = spawn(move || {
        while let Ok(e) = r1.recv() {
            println!("handle from second thread: {:?}", e);
        }
        Ok(())
    });
    handles.push(handle);

    while let Ok(e) = r.recv() {
        println!("{:?} - {:?}", e.device_id, e.state);
        // TODO: rewrap before sending
        s1.send(e).unwrap();
    }
    for handle in handles {
        handle.join().unwrap();
    }

    // handle2.join().unwrap();
    // match handle.join() {
    //     Ok(Err(e)) => println!("Error in handling sender thread: {:?}", e),
    //     Err(e) => println!("Found err {:?}", e),
    //     _ => println!("Finished"),
    // };
}
