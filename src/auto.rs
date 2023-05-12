/// auto contains the main functions related to automation, and links the different parts together

use crate::sysfs::FileEvent;
use std;

/// run is the main function bringing all channels together
pub fn run(
    file_read_rx: std::sync::mpsc::Receiver<FileEvent>,
    log_write_tx: std::sync::mpsc::Sender<FileEvent>,
    mqtt_publish_tx: std::sync::mpsc::Sender<FileEvent>,
) {
    // Simple pass-through, for now
    for event in file_read_rx {
        log_write_tx.send(event).unwrap();
        mqtt_publish_tx.send(event).unwrap();
    }
}
