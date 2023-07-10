/// sysfs contains the interface the file system based view on IO
pub mod read;
pub mod write;

pub type DeviceId = u8;
pub type FileEvent = (DeviceId, bool, std::time::Duration);
