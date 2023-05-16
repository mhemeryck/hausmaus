#[derive(Eq, PartialEq, Debug)]
pub enum DeviceType {
    DigitalInput,
    DigitalOutput,
    RelayOutput,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Device {
    pub device_type: DeviceType,
    pub io_group: i32,
    pub number: i32,
}

//impl Device {
//    const FILENAME_PATTERN: &str =
//        r"/(?P<device_fmt>di|do|ro)_(?P<io_group>1|2|3)_(?P<number>\d{2})/(di|do|ro)_value";
//
//    /// file path from prefix built with device name parts
//    fn path(&self, prefix: &str) -> String {
//        format!(
//            "{prefix}/io_group_{io_group}/{device_fmt}_{io_group}_{number:02}/{device_fmt}_value",
//            prefix = prefix,
//            io_group = self.io_group,
//            number = self.number,
//            device_fmt = match self.device_type {
//                DeviceType::DigitalInput => "di",
//                DeviceType::DigitalOutput => "do",
//                DeviceType::RelayOutput => "ro",
//            },
//        )
//    }
//    /*
//
//    /// Construct MQTT state topic from device name parts
//    fn state_topic(&self, device_name: &str) -> &str {}
//
//    /// Construct MQTT command topic from device name parts
//    fn command_topic(&self, device_name: &str) -> &str {}
//    */
//
//    ///// Create a device just from a path string
//    //fn from_path(path_str: &str) -> Option<Self> {
//    //    let re = regex::Regex::new(Self::FILENAME_PATTERN).unwrap();
//    //    if let Some(captures) = re.captures(path_str) {
//    //        if let (Some(device_fmt), Some(io_group_str), Some(number_str)) = (
//    //            captures.name("device_fmt"),
//    //            captures.name("io_group"),
//    //            captures.name("number"),
//    //        ) {
//    //            // Map against device type from capture
//    //            let device_type = match device_fmt.as_str() {
//    //                "di" => DeviceType::DigitalInput,
//    //                "do" => DeviceType::DigitalOutput,
//    //                "ro" => DeviceType::RelayOutput,
//    //                _ => return None,
//    //            };
//
//    //            // Parse and cast from capture
//    //            if let (Ok(io_group), Ok(number)) = (
//    //                io_group_str.as_str().parse::<i32>(),
//    //                number_str.as_str().parse::<i32>(),
//    //            ) {
//    //                return Some(Self {
//    //                    device_type,
//    //                    io_group,
//    //                    number,
//    //                });
//    //            }
//    //        }
//    //    }
//    //    // In all other cases, nothing was found
//    //    None
//    //}
//}
