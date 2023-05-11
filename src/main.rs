use clap;
use env_logger;
use log;
use regex;
use std;

use hausmaus;

const PATH: &str = "/run/unipi";
// Check whether we need all devices here or just the digital inputs
const FILENAME_PATTERN: &str =
    //r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/(di|do|ro)_value$";
    r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/di_value$";

type FileEvent = (bool, std::time::Duration);

/// write_events is just a dummy receiver writing output
fn write_events(rx: std::sync::mpsc::Receiver<FileEvent>) {
    for (state, duration) in rx {
        log::info!("changed: {:?}, {:?}", state, duration);
    }
}

/// maus is the main function bringing all channels together
fn maus(
    file_read_rx: std::sync::mpsc::Receiver<FileEvent>,
    log_write_tx: std::sync::mpsc::Sender<FileEvent>,
) {
    // Simple pass-through, for now
    for event in file_read_rx {
        log_write_tx.send(event).unwrap();
    }
}

/// START
fn main() {
    // CLI args
    let matches = clap::Command::new("hausmaus")
        .arg(
            clap::Arg::new("sysfs")
                .default_value(PATH)
                .long("sysfs-path")
                .help("SysFS scan path"),
        )
        .get_matches();
    let sysfs_path = matches.get_one::<String>("sysfs").unwrap();

    // log config
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    log::debug!("Start hausmaus");

    // Crawl a folder for paths to watch based on a regex
    log::debug!("Start crawling path {:?}", sysfs_path);
    let mut paths: std::vec::Vec<std::path::PathBuf> = std::vec::Vec::new();
    let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
    hausmaus::sysfs::crawl(&std::path::Path::new(&sysfs_path), &re, &mut paths).unwrap();
    log::debug!("Finished crawling path {:?}", sysfs_path);

    let mut handles = std::vec::Vec::with_capacity(3);
    // file read channel
    let (file_read_tx, file_read_rx) = std::sync::mpsc::channel();

    //// MQTT write channel
    //let (mqtt_write_tx, mqtt_write_rx) = std::sync::mpsc::channel();

    // dummy log write channel
    let (log_write_tx, log_write_rx) = std::sync::mpsc::channel();

    log::debug!("Start main file event watcher thread");
    let file_event_paths = paths.clone();
    let file_event_tx = file_read_tx.clone();
    let handle = std::thread::spawn(move || {
        hausmaus::sysfs::read::watch_input_file_events(file_event_paths, file_event_tx);
    });
    handles.push(handle);

    log::debug!("Start thread to write to events");
    let handle = std::thread::spawn(move || {
        write_events(log_write_rx);
    });
    handles.push(handle);

    log::debug!("Start thread to connect all together");
    let handle = std::thread::spawn(move || {
        maus(file_read_rx, log_write_tx);
    });
    handles.push(handle);

    // Block on the file handles processing
    for handle in handles {
        handle.join().unwrap();
    }
}
