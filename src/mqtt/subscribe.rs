/// subscribe module accepts incoming MQTT messages and forwards it back to the rest
use paho_mqtt;

/// handle incoming messages
pub async fn handle_incoming_messages(
    mqtt_client: &paho_mqtt::AsyncClient,
    devices: &std::vec::Vec<crate::device::Device>,
    tx: std::sync::mpsc::Sender<crate::mqtt::MQTTEvent>,
) -> paho_mqtt::errors::Result<()> {
    // Receive channel
    let mqtt_rx: paho_mqtt::Receiver<Option<paho_mqtt::Message>> = mqtt_client.start_consuming();

    // Subscribe to command topics for devices
    let mut topics: std::vec::Vec<String> = std::vec::Vec::with_capacity(devices.len());
    command_topics_for_devices(&devices, &mut topics).await;
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
            let device = crate::mqtt::device_from_topic(msg.topic()).unwrap();
            let device = std::sync::Arc::new(device);

            let toggle: bool;
            if msg.payload_str().as_ref() == "ON" {
                toggle = true;
            } else {
                toggle = false;
            }

            tx.send((device, toggle)).unwrap();
        }
    }

    // cleanup in case we'd quit
    if mqtt_client.is_connected() {
        mqtt_client.unsubscribe_many(&topics).await?;
        mqtt_client.disconnect(None).await?;
    }

    Ok(())
}

/// subscribe to a series of topics
async fn command_topics_for_devices(
    devices: &std::vec::Vec<crate::device::Device>,
    topics: &mut std::vec::Vec<String>,
) {
    for device in devices {
        topics.push(command_topic_for_device(&device));
    }
}

fn command_topic_for_device(device: &crate::device::Device) -> String {
    format!(
        "{name}/{device_type}/{io_group:1}_{number:02}/set",
        name = device.module_name,
        device_type = match device.device_type {
            crate::device::DeviceType::DigitalInput => "input",
            crate::device::DeviceType::DigitalOutput => "output",
            crate::device::DeviceType::RelayOutput => "relay",
        },
        io_group = device.io_group,
        number = device.number
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_topic_for_device() {
        let device = crate::device::Device {
            module_name: String::from("foo"),
            device_type: crate::device::DeviceType::DigitalOutput,
            io_group: 1,
            number: 3,
        };

        assert_eq!(command_topic_for_device(&device), "foo/output/1_03/set");
    }
}
