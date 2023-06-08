use regex;

#[derive(Eq, PartialEq, Debug)]
pub enum DeviceType {
    DigitalInput,
    DigitalOutput,
    RelayOutput,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Device {
    // Name of the module the device is linked to
    pub module_name: String,
    pub device_type: DeviceType,
    pub io_group: i32,
    pub number: i32,
    pub path: String,
}

impl Device {
    const FILENAME_PATTERN: &str = r"/io_group(1|2|3)/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/(di|do|ro)_value$";

    // Construct a device from a path as a string
    pub fn from_path(path_str: &str, module_name: &str) -> Result<Self, crate::errors::MausError> {
        if let Ok(re) = regex::Regex::new(Self::FILENAME_PATTERN) {
            if let Some(captures) = re.captures(path_str) {
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
                        io_group_str.as_str().parse::<i32>(),
                        number_str.as_str().parse::<i32>(),
                    ) {
                        let module_name = module_name.to_string();
                        let path = path_str.to_string();
                        return Ok(Self {
                            module_name,
                            device_type,
                            io_group,
                            number,
                            path,
                        });
                    }
                }
            }
        }

        // In all other cases, nothing was found
        Err(crate::errors::MausError::new(
            "Could not create a device from path: regular expression does not match".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_from_path() {
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/di_value";
        let module_name = "foo";
        if let Ok(device) = Device::from_path(&path, &module_name) {
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
    fn test_device_from_path_not_found() {
        let path = "sys/devices/platform/unipi_plc/io_group2/di_2_07/foo";
        let module_name = "foo";
        if let Ok(_) = Device::from_path(&path, &module_name) {
            panic!("It shouldn't find a device in this case!");
        }
    }
}
