use std::thread::JoinHandle;

use crossbeam::{channel::Receiver, select};

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

#[derive(Debug, Copy, Clone)]
pub enum CoverPosition {
    Open,
    Closed,
    Position(u8),
}

const COVER_SLEEP_TIME: f32 = 0.5;
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

    motor_up: u8,
    motor_down: u8,
}

impl Cover {
    pub fn new(motor_up: u8, motor_down: u8) -> Self {
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

    pub fn process_event(&mut self, event: CoverEvent) {
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
                log::info!("I was stopped -> time went by -- nothing left to do!")
                // TODO: implement timing logic
            }
            (CoverState::Opening, CoverEvent::TimerTick) => {
                log::info!("I was opening -> time went by!");

                match self.position {
                    CoverPosition::Open => {
                        log::info!("Already completely open, nothing left to do");
                    }
                    CoverPosition::Closed => {
                        self.position = CoverPosition::Position(1);
                    }
                    CoverPosition::Position(pos) => {
                        if pos < COVER_MAX_POSITION {
                            self.position = CoverPosition::Position(pos + 1)
                        } else {
                            self.position = CoverPosition::Open
                        }
                    }
                }
                log::info!("New position {:?}", self.position);
            }
            (CoverState::Closing, CoverEvent::TimerTick) => {
                log::info!("I was closing -> time went by!");

                match self.position {
                    CoverPosition::Open => {
                        self.position = CoverPosition::Position(u8::MAX - 2);
                    }
                    CoverPosition::Closed => {
                        self.position = CoverPosition::Closed;
                    }
                    CoverPosition::Position(pos) => {
                        self.position = CoverPosition::Position(pos - 1);
                    }
                }
                log::info!("New position {:?}", self.position);
            }
        }
    }

    fn start_timer(&self) -> std::thread::JoinHandle<()> {
        let mut self_clone = self.clone();
        std::thread::spawn(move || {
            let tick_duration = std::time::Duration::from_secs_f32(COVER_SLEEP_TIME);
            loop {
                let start_time = std::time::Instant::now();
                log::info!("TICK!");

                self_clone.process_event(CoverEvent::TimerTick);

                // Calculate remainder time after processing event
                let elapsed = start_time.elapsed();
                let remaining_time = tick_duration
                    .checked_sub(elapsed)
                    .unwrap_or_else(|| std::time::Duration::from_secs(0));

                log::info!("{:?}", remaining_time);

                std::thread::sleep(remaining_time);
            }
        })
    }

    fn start_events(&self, events: Receiver<CoverEvent>) -> JoinHandle<()> {
        let mut self_clone = self.clone();
        std::thread::spawn(move || {
            for event in events {
                self_clone.process_event(event);
            }
        })
    }

    pub fn start(&self, events: Receiver<CoverEvent>) {
        let timer_thread = self.start_timer();
        let event_thread = self.start_events(events);

        event_thread.join().unwrap();
        timer_thread.join().unwrap();
    }
}
