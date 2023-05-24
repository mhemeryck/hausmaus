/// subscribe module accepts incoming MQTT messages and forwards it back to the rest
use log;
use paho_mqtt;

/// subscribe to a series of topics
pub async fn subscribe_topics(
    mqtt_client: &paho_mqtt::AsyncClient,
    devices: &std::vec::Vec<crate::device::Device>,
) -> paho_mqtt::errors::Result<()> {
    for device in devices {
        let topic = command_topic_for_device(&device);
        log::debug!("Subscribing for device {:?} to topic {:?}", device, topic);
        mqtt_client.subscribe(topic.as_str(), paho_mqtt::QOS_2).await?;
    }
    Ok(())
}

fn command_topic_for_device(device: &crate::device::Device) -> String {
    format!(
        "{name}/{device_type}/{io_group:1}_{number:02}/set",
        name = device.name,
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
            name: String::from("foo"),
            device_type: crate::device::DeviceType::DigitalOutput,
            io_group: 1,
            number: 3,
        };

        assert_eq!(command_topic_for_device(&device), "foo/output/1_03/set");
    }
}
