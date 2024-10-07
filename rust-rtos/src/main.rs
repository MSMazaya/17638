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
use app_state::{AppResetMessage, AppState, MAX_QUEUE_SIZE};
use cortex_m_rt::entry;
use freertos_rust::*;

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let (leds, user_btn, accelerometer) = peripherals::setup();
    let state = Arc::new(Mutex::new(AppState::new()).unwrap());
    let state_queue = Arc::new(Queue::<AppResetMessage>::new(MAX_QUEUE_SIZE).unwrap());
    let task_resetter_semaphore = Arc::new(Semaphore::new_binary().unwrap());

    ecf::setup_interrupt(user_btn.interrupt());
    ecf::setup_interrupt_resource(user_btn, Arc::clone(&state_queue));

    Task::new()
        .name("accelerometer")
        .stack_size(128)
        .priority(TaskPriority(2))
        .start(tasks::accelerometer_task(
            Arc::clone(&state_queue),
            accelerometer,
        ))
        .unwrap();

    Task::new()
        .name("output")
        .stack_size(128)
        .priority(TaskPriority(1))
        .start(tasks::output_task(
            Arc::clone(&state_queue),
            Arc::clone(&state),
            leds,
        ))
        .unwrap();

    FreeRtosUtils::start_scheduler()
}
