use alloc::sync::Arc;
use freertos_rust::{CurrentTask, Duration, Mutex, Queue, Semaphore, Task};
use stm32f3xx_hal::prelude::_embedded_hal_digital_OutputPin;

use crate::{
    app_state::{
        AppResetMessage, AppState, ACTIVE_COUNTER_INITIAL_VALUE, PRE_ALARM_COUNTER_INITIAL_VALUE,
    },
    peripherals::{Accelerometer, Leds},
};

const DIFFERENCE_TOLERANCE: i16 = 1000;

pub fn accelerometer_task(
    state_queue: Arc<Queue<AppResetMessage>>,
    mut accelerometer: Accelerometer,
) -> impl FnOnce(Task) + Send + 'static {
    let mut prev_x = 0;
    let mut prev_y = 0;
    let mut prev_z = 0;
    if let Ok(axis) = accelerometer.accel() {
        prev_x = axis.x;
        prev_y = axis.y;
        prev_z = axis.z;
    }
    move |_| loop {
        if let Ok(axis) = accelerometer.accel() {
            let difference =
                (axis.x - prev_x).abs() + (axis.y - prev_y).abs() + (axis.z - prev_z).abs();
            if difference > DIFFERENCE_TOLERANCE {
                let _ = state_queue.send(AppResetMessage::FromAccelerometer, Duration::infinite());
            }
            prev_x = axis.x;
            prev_y = axis.y;
            prev_z = axis.z;
        }
        CurrentTask::delay(Duration::ms(1000));
    }
}

pub fn output_task(
    state_queue: Arc<Queue<AppResetMessage>>,
    s_arc: Arc<Mutex<AppState>>,
    mut leds: Leds,
) -> impl FnOnce(Task) + Send + 'static {
    move |_| loop {
        if let Ok(mut s) = s_arc.lock(Duration::infinite()) {
            if let Ok(transition) = state_queue.receive(Duration::zero()) {
                match transition {
                    AppResetMessage::FromButton => match *s {
                        AppState::Alarm => *s = AppState::Active(ACTIVE_COUNTER_INITIAL_VALUE),
                        _ => {}
                    },
                    AppResetMessage::FromAccelerometer => match *s {
                        AppState::PreAlarm(_) => {
                            *s = AppState::Active(ACTIVE_COUNTER_INITIAL_VALUE)
                        }
                        _ => {}
                    },
                }
            }
            match *s {
                AppState::Active(counter) => {
                    if counter != 0 {
                        let _ = leds.north.set_high();
                        CurrentTask::delay(Duration::ms(250));
                        let _ = leds.north.set_low();
                        CurrentTask::delay(Duration::ms(750));
                        s.decrement_counter();
                    } else {
                        s.transition();
                    }
                }
                AppState::PreAlarm(counter) => {
                    let first_direction = leds.current_direction;
                    leds.turn_on_current_for(Duration::ms(100));
                    leds.to_next_direction();
                    while leds.current_direction != first_direction {
                        leds.turn_on_current_for(Duration::ms(100));
                        leds.to_next_direction();
                    }
                    CurrentTask::delay(Duration::ms(2000));
                    s.decrement_counter();
                    s.decrement_counter();
                    if counter == 0 {
                        s.transition();
                    }
                }
                AppState::Alarm => {
                    leds.set_high_all_direction();
                    CurrentTask::delay(Duration::ms(1000));
                    leds.set_low_all_direction();
                    CurrentTask::delay(Duration::ms(1000));
                }
            };
        }
    }
}
