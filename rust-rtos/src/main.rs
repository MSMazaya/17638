#![no_main]
#![no_std]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

// Halt on panic
use panic_halt as _;

extern crate alloc;
use alloc::sync::Arc;
use core::alloc::Layout;
use cortex_m::asm;
use cortex_m_rt::{entry, exception, ExceptionFrame};
use freertos_rust::*;
use stm32f3xx_hal::{
    gpio::*,
    pac::{self, gpioa},
    prelude::*,
};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;
static mut S1: Option<Semaphore> = None;
static mut S2: Option<Semaphore> = None;
static mut S3: Option<Semaphore> = None;

type LedPin<const T: u8> = Pin<Gpioe, U<T>, Output<PushPull>>;

// fn generate_led_blinky<const T: u8>(
//     mut led: LedPin<T>,
//     mut s_before: Option<&'static mut Semaphore>,
//     mut s_after: Option<&'static mut Semaphore>,
// ) -> impl FnMut(Task) -> () {
//     move |_| loop {
//         unsafe {
//             let _ = led.set_low();
//             let s1 = s_before.as_mut().unwrap();
//             let _ = s1.take(Duration::infinite());
//             let _ = led.set_high();
//             freertos_rust::CurrentTask::delay(Duration::ms(1000));
//             let s2 = s_after.as_mut().unwrap();
//             s2.give();
//             freertos_rust::CurrentTask::delay(Duration::ms(20));
//         }
//     }
// }

fn generate_led_blinky<const T: u8>(
    mut led: LedPin<T>,
    s: Arc<Semaphore>,
) -> impl FnMut(Task) -> () {
    move |_| loop {
        let _ = led.set_low();
        let _ = s.take(Duration::infinite());
        let _ = led.set_high();
        freertos_rust::CurrentTask::delay(Duration::ms(1000));
        s.give();
        freertos_rust::CurrentTask::delay(Duration::ms(20));
    }
}

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let mut rcc = p.RCC.constrain();
    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let led_9 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led_8 = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led_10 = gpioe
        .pe10
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let user_btn = gpioa
        .pa0
        .into_pull_up_input(&mut gpioa.moder, &mut gpioa.pupdr);

    // user_btn.enable_interrupt();
    // SB20 solder bridge
    // B1 user btn
    // PA0

    unsafe {
        S1 = Some(Semaphore::new_counting(1, 1).unwrap());
        S2 = Some(Semaphore::new_counting(1, 0).unwrap());
        S3 = Some(Semaphore::new_counting(1, 0).unwrap());
    }

    let mut s = Arc::new(Semaphore::new_counting(1, 1).unwrap());

    Task::new()
        .name("blinky")
        .stack_size(128)
        .priority(TaskPriority(4))
        .start(generate_led_blinky(led_8, Arc::clone(&mut s)))
        .unwrap();
    Task::new()
        .name("another_blinky")
        .stack_size(128)
        .priority(TaskPriority(4))
        .start(generate_led_blinky(led_9, Arc::clone(&mut s)))
        .unwrap();
    // Task::new()
    //     .name("blinky")
    //     .stack_size(128)
    //     .priority(TaskPriority(3))
    //     .start(generate_led_blinky(led_9, unsafe { S1.as_mut() }, unsafe {
    //         S2.as_mut()
    //     }))
    //     .unwrap();
    // Task::new()
    //     .name("another_blinky")
    //     .stack_size(128)
    //     .priority(TaskPriority(2))
    //     .start(generate_led_blinky(led_8, unsafe { S2.as_mut() }, unsafe {
    //         S3.as_mut()
    //     }))
    //     .unwrap();
    // Task::new()
    //     .name("yet_another_blinky")
    //     .stack_size(128)
    //     .priority(TaskPriority(1))
    //     .start(generate_led_blinky(
    //         led_10,
    //         unsafe { S3.as_mut() },
    //         unsafe { S1.as_mut() },
    //     ))
    //     .unwrap();
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
