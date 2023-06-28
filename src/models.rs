/// Represents an Normally Open push button
pub struct PushButton {
    // Link to underlying device
    pub device_id: u8,
    // Current state of the button; either pushed (true) or not (false)
    pub pushed: bool,
    // Instant to keep track of last update TODO: to be checked how of to push out updates
    //t: std::time::Instant,
}

/// Represents a light control
struct Light {
    // Link to the underlying device
    device_id: u8,
    // LIght can be on or off
    state: bool,
}

/// Represents a dimmable light control
struct DimmableLight {
    device_id: u8,
    state: bool,
    // simple 256 level brightness control TODO: to be checked if this is enough (corresponds to
    // DALI, so probably OK enough
    brightness: u8,
}

pub enum CoverDirection {
    Up,
    Down,
    Stopped,
}

pub struct Cover {
    pub motor_up: u8,
    pub motor_down: u8,
    pub direction: CoverDirection,
    pub position: u8,
}

impl Cover {
    fn up(&mut self) {
    }

    fn down(&mut self) {
    }

    fn go_to_position(&mut self, pos: u8) {
    }

    fn stop(&mut self) {
    }
}
