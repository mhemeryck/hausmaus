/// publish module accepts all incoming events and publishes them to MQTT
use crate::sysfs::FileEvent;

/// handle_messages receives any file events and sends them out over MQTT
pub fn publish_messages(
    rx: std::sync::mpsc::Receiver<FileEvent>,
    mut mqtt_client: rumqttc::Client,
    state_topic_map: &std::collections::HashMap<u8, String>,
) {
    for (device_id, state, duration) in rx {
        let message_str: &str = match state {
            true => "ON",
            false => "OFF",
        };
        if let Some(topic) = state_topic_map.get(&device_id) {
            log::debug!(
                "publishing message for device #{}: {:?}, {:?}, {}",
                device_id,
                state,
                duration,
                topic
            );
            let result = mqtt_client.publish(topic, rumqttc::QoS::AtLeastOnce, false, message_str);
            match result {
                Ok(r) => log::debug!("Everything OK {:?}", r),
                Err(e) => log::debug!("Error {:?}", e),
            }
        }
    }
}
