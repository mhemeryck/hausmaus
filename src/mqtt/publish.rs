/// publish module accepts all incoming events and publishes them to MQTT
use log;
use paho_mqtt;
use std;

use crate::sysfs::FileEvent;

/// handle_messages receives any file events and sends them out over MQTT
pub async fn publish_messages(
    rx: std::sync::mpsc::Receiver<FileEvent>,
    mqtt_client: &paho_mqtt::AsyncClient,
    state_topic_map: &std::collections::HashMap<u8, String>,
) -> paho_mqtt::errors::Result<()> {
    for (device_id, state, duration) in rx {
        log::debug!(
            "publishing message for device #{}: {:?}, {:?}",
            device_id,
            state,
            duration
        );
        let message_str = match state {
            true => "ON",
            false => "OFF",
        };
        if let Some(topic) = state_topic_map.get(&device_id) {
            let message = paho_mqtt::Message::new(topic, message_str.as_bytes(), paho_mqtt::QOS_2);
            mqtt_client.publish(message).await?;
        }
    }
    Ok(())
}
