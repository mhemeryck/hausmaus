use crate::sysfs::FileEvent;
/// auto contains the main functions related to automation, and links the different parts together
use std;

enum State {
    Stopped,
    Up,
    Down,
}

struct Cover {
    button_up: u8,
    button_down: u8,
    motor_up: u8,
    motor_down: u8,

    state: State,
    position: u8,
}

/// Connect channels from sysfs read -> mqtt publish
pub fn run_sysfs_to_mqtt(
    file_read_rx: std::sync::mpsc::Receiver<FileEvent>,
    log_write_tx: std::sync::mpsc::Sender<FileEvent>,
    mqtt_publish_tx: std::sync::mpsc::Sender<FileEvent>,
    file_write_tx: std::sync::mpsc::Sender<crate::mqtt::MQTTEvent>,
) {
    let mut cover = Cover {
        button_up: 0,
        button_down: 1,
        motor_up: 2,
        motor_down: 3,
        state: State::Stopped,
        position: 0,
    };

    let mut covers = std::vec![cover];

    let mut rule_map: std::collections::HashMap<u8, usize> = std::collections::HashMap::new();
    for (i, c) in covers.iter().enumerate() {
        rule_map.insert(c.button_up, i);
        rule_map.insert(c.button_down, i);
    }

    // Simple pass-through, for now
    for event in file_read_rx {
        // Connect to log write
        log_write_tx.send(event).unwrap();

        // Connect to MQTT publish
        mqtt_publish_tx.send(event).unwrap();

        let (device_id, _, _) = event;
        if let Some(&cover_id) = rule_map.get(&device_id) {
            log::info!("Found your cover! {}", cover_id);
            if let Some(cover) = covers.get_mut(cover_id) {
                match device_id {
                    up if up == cover.button_up => {
                        log::info!("You wanted to go up!");
                    }
                    down if down == cover.button_down => {
                        log::info!("You wanted to go down!");
                    }
                    _ => {
                        log::info!("I don't know what to do");
                    }
                };
                log::info!("And here's the actual cover {}", cover.button_up);
            }
        }
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
