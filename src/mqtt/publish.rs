/// publish module accepts all incoming events and publishes them to MQTT
use log;
use paho_mqtt;

use crate::sysfs::FileEvent;

const MQTT_TOPIC: &str = "foo";

/// handle_messages receives any file events and sends them out over MQTT
pub async fn publish_messages(
    mut rx: tokio::sync::mpsc::Receiver<FileEvent>,
    mqtt_client: &paho_mqtt::Client,
) -> paho_mqtt::errors::Result<()> {
    while let Some((device, state, duration)) = rx.recv().await {
        log::debug!("changed: {:?}, {:?}, {:?}", device, state, duration);
        let message_str = format!("{}|{:?}", state, duration);
        let message = paho_mqtt::Message::new(MQTT_TOPIC, message_str.as_bytes(), paho_mqtt::QOS_2);
        mqtt_client.publish(message)?;
    }
    Ok(())
}

//fn state_topic_for_device(device: &crate::device::Device) {}
