use regex;
use std;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum DeviceType {
    DigitalInput,
    DigitalOutput,
    RelayOutput,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Device {
    // Simple number identifying the device
    pub id: u8,
    // Name of the module the device is linked to
    pub module_name: String,
    pub device_type: DeviceType,
    pub io_group: i8,
    pub number: i8,
    pub path: String,
}
const FILENAME_PATTERN: &str = r"/io_group(1|2|3)/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/(di|do|ro)_value$";
// Construct a device from a regex captures
fn device_from_captures(
    captures: &regex::Captures,
    id: u8,
    path_str: &str,
    module_name: &str,
) -> Result<crate::device::Device, crate::errors::MausError> {
    if let (Some(device_fmt), Some(io_group_str), Some(number_str)) = (
        captures.name("device_fmt"),
        captures.name("io_group"),
        captures.name("number"),
    ) {
        // Map against device type from capture
        let device_type = match device_fmt.as_str() {
            "di" => DeviceType::DigitalInput,
            "do" => DeviceType::DigitalOutput,
            "ro" => DeviceType::RelayOutput,
            _ => {
                return Err(crate::errors::MausError::new(
                    "Could not determine device type from path".to_string(),
                ))
            }
        };

        // Parse and cast from capture
        if let (Ok(io_group), Ok(number)) = (
            io_group_str.as_str().parse::<i8>(),
            number_str.as_str().parse::<i8>(),
        ) {
            let module_name = module_name.to_string();
            let path = path_str.to_string();
            return Ok(crate::device::Device {
                id,
                module_name,
                device_type,
                io_group,
                number,
                path,
            });
        }
    }

    // In all other cases, nothing was found
    Err(crate::errors::MausError::new(
        "Could not create a device from path: regular expression does not match".to_string(),
    ))
}

// Recursively crawl a directory, match against a regex and if it matches, add it a vector of
// devices
fn crawl(
    dir: &std::path::Path,
    module_name: &str,
    re: &regex::Regex,
    devices: &mut std::vec::Vec<crate::device::Device>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        log::debug!("Checking dir {:?}", dir);
        for entry in std::fs::read_dir(dir)? {
            let entry: std::fs::DirEntry = entry?;
            let path = entry.path();
            log::debug!("Checking path {:?}", path);
            // Skip symlinks to avoid infinite loops
            if path.is_symlink() {
                continue;
            }

            // dirs need to be crawled further
            if path.is_dir() {
                crawl(&path, module_name, re, devices)?;
            } else {
                if let Some(path_str) = path.to_str() {
                    // The id we use here is just the current length of the list
                    let id: u8 = devices.len().try_into().unwrap();
                    if let Some(captures) = re.captures(path_str) {
                        if let Ok(device) =
                            device_from_captures(&captures, id, path_str, module_name)
                        {
                            devices.push(device);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Populate a vector of devices from a given directory
pub fn devices_from_path(
    dir: &str,
    module_name: &str,
    devices: &mut std::vec::Vec<crate::device::Device>,
) -> Result<(), crate::errors::MausError> {
    if let Ok(re) = regex::Regex::new(FILENAME_PATTERN) {
        crawl(&std::path::Path::new(&dir), module_name, &re, devices).unwrap();
        return Ok(());
    }
    Err(crate::errors::MausError::new(
        "Could not build list of devices".to_string(),
    ))
}

// Map a device to an MQTT state topic
fn state_topic_for_device(device: &crate::device::Device) -> String {
    format!(
        "{name}/{device_type}/{io_group:1}_{number:02}/state",
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

/// Set up mapping device -> state topic
pub fn device_state_topics(
    devices: &std::vec::Vec<Device>,
    cache: &mut std::collections::HashMap<u8, String>,
) {
    for device in devices {
        cache.insert(device.id, state_topic_for_device(&device));
    }
}

/// Mapping command topic -> device ID
pub fn device_command_topics(
    devices: &std::vec::Vec<Device>,
    cache: &mut std::collections::HashMap<String, u8>,
) {
    for device in devices {
        cache.insert(command_topic_for_device(&device), device.id);
    }
}

/// Mapping device ID -> path
pub fn device_paths(
    devices: &std::vec::Vec<Device>,
    cache: &mut std::collections::HashMap<u8, String>,
) {
    for device in devices {
        cache.insert(device.id, device.path.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_state_topic_for_device() {
        let device = crate::device::Device {
            id: 0,
            path: "/foo/bar".to_string(),
            module_name: String::from("foo"),
            device_type: crate::device::DeviceType::DigitalOutput,
            io_group: 1,
            number: 3,
        };

        assert_eq!(state_topic_for_device(&device), "foo/output/1_03/state");
    }

    #[test]
    fn test_command_topic_for_device() {
        let device = crate::device::Device {
            id: 0,
            path: "/foo/bar".to_string(),
            module_name: String::from("foo"),
            device_type: crate::device::DeviceType::DigitalOutput,
            io_group: 1,
            number: 3,
        };

        assert_eq!(command_topic_for_device(&device), "foo/output/1_03/set");
    }

    #[test]
    fn test_device_from_captures() {
        let id = 1;
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/di_value";
        let module_name = "foo";
        let re = regex::Regex::new(crate::device::FILENAME_PATTERN).unwrap();
        let captures = re.captures(path).unwrap();
        if let Ok(device) = device_from_captures(&captures, id, &path, &module_name) {
            assert_eq!(device.module_name, "foo");
            assert_eq!(device.number, 7);
            assert_eq!(device.io_group, 2);
            assert_eq!(device.device_type, DeviceType::DigitalInput);
            assert_eq!(device.path, path.to_string());
        } else {
            panic!("Could not find a device from path");
        }
    }

    #[test]
    fn test_device_from_captures_not_found() {
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/foo";
        let re = regex::Regex::new(crate::device::FILENAME_PATTERN).unwrap();
        if let Some(_) = re.captures(path) {
            panic!("Found a device, should not be the case");
        }
    }

    #[test]
    fn test_crawl_simple_file_matches() {
        // Top-level tmp dir
        let tmp_dir =
            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
        // Paths between
        let folder_structure = "sys/devices/platform/unipi_plc/io_group2/di_2_07/";
        let full_path = tmp_dir.path().join(folder_structure);
        // Create them
        std::fs::create_dir_all(&full_path).expect("Could not create folder");
        // The final path
        let path = full_path.join("di_value");

        // Write some stuff
        let mut tmp_file = std::fs::File::create(&path).expect("Could not open a new temp file");
        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");

        let module_name = "foo";
        let re = regex::Regex::new(crate::device::FILENAME_PATTERN).unwrap();

        let mut devices = std::vec::Vec::new();

        crawl(tmp_dir.path(), &module_name, &re, &mut devices).expect("Expect crawl to work");

        assert_eq!(devices.len(), 1);

        let path_string = String::from(path.to_str().unwrap());

        let device = &devices[0];
        assert_eq!(device.module_name, "foo");
        assert_eq!(device.number, 7);
        assert_eq!(device.io_group, 2);
        assert_eq!(device.device_type, DeviceType::DigitalInput);
        assert_eq!(device.path, path_string);

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_crawl_simple_file_no_matches() {
        // Top-level tmp dir
        let tmp_dir =
            tempdir::TempDir::new("myfolder").expect("Could not create a temporary folder");
        // Paths between
        let folder_structure = "sys/devices/platform/unipi_plc/io_group2/di_2_07/";
        let full_path = tmp_dir.path().join(folder_structure);
        // Create them
        std::fs::create_dir_all(&full_path).expect("Could not create folder");
        // The final path
        let path = full_path.join("foohaha");

        // Write some stuff
        let mut tmp_file = std::fs::File::create(&path).expect("Could not open a new temp file");
        writeln!(tmp_file, "Hello").expect("Could not write contents to temp file");

        let module_name = "foo";
        let re = regex::Regex::new(crate::device::FILENAME_PATTERN).unwrap();

        let mut devices = std::vec::Vec::new();

        crawl(tmp_dir.path(), &module_name, &re, &mut devices).expect("Expect crawl to work");

        assert_eq!(devices.len(), 0);

        tmp_dir.close().unwrap();
    }
}
