use rumqttc;
/// subscribe module accepts incoming MQTT messages and forwards it back to the rest
use std;

/// Subscribe to the topics as available in the topics from the command topic map
pub async fn subscribe_topics(
    mqtt_client: &rumqttc::AsyncClient,
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

    mqtt_client.subscribe_many(topic_qos).await.unwrap();
}

/// handle incoming messages
pub async fn handle_incoming_messages(
    _tx: std::sync::mpsc::Sender<crate::mqtt::MQTTEvent>,
    mqtt_loop: &mut rumqttc::EventLoop,
) {
    // handle message
    loop {
        let event = mqtt_loop.poll().await.unwrap();
        log::debug!("Received incoming event {:?}", event);
        //if let Some(msg) = msg {
        //    log::info!(
        //        "Get msg {:?} for topic {:?} payload {:?}",
        //        msg,
        //        msg.topic(),
        //        msg.payload_str()
        //    );

        //    let toggle: bool;
        //    if msg.payload_str().as_ref() == "ON" {
        //        toggle = true;
        //    } else {
        //        toggle = false;
        //    }

        //    if let Some(&device_id) = command_topic_map.get(msg.topic()) {
        //        log::debug!("Received message for device #{}", device_id);
        //        tx.send((device_id, toggle)).unwrap();
        //    }
        //}
    }
}
