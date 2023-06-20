/// Write incoming messages back by updating the related file system entry
use log;
use std;
use std::io::Write;

pub fn handle_file_command(
    rx: std::sync::mpsc::Receiver<crate::mqtt::MQTTEvent>,
    path_map: &std::collections::HashMap<u8, String>,
) {
    for (device_id, toggle) in rx {
        if let Some(path) = path_map.get(&device_id) {
            log::info!(
                "Received message for device #{} {:?} new path {}",
                device_id,
                toggle,
                path
            );
            if let Ok(mut file) = std::fs::File::create(&path) {
                let content = match toggle {
                    true => "1",
                    false => "0",
                };
                file.write_all(content.as_bytes()).unwrap();
            }
        }
    }
}
