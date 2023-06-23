use log;
use std;
use std::io::{Read, Seek};

use crate::sysfs::FileEvent;

const POLL_INTERVAL: u64 = 200;

/// Wait for toggle on a specific path
fn wait_for_toggle(
    device: crate::device::Device,
    tx: std::sync::mpsc::Sender<FileEvent>,
) -> std::io::Result<()> {
    log::debug!("Start monitoring path {:?}", device.path);
    let file = std::fs::File::open(&device.path)?;
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
                    "Toggled for device #{} path {:?} ! {:?} / {:?}",
                    device.id,
                    device.path,
                    value,
                    toggle_time
                );
                tx.send((device.id, value, toggle_time)).unwrap();
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
pub fn watch_input_file_events(
    devices: std::vec::Vec<crate::device::Device>,
    tx: std::sync::mpsc::Sender<FileEvent>,
) {
    let mut handles = std::vec::Vec::with_capacity(devices.len());
    for device in devices {
        let path_tx = tx.clone();
        let handle = std::thread::spawn(move || {
            wait_for_toggle(device, path_tx).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
