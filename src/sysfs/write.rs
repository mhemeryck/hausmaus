use log;
/// Write incoming messages back by updating the related file system entry
use std;
use std::io::Write;

pub async fn handle_file_command(
    prefix: String,
    rx: std::sync::mpsc::Receiver<crate::mqtt::MQTTEvent>,
) {
    for (device, toggle) in rx {
        let path = path_for_device(prefix.as_str(), &device);
        log::info!(
            "Received message {:?} {:?} new path {}",
            device,
            toggle,
            path
        );
        if let Ok(mut file) = std::fs::File::create(&path) {
            let content = match toggle {
                true => "1",
                false => "0",
            };
            file.write_all(content.as_bytes()).unwrap();
        }
    }
}

fn path_for_device(prefix: &str, device: &crate::device::Device) -> String {
    format!(
        "{prefix}/io_group{io_group}/{device_fmt}_{io_group}_{number:02}/{device_fmt}_value",
        prefix = prefix,
        io_group = device.io_group,
        number = device.number,
        device_fmt = match device.device_type {
            crate::device::DeviceType::DigitalInput => "di",
            crate::device::DeviceType::DigitalOutput => "do",
            crate::device::DeviceType::RelayOutput => "ro",
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_for_device() {
        let prefix = "/sys/devices/platform/unipi_plc";
        let device = crate::device::Device {
            module_name: String::from("foo"),
            number: 7,
            io_group: 2,
            device_type: crate::device::DeviceType::DigitalOutput,
        };
        let path = path_for_device(prefix, &device);
        assert_eq!(
            path,
            "/sys/devices/platform/unipi_plc/io_group2/do_2_07/do_value"
        );
    }
}
