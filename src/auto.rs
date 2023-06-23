/// auto contains the main functions related to automation, and links the different parts together
use crate::sysfs::FileEvent;

/// Connect channels from sysfs read -> mqtt publish
pub fn run_sysfs_to_mqtt(
    file_read_rx: std::sync::mpsc::Receiver<FileEvent>,
    log_write_tx: std::sync::mpsc::Sender<FileEvent>,
    mqtt_publish_tx: std::sync::mpsc::Sender<FileEvent>,
) {
    // Simple pass-through, for now
    for event in file_read_rx {
        // Connect to log write
        log_write_tx.send(event).unwrap();

        // Connect to MQTT publish
        mqtt_publish_tx.send(event).unwrap();
    }
}

pub fn run_mqtt_to_sysfs(
    mqtt_subscribe_rx: std::sync::mpsc::Receiver<crate::mqtt::MQTTEvent>,
    file_write_tx: std::sync::mpsc::Sender<crate::mqtt::MQTTEvent>,
) {
    for msg in mqtt_subscribe_rx {
        log::debug!("Message received {:?}", msg);
        file_write_tx.send(msg).unwrap();
    }
}
