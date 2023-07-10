pub mod publish;
pub mod subscribe;

use crate::sysfs::DeviceId;

pub type MQTTEvent = (DeviceId, bool);
