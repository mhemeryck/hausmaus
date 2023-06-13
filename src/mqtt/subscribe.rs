/// subscribe module accepts incoming MQTT messages and forwards it back to the rest
use paho_mqtt;

/// handle incoming messages
pub async fn handle_incoming_messages(
    tx: std::sync::mpsc::Sender<crate::mqtt::MQTTEvent>,
    mqtt_client: &paho_mqtt::AsyncClient,
    command_topic_map: &std::collections::HashMap<String, u8>,
) -> paho_mqtt::errors::Result<()> {
    // Receive channel
    let mqtt_rx: paho_mqtt::Receiver<Option<paho_mqtt::Message>> = mqtt_client.start_consuming();

    // Subscribe to command topics for devices
    let topics: std::vec::Vec<String> = command_topic_map.keys().cloned().collect();
    mqtt_client
        .subscribe_many(&topics, &std::vec![paho_mqtt::QOS_2; topics.len()])
        .await?;

    // handle message
    for msg in mqtt_rx {
        if let Some(msg) = msg {
            log::info!(
                "Get msg {:?} for topic {:?} payload {:?}",
                msg,
                msg.topic(),
                msg.payload_str()
            );

            let toggle: bool;
            if msg.payload_str().as_ref() == "ON" {
                toggle = true;
            } else {
                toggle = false;
            }

            if let Some(&device_id) = command_topic_map.get(msg.topic()) {
                log::debug!("Received message for device #{}", device_id);
                tx.send((device_id, toggle)).unwrap();
            }
        }
    }

    // cleanup in case we'd quit
    if mqtt_client.is_connected() {
        mqtt_client.unsubscribe_many(&topics).await?;
        mqtt_client.disconnect(None).await?;
    }

    Ok(())
}
