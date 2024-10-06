pub enum AppState {
    Active(u32),
    PreAlarm(u32),
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
            AppState::Active(_) => AppState::PreAlarm(16),
            AppState::PreAlarm(_) => AppState::Alarm,
            AppState::Alarm => AppState::Active(5),
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
