#![no_main]
#![no_std]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

// Halt on panic
use panic_halt as _;

use core::alloc::Layout;
use cortex_m::asm;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use freertos_rust::*;
use stm32f3xx_hal::{gpio::*, pac, prelude::*, time::duration};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;
// static mut S1: Option<FreeRtosSemaphoreHandle> = None;
// static mut S2: Option<FreeRtosSemaphoreHandle> = None;
static mut SEMAPHORE_HANDLE: Option<Semaphore> = None;

type LedPin<const T: u8> = Pin<Gpioe, U<T>, Output<PushPull>>;

// unsafe fn create_binary_semaphore() -> Result<FreeRtosSemaphoreHandle, FreeRtosError> {
//     let s = freertos_rs_create_binary_semaphore();
//     if s == 0 as *const _ {
//         return Err(FreeRtosError::OutOfMemory);
//     }
//     Ok(s)
// }

fn generate_led_blinky<const T: u8>(mut led: LedPin<T>) -> impl FnMut(Task) -> () {
    move |_| loop {
        unsafe {
            let s1 = SEMAPHORE_HANDLE.as_mut().unwrap();
            let _ = led.set_low();
            let _ = s1.take(Duration::infinite());
            let _ = led.set_high();
            freertos_rust::CurrentTask::delay(Duration::ms(1000));
            s1.give();
            freertos_rust::CurrentTask::delay(Duration::ms(20));
        }
    }
}

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let mut rcc = p.RCC.constrain();
    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
    let mut led_9 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led_8 = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let mut led_10 = gpioe
        .pe10
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    unsafe {
        SEMAPHORE_HANDLE = Some(Semaphore::new_counting(1, 1).unwrap());
    }

    Task::new()
        .name("blinky")
        .stack_size(128)
        .priority(TaskPriority(3))
        .start(generate_led_blinky(led_9))
        .unwrap();
    Task::new()
        .name("another_blinky")
        .stack_size(128)
        .priority(TaskPriority(3))
        .start(generate_led_blinky(led_8))
        .unwrap();
    Task::new()
        .name("yet_another_blinky")
        .stack_size(128)
        .priority(TaskPriority(3))
        .start(generate_led_blinky(led_10))
        .unwrap();
    FreeRtosUtils::start_scheduler();
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();
    loop {}
}

#[no_mangle]
fn vApplicationStackOverflowHook(pxTask: FreeRtosTaskHandle, pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
