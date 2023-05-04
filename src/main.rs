use env_logger;
use log;
use regex;
use std;
use std::io::{Read, Seek};

const PATH: &str = "/home/mhemeryck/Projects/hausmaus/fixtures";
//const PATH: &str = "/sys/devices/platform/unipi_plc/";
// Check whether we need all devices here or just the digital inputs
const FILENAME_PATTERN: &str =
    //r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/(di|do|ro)_value$";
    r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/di_value$";
const POLL_INTERVAL: u64 = 250;

/// Crawls a directory structure for filenames matching given input
fn crawl(
    dir: &std::path::Path,
    filename_regex: &regex::Regex,
    paths: &mut std::vec::Vec<std::path::PathBuf>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry: std::fs::DirEntry = entry?;
            let path = entry.path();
            if path.is_dir() {
                crawl(&path, filename_regex, paths)?;
            } else if !path.is_symlink() {
                match path.to_str() {
                    Some(path_str) => {
                        if filename_regex.is_match(path_str) {
                            paths.push(path);
                        }
                    }
                    None => {}
                }
            }
        }
    }
    Ok(())
}

/// Wait for toggle on a specific path
fn wait_for_toggle(
    path: String,
    tx: std::sync::mpsc::Sender<(bool, std::time::Duration)>,
) -> std::io::Result<()> {
    log::debug!("Start monitoring path {:?}", path);
    let file = std::fs::File::open(&path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut first_char = [0; 1];

    let mut last_value: Option<bool> = None;
    let mut last_toggle_time: Option<std::time::Instant> = None;

    loop {
        // Go back to first line and read it again
        reader.seek(std::io::SeekFrom::Start(0))?;
        reader.read_exact(&mut first_char)?;
        let first_char = first_char[0] as char;

        // Parse to bool
        let value = match first_char {
            '0' => false,
            '1' => true,
            _ => continue, // skip invalid lines
        };

        // Update last value and last toggle time
        if let Some(last_value) = last_value {
            if last_value != value {
                let toggle_time = last_toggle_time
                    .map(|t| t.elapsed())
                    .unwrap_or_else(|| std::time::Duration::from_secs(0));
                log::debug!(
                    "Toggled for path {:?} ! {:?} / {:?}",
                    path,
                    value,
                    toggle_time
                );
                tx.send((value, toggle_time)).unwrap();
                last_toggle_time = Some(std::time::Instant::now());
            }
        } else {
            last_toggle_time = Some(std::time::Instant::now());
        }
        last_value = Some(value);

        // Go to bed again!
        std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL));
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    // Crawl a folder for paths to watch based on a regex
    let mut paths: std::vec::Vec<std::path::PathBuf> = std::vec::Vec::new();
    let re = regex::Regex::new(FILENAME_PATTERN).unwrap();

    let (tx, rx) = std::sync::mpsc::channel();

    crawl(&std::path::Path::new(&PATH), &re, &mut paths).unwrap();

    let mut handles = std::vec::Vec::with_capacity(paths.len());
    for path in paths {
        log::debug!("Found path: {:?}", path);
        if let Some(path_str) = path.to_str() {
            let path_str = path_str.to_string();
            let path_tx = tx.clone();
            let handle = std::thread::spawn(move || {
                wait_for_toggle(path_str, path_tx).unwrap();
            });
            handles.push(handle);
        }
    }

    for (state, duration) in rx {
        log::info!("changed: {:?}, {:?}", state, duration);
    }

    // Block on the file handles processing
    for handle in handles {
        handle.join().unwrap();
    }
}
