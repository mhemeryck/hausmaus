/// Represents an Normally Open push button
struct PushButton {
    // Link to underlying device
    device: *const Device,
    // Current state of the button; either pushed (true) or not (false)
    state: bool,
    // Instant to keep track of last update TODO: to be checked how of to push out updates
    t: std::time::Instant,
}

/// Represents a light control
struct Light {
    // Link to the underlying device
    device: *const Device,
    // LIght can be on or off
    state: bool,
}

/// Represents a dimmable light control
struct DimmableLight {
    device: *const Device,
    state: bool,
    // simple 256 level brightness control TODO: to be checked if this is enough (corresponds to
    // DALI, so probably OK enough
    brightness: u8,
}

enum CoverDirection {
    Up,
    Down,
    Stopped,
}

struct Cover {
    motor_up: *const Device,
    motor_down: *const Device,
    cover_direction: CoverDirection,
    position: u8,
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
