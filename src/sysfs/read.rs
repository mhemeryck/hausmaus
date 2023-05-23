use std;
use std::io::{Read, Seek};

use crate::sysfs;
use crate::sysfs::FileEvent;
use tokio;

const POLL_INTERVAL: u64 = 200;

/// Wait for toggle on a specific path
fn wait_for_toggle(
    path: String,
    device_name: String,
    tx: std::sync::mpsc::Sender<FileEvent>,
) -> std::io::Result<()> {
    log::debug!("Start monitoring path {:?}", path);
    let file = std::fs::File::open(&path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut first_char = [0; 1];

    let mut last_value: Option<bool> = None;
    let mut last_toggle_time: Option<std::time::Instant> = None;

    let device = sysfs::device_from_path(&device_name.as_str(), &path).unwrap();
    let device = std::sync::Arc::new(device);

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
                let device_clone = std::sync::Arc::clone(&device);
                tx.send((device_clone, value, toggle_time)).unwrap();
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

/// watch_input file events is the main block responsible for watching SysFS file events
pub async fn watch_input_file_events(
    paths: std::vec::Vec<std::path::PathBuf>,
    device_name: String,
    tx: std::sync::mpsc::Sender<FileEvent>,
) {
    let mut handles = std::vec::Vec::with_capacity(paths.len());
    for path in paths {
        log::debug!("Found path: {:?}", path);
        if let Some(path_str) = path.to_str() {
            let path_str = path_str.to_string();
            let path_tx = tx.clone();
            let device_name_clone = device_name.clone();
            let handle = tokio::task::spawn_blocking(move || {
                wait_for_toggle(path_str, device_name_clone, path_tx).unwrap();
            });
            handles.push(handle);
        }
    }

    // Block on the file handles processing
    for handle in handles {
        handle.await.unwrap();
    }
}
