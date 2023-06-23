/// sysfs contains the interface the file system based view on IO
pub mod read;
pub mod write;

pub type FileEvent = (u8, bool, std::time::Duration);
