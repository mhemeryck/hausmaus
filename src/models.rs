use crate::sysfs::DeviceId;
use crossbeam::{
    channel::{tick, Receiver},
    select,
};
use std::{
    thread::{spawn, JoinHandle},
    time::Duration,
};

/// Represents an Normally Open push button
pub struct PushButton {
    // Link to underlying device
    pub device_id: DeviceId,
    // Current state of the button; either pushed (true) or not (false)
    pub pushed: bool,
    // Instant to keep track of last update TODO: to be checked how of to push out updates
    //t: std::time::Instant,
}

/// Represents a light control
struct Light {
    // Link to the underlying device
    device_id: DeviceId,
    // LIght can be on or off
    state: bool,
}

/// Represents a dimmable light control
struct DimmableLight {
    device_id: DeviceId,
    state: bool,
    // simple 256 level brightness control TODO: to be checked if this is enough (corresponds to
    // DALI, so probably OK enough
    brightness: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoverPosition {
    Open,
    Closed,
    Position(u8),
}

const COVER_SLEEP_TIME_MILLIS: u64 = 500;
const COVER_MAX_POSITION: u8 = u8::MAX - 1;

#[derive(Debug, Copy, Clone)]
pub enum CoverEvent {
    PushButtonOpen,
    PushButtonClose,
    TimerTick,
}

#[derive(Debug, Copy, Clone)]
pub enum CoverState {
    Opening,
    Closing,
    Stopped,
}

#[derive(Debug, Copy, Clone)]
pub struct Cover {
    pub state: CoverState,
    pub position: CoverPosition,

    motor_up: DeviceId,
    motor_down: DeviceId,
}

impl Cover {
    pub fn new(motor_up: DeviceId, motor_down: DeviceId) -> Self {
        Self {
            motor_up,
            motor_down,
            state: CoverState::Stopped,
            position: CoverPosition::Open,
        }
    }

    fn open(&mut self) {
        log::info!("Opening!");
        // TODO
        // - stop motor down
        // - start motor up
        // - publish event "opening"
    }

    fn close(&mut self) {
        log::info!("Closing!");
        // TODO
        // - stop motor up
        // - start motor down
        // - publish event "closing"
    }

    fn stop(&mut self) {
        log::info!("Stopping!");
        // TODO
        // - stop motor up
        // - stop motor down
        // - publish event "closed | open | opening | closing" -- depending on current position
    }

    fn process_event(&mut self, event: CoverEvent) {
        match (self.state, event) {
            (CoverState::Stopped, CoverEvent::PushButtonOpen) => {
                log::info!("I was stopped -> opening now!");
                self.state = CoverState::Opening;
                self.open();
            }
            (CoverState::Stopped, CoverEvent::PushButtonClose) => {
                log::info!("I was stopped -> closing now!");
                self.state = CoverState::Closing;
                self.close()
            }
            (CoverState::Opening, CoverEvent::PushButtonOpen) => {
                log::info!("I was opening -> stopping now!");
                self.state = CoverState::Stopped;
                self.stop()
            }
            (CoverState::Opening, CoverEvent::PushButtonClose) => {
                log::info!("I was opening -> closing now!");
                self.state = CoverState::Closing;
                self.close()
            }
            (CoverState::Closing, CoverEvent::PushButtonClose) => {
                log::info!("I was closing -> stopping now!");
                self.state = CoverState::Stopped;
                self.stop()
            }
            (CoverState::Closing, CoverEvent::PushButtonOpen) => {
                log::info!("I was closing -> opening now!");
                self.state = CoverState::Opening;
                self.open()
            }
            (CoverState::Stopped, CoverEvent::TimerTick) => {
                log::debug!("I was stopped -> time went by -- nothing left to do!");
            }
            (CoverState::Opening, CoverEvent::TimerTick) => {
                log::info!("I was opening -> time went by!");

                self.position = match self.position {
                    CoverPosition::Closed => CoverPosition::Position(1),
                    CoverPosition::Position(pos) if pos < COVER_MAX_POSITION => {
                        CoverPosition::Position(pos + 1)
                    }
                    CoverPosition::Position(_) | CoverPosition::Open => CoverPosition::Open,
                };

                log::info!("New position {:?}", self.position);

                if self.position == CoverPosition::Open {
                    log::info!("I can stop opening now ...");
                    self.state = CoverState::Stopped;
                    self.stop()
                }
            }
            (CoverState::Closing, CoverEvent::TimerTick) => {
                log::info!("I was closing -> time went by!");

                self.position = match self.position {
                    CoverPosition::Open => CoverPosition::Position(COVER_MAX_POSITION - 1),
                    CoverPosition::Position(pos) if pos > 0 => CoverPosition::Position(pos - 1),
                    CoverPosition::Position(_) | CoverPosition::Closed => CoverPosition::Closed,
                };

                log::info!("New position {:?}", self.position);

                if self.position == CoverPosition::Closed {
                    log::info!("I can stop closing now ...");
                    self.state = CoverState::Stopped;
                    self.stop()
                }
            }
        }
    }

    /// Monitor handles events from an incoming channel
    pub fn monitor(mut self, event_rx: Receiver<CoverEvent>) -> JoinHandle<()> {
        let ticker = tick(Duration::from_millis(COVER_SLEEP_TIME_MILLIS));

        spawn(move || loop {
            select! {
                recv(event_rx) -> msg => {
                    if let Ok(CoverEvent::PushButtonOpen) | Ok(CoverEvent::PushButtonClose) = msg {
                        log::debug!("Got an event {:?}", msg);
                        self.process_event(msg.unwrap());
                    }
                },
                recv(ticker) -> msg => {
                    log::debug!("Got a timer {:?}", msg);
                    self.process_event(CoverEvent::TimerTick);
                },
            }
        })
    }
}
