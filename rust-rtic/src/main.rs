#![no_main]
#![no_std]

// Halt on panic
use panic_semihosting as _;
use rtic_monotonics::systick::prelude::*;
mod app_state;
mod peripherals;

systick_monotonic!(Mono, 36_000);

#[rtic::app(device = stm32f3xx_hal::pac, peripherals = true, dispatchers=[EXTI1, EXTI4, EXTI3])]
mod app {
    use core::borrow::BorrowMut;

    use app_state::{AppResetMessage, AppState, ACTIVE_COUNTER_INITIAL_VALUE};
    use cortex_m_semihosting::hprintln;
    use peripherals::{Accelerometer, Leds};
    use rtic_sync::{channel::*, make_channel};
    use stm32f3xx_hal::prelude::_embedded_hal_digital_OutputPin;

    use super::*;

    #[shared]
    struct Shared {
        app_state: AppState,
    }

    // Local resources go here
    #[local]
    struct Local {
        prev_x: i16,
        prev_y: i16,
        prev_z: i16,
        accelerometer: Accelerometer,
        leds: Leds,
    }

    const CAPACITY: usize = 5;
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let (mut leds, user_btn, accelerometer) = peripherals::setup(cx);
        let app_state = AppState::new();
        let (s, r) = make_channel!(AppResetMessage, CAPACITY);

        accelerometer_task::spawn(s).unwrap();
        output_task::spawn().unwrap();
        transition_task::spawn(r).unwrap();

        (
            Shared { app_state },
            Local {
                // Initialization of local resources go here
                prev_x: 0,
                prev_y: 0,
                prev_z: 0,
                accelerometer,
                leds,
            },
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(binds = EXTI0, shared = [app_state])]
    fn exti0(mut cx: exti0::Context) {
        cx.shared.app_state.lock(|s| match s {
            AppState::Alarm => s.reset(),
            _ => {}
        });
    }

    #[task(priority=2,local=[prev_x, prev_y, prev_z, accelerometer])]
    async fn accelerometer_task(
        c: accelerometer_task::Context,
        mut sender: Sender<'static, AppResetMessage, CAPACITY>,
    ) {
        let prev_x = c.local.prev_x;
        let prev_y = c.local.prev_y;
        let prev_z = c.local.prev_z;
        let accelerometer = c.local.accelerometer;
        if let Ok(axis) = accelerometer.accel() {
            *prev_x = axis.x;
            *prev_y = axis.y;
            *prev_z = axis.z;
        }
        loop {
            if let Ok(axis) = accelerometer.accel() {
                let difference =
                    (axis.x - *prev_x).abs() + (axis.y - *prev_y).abs() + (axis.z - *prev_z).abs();
                if difference > 1000 {
                    let _ = sender.send(AppResetMessage::FromAccelerometer).await;
                }
                *prev_x = axis.x;
                *prev_y = axis.y;
                *prev_z = axis.z;
            }
            Mono::delay(1000.millis()).await;
        }
    }

    #[task(priority=2,shared=[app_state])]
    async fn transition_task(
        c: transition_task::Context,
        mut receiver: Receiver<'static, AppResetMessage, CAPACITY>,
    ) {
        let mut shared_app_state = c.shared.app_state;
        while let Ok(transition) = receiver.recv().await {
            shared_app_state.lock(|s| {
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
                };
            })
        }
    }

    #[task(priority=1,local=[leds], shared=[app_state])]
    async fn output_task(c: output_task::Context) {
        let mut shared_app_state = c.shared.app_state;
        let leds = c.local.leds;
        loop {
            let s = shared_app_state.lock(|s| s.clone());
            match s {
                AppState::Active(counter) => {
                    if counter != 0 {
                        let _ = leds.north.set_high();
                        Mono::delay(25.millis()).await;
                        let _ = leds.north.set_low();
                        Mono::delay(75.millis()).await;
                        let _ = shared_app_state.lock(|s| s.decrement_counter());
                    } else {
                        let _ = shared_app_state.lock(|s| s.transition());
                    }
                }
                AppState::PreAlarm(counter) => {
                    let first_direction = leds.current_direction;
                    leds.set_high_current_direction();
                    Mono::delay(10.millis()).await;
                    leds.set_low_current_direction();
                    leds.to_next_direction();
                    while leds.current_direction != first_direction {
                        leds.set_high_current_direction();
                        Mono::delay(10.millis()).await;
                        leds.set_low_current_direction();
                        leds.to_next_direction();
                    }
                    Mono::delay(200.millis()).await;
                    let _ = shared_app_state.lock(|s| {
                        s.decrement_counter();
                        s.decrement_counter();
                    });
                    if counter == 0 {
                        let _ = shared_app_state.lock(|s| {
                            s.transition();
                        });
                    }
                }
                AppState::Alarm => {
                    leds.set_high_all_direction();
                    Mono::delay(100.millis()).await;
                    leds.set_low_all_direction();
                    Mono::delay(100.millis()).await;
                }
            };
        }
    }
}
