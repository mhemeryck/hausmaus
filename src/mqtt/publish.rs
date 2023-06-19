/// publish module accepts all incoming events and publishes them to MQTT
use log;
use rumqttc;
use std;

use crate::sysfs::FileEvent;

/// handle_messages receives any file events and sends them out over MQTT
pub async fn publish_messages(
    mut rx: tokio::sync::mpsc::Receiver<FileEvent>,
    mqtt_client: rumqttc::AsyncClient,
    state_topic_map: &std::collections::HashMap<u8, String>,
) {
    while let Some((device_id, state, duration)) = rx.recv().await {
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
            let result = mqtt_client
                .publish(topic, rumqttc::QoS::AtLeastOnce, false, message_str)
                .await;
            match result {
                Ok(r) => log::debug!("Everything OK {:?}", r),
                Err(e) => log::debug!("Error {:?}", e),
            }
        }
    }
}
