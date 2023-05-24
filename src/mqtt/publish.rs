/// publish module accepts all incoming events and publishes them to MQTT
use log;
use paho_mqtt;

use crate::sysfs::FileEvent;

/// handle_messages receives any file events and sends them out over MQTT
pub async fn publish_messages(
    mut rx: tokio::sync::mpsc::Receiver<FileEvent>,
    mqtt_client: &paho_mqtt::AsyncClient,
) -> paho_mqtt::errors::Result<()> {
    while let Some((device, state, duration)) = rx.recv().await {
        log::debug!("changed: {:?}, {:?}, {:?}", device, state, duration);
        let message_str = match state {
            true => "ON",
            false => "OFF",
        };
        let message = paho_mqtt::Message::new(
            state_topic_for_device(&device),
            message_str.as_bytes(),
            paho_mqtt::QOS_2,
        );
        mqtt_client.publish(message).await?;
    }
    Ok(())
}

/// generate a state topic to publish to for a given device
fn state_topic_for_device(device: &crate::device::Device) -> String {
    format!(
        "{name}/{device_type}/{io_group:1}_{number:02}/state",
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
    fn test_state_topic_for_device() {
        let device = crate::device::Device {
            name: String::from("foo"),
            device_type: crate::device::DeviceType::DigitalOutput,
            io_group: 1,
            number: 3,
        };

        assert_eq!(state_topic_for_device(&device), "foo/output/1_03/state");
    }
}
