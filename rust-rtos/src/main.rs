#![no_main]
#![no_std]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use panic_halt as _;

extern crate alloc;
mod app_state;
mod ecf;
mod peripherals;
mod tasks;
use alloc::sync::Arc;
use app_state::AppState;
use cortex_m_rt::entry;
use freertos_rust::*;
use tasks::{accelerometer_task, output_task, state_resetter_task};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let (leds, user_btn, accelerometer) = peripherals::setup();
    let state = Arc::new(Mutex::new(AppState::new()).unwrap());
    let task_resetter_semaphore = Arc::new(Semaphore::new_binary().unwrap());

    ecf::setup_interrupt(user_btn.interrupt());
    ecf::setup_interrupt_resource(user_btn, Arc::clone(&task_resetter_semaphore));

    Task::new()
        .name("accelerometer")
        .stack_size(128)
        .priority(TaskPriority(2))
        .start(accelerometer_task(Arc::clone(&state), accelerometer))
        .unwrap();

    Task::new()
        .name("state_resetter")
        .stack_size(128)
        .priority(TaskPriority(2))
        .start(state_resetter_task(
            Arc::clone(&state),
            Arc::clone(&task_resetter_semaphore),
        ))
        .unwrap();

    Task::new()
        .name("output")
        .stack_size(128)
        .priority(TaskPriority(1))
        .start(output_task(Arc::clone(&state), leds))
        .unwrap();

    FreeRtosUtils::start_scheduler()
}
