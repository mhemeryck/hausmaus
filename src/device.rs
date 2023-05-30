#[derive(Eq, PartialEq, Debug)]
pub enum DeviceType {
    DigitalInput,
    DigitalOutput,
    RelayOutput,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Device {
    pub name: String,
    pub device_type: DeviceType,
    pub io_group: i32,
    pub number: i32,
}
