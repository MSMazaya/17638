pub const MAX_QUEUE_SIZE: usize = 5;
pub const PRE_ALARM_COUNTER_INITIAL_VALUE: usize = 16;
pub const ACTIVE_COUNTER_INITIAL_VALUE: usize = 5;

#[derive(Clone, Copy)]
pub enum AppState {
    Active(usize),
    PreAlarm(usize),
    Alarm,
}

impl AppState {
    pub fn new() -> AppState {
        AppState::Active(5)
    }
    pub fn reset(&mut self) {
        *self = AppState::new();
    }
    pub fn transition(&mut self) {
        *self = match self {
            AppState::Active(_) => AppState::PreAlarm(PRE_ALARM_COUNTER_INITIAL_VALUE),
            AppState::PreAlarm(_) => AppState::Alarm,
            AppState::Alarm => AppState::Active(ACTIVE_COUNTER_INITIAL_VALUE),
        };
    }
    pub fn decrement_counter(&mut self) {
        *self = match self {
            AppState::Active(counter) => AppState::Active(*counter - 1),
            AppState::PreAlarm(counter) => AppState::PreAlarm(*counter - 1),
            AppState::Alarm => AppState::Alarm,
        };
    }
}

#[derive(Clone, Copy)]
pub enum AppResetMessage {
    FromButton,
    FromAccelerometer,
}
