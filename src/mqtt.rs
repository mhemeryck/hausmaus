pub mod publish;
pub mod subscribe;

pub type MQTTEvent = (std::sync::Arc<crate::device::Device>, bool);

use regex;
use std;

const TOPIC_PATTERN: &str =
    r"(?P<name>\w+)/(?P<type>input|output|relay)/(?P<io_group>1|2|3)_(?P<number>\d+)/(state|set)";

/// Determine a device from a topic
pub fn device_from_topic(topic: &str) -> Option<crate::device::Device> {
    let re = regex::Regex::new(TOPIC_PATTERN).unwrap();
    if let Some(captures) = re.captures(topic) {
        if let (Some(name_str), Some(device_fmt), Some(io_group_str), Some(number_str)) = (
            captures.name("name"),
            captures.name("type"),
            captures.name("io_group"),
            captures.name("number"),
        ) {
            // Map against device type from capture
            let device_type = match device_fmt.as_str() {
                "input" => crate::device::DeviceType::DigitalInput,
                "output" => crate::device::DeviceType::DigitalOutput,
                "relay" => crate::device::DeviceType::RelayOutput,
                _ => return None,
            };
            // Parse and cast from capture
            if let (Ok(io_group), Ok(number)) = (
                io_group_str.as_str().parse::<i32>(),
                number_str.as_str().parse::<i32>(),
            ) {
                let name = name_str.as_str().to_string();
                return Some(crate::device::Device {
                    name,
                    device_type,
                    io_group,
                    number,
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_device_from_topic() {
        let topic = "foo/output/3_07/set";
        if let Some(device) = device_from_topic(&topic) {
            assert_eq!(device.name, "foo");
            assert_eq!(device.number, 7);
            assert_eq!(device.io_group, 3);
            assert_eq!(device.device_type, crate::device::DeviceType::DigitalOutput);
        } else {
            panic!("Could not find a device from path");
        }
    }
}
