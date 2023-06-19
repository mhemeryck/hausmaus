/// auto contains the main functions related to automation, and links the different parts together
use crate::sysfs::FileEvent;
use tokio;

/// Connect channels from sysfs read -> mqtt publish
pub async fn run_sysfs_to_mqtt(
    mut file_read_rx: tokio::sync::mpsc::Receiver<FileEvent>,
    log_write_tx: tokio::sync::mpsc::Sender<FileEvent>,
    mqtt_publish_tx: tokio::sync::mpsc::Sender<FileEvent>,
) {
    // Simple pass-through, for now
    while let Some(event) = file_read_rx.recv().await {
        // Connect to log write
        log_write_tx.send(event).await.unwrap();

        // Connect to MQTT publish
        mqtt_publish_tx.send(event).await.unwrap();
    }
}

pub async fn run_mqtt_to_sysfs(
    mut mqtt_subscribe_rx: tokio::sync::mpsc::Receiver<crate::mqtt::MQTTEvent>,
    file_write_tx: tokio::sync::mpsc::Sender<crate::mqtt::MQTTEvent>,
) {
    while let Some(msg) = mqtt_subscribe_rx.recv().await {
        log::debug!("Message received {:?}", msg);
        file_write_tx.send(msg).await.unwrap();
    }
}
