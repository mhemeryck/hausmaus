/// auto contains the main functions related to automation, and links the different parts together
use crate::sysfs::FileEvent;
use paho_mqtt;
use std;

/// run is the main function bringing all channels together
pub async fn run_sysfs_to_mqtt(
    file_read_rx: std::sync::mpsc::Receiver<FileEvent>,
    log_write_tx: std::sync::mpsc::Sender<FileEvent>,
    mqtt_publish_tx: std::sync::mpsc::Sender<FileEvent>,
) {
    // Simple pass-through, for now
    for event in file_read_rx {
        // quick clone such that we can also send it to mqtt
        let event_clone = event.clone();
        log_write_tx.send(event).unwrap();
        // pass to mqtt tokio publisher; requires to be run in its own task
        let mqtt_tx_clone = mqtt_publish_tx.clone();
        mqtt_tx_clone.send(event_clone).unwrap();
    }
}

pub async fn run_mqtt_to_sysfs(
    mqtt_subscribe_rx: std::sync::mpsc::Receiver<paho_mqtt::Message>,
) {
    for msg in mqtt_subscribe_rx {
        log::info!("Message received {:?}", msg);
    }
}
