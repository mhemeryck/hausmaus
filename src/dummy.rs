/// dummy is just a module to output some of the data by logging it.
use tokio;

use crate::sysfs::FileEvent;

/// write_events is just a dummy receiver writing output
pub async fn write_events(mut rx: tokio::sync::mpsc::Receiver<FileEvent>) {
    while let Some((device_id, state, duration)) = rx.recv().await {
        log::info!("Device #{} changed: {:?}, {:?}", device_id, state, duration);
    }
}
