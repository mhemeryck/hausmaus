use env_logger;
use log;
use regex;
use std;

// Check whether we need all devices here or just the digital inputs
const FILENAME_PATTERN: &str =
    r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/di_value$";

/// run is the main entry point to start the maus
///
/// It spawns:
/// - all input reader threads
/// - all output write threads
/// - the main automation engine thread to link input events to output events
pub fn run(sysfs_path: &str) {
    // log config
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    log::debug!("Start hausmaus");
    // Crawl a folder for paths to watch based on a regex
    log::debug!("Start crawling path {:?}", sysfs_path);
    let mut paths: std::vec::Vec<std::path::PathBuf> = std::vec::Vec::new();
    let re = regex::Regex::new(FILENAME_PATTERN).unwrap();
    crate::sysfs::crawl(&std::path::Path::new(&sysfs_path), &re, &mut paths).unwrap();
    log::debug!("Finished crawling path {:?}", sysfs_path);

    let mut handles = std::vec::Vec::with_capacity(3);
    // file read channel
    let (file_read_tx, file_read_rx) = std::sync::mpsc::channel();

    //// MQTT write channel let (mqtt_write_tx, mqtt_write_rx) = std::sync::mpsc::channel();

    // dummy log write channel
    let (log_write_tx, log_write_rx) = std::sync::mpsc::channel();

    log::debug!("Start main file event watcher thread");
    let file_event_paths = paths.clone();
    let file_event_tx = file_read_tx.clone();
    let handle = std::thread::spawn(move || {
        crate::sysfs::read::watch_input_file_events(file_event_paths, file_event_tx);
    });
    handles.push(handle);

    log::debug!("Start thread to write to events");
    let handle = std::thread::spawn(move || {
        crate::dummy::write_events(log_write_rx);
    });
    handles.push(handle);

    log::debug!("Start thread to connect all together");
    let handle = std::thread::spawn(move || {
        crate::auto::run(file_read_rx, log_write_tx);
    });
    handles.push(handle);

    // Block on the file handles processing
    for handle in handles {
        handle.join().unwrap();
    }
}
