/// dummy is just a module to output some of the data by logging it.
use std;

use crate::sysfs::FileEvent;

/// write_events is just a dummy receiver writing output
pub fn write_events(rx: std::sync::mpsc::Receiver<FileEvent>) {
    for (device_id, state, duration) in rx {
        log::info!("Device #{} changed: {:?}, {:?}", device_id, state, duration);
    }
}
