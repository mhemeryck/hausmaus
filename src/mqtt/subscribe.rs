/// subscribe module accepts incoming MQTT messages and forwards it back to the rest
use rumqttc;
use std;

/// Subscribe to the topics as available in the topics from the command topic map
pub fn subscribe_topics(
    mqtt_client: &mut rumqttc::Client,
    command_topic_map: &std::collections::HashMap<String, u8>,
) {
    // COnvert to vector of (topic, QoS)
    let mut topic_qos: std::vec::Vec<rumqttc::SubscribeFilter> = std::vec::Vec::new();
    for topic in command_topic_map.keys() {
        topic_qos.push(rumqttc::SubscribeFilter {
            path: topic.clone(),
            qos: rumqttc::QoS::AtLeastOnce,
        });
    }

    mqtt_client.subscribe_many(topic_qos).unwrap();
}

/// handle incoming messages
pub fn handle_incoming_messages(
    tx: std::sync::mpsc::Sender<crate::mqtt::MQTTEvent>,
    mqtt_loop: &mut rumqttc::Connection,
    command_topic_map: &std::collections::HashMap<String, u8>,
) {
    // handle message
    for event in mqtt_loop.iter() {
        log::debug!("Received incoming event {:?}", event);
        if let Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg))) = event {
            log::debug!("Incoming event {:?} {:?}", msg.topic, msg.payload);

            if let Ok(payload) = std::str::from_utf8(&msg.payload.to_owned()) {
                let toggle: Option<bool> = match payload {
                    "ON" => Some(true),
                    "OFF" => Some(false),
                    _ => None,
                };

                if let (Some(&device_id), Some(payload)) =
                    (command_topic_map.get(&msg.topic), toggle)
                {
                    log::debug!("Received message for device #{}", device_id);
                    tx.send((device_id, payload)).unwrap();
                }
            }
        }
    }
}
