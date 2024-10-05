#![no_main]
#![no_std]
#![feature(lang_items)]
#![feature(alloc_error_handler)]

use panic_halt as _;

extern crate alloc;
use alloc::sync::Arc;
use core::{
    alloc::Layout,
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
};
use cortex_m::{asm, interrupt::Mutex as CortexMMutex};
use cortex_m_rt::{entry, exception, ExceptionFrame};
use freertos_rust::{FreeRtosHooks, *};
use stm32f3xx_hal::{
    gpio::*,
    interrupt,
    pac::{self, gpioa, Peripherals},
    prelude::*,
    syscfg,
    timer::{Event, Timer},
    Switch,
};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

type LedPin<const T: u8> = Pin<Gpioe, U<T>, Output<PushPull>>;
static G_BTN: CortexMMutex<RefCell<Option<Pin<Gpioa, U<0>, Input>>>> =
    CortexMMutex::new(RefCell::new(None));
static G_RESETTER_SEMAPHORE: CortexMMutex<RefCell<Option<Arc<Semaphore>>>> =
    CortexMMutex::new(RefCell::new(None));

#[derive(PartialEq, Eq, Copy, Clone)]
enum LedDirection {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

struct Leds {
    current_direction: LedDirection,
    northwest: LedPin<8>,
    north: LedPin<9>,
    northeast: LedPin<10>,
    east: LedPin<11>,
    southeast: LedPin<12>,
    south: LedPin<13>,
    southwest: LedPin<14>,
    west: LedPin<15>,
}

impl Leds {
    fn set_high_all_direction(&mut self) {
        let _ = self.north.set_high();
        let _ = self.northeast.set_high();
        let _ = self.east.set_high();
        let _ = self.southeast.set_high();
        let _ = self.south.set_high();
        let _ = self.southwest.set_high();
        let _ = self.west.set_high();
        let _ = self.northwest.set_high();
    }
    fn set_low_all_direction(&mut self) {
        let _ = self.north.set_low();
        let _ = self.northeast.set_low();
        let _ = self.east.set_low();
        let _ = self.southeast.set_low();
        let _ = self.south.set_low();
        let _ = self.southwest.set_low();
        let _ = self.west.set_low();
        let _ = self.northwest.set_low();
    }
    fn set_high_current_direction(&mut self) {
        let _ = match self.current_direction {
            LedDirection::N => self.north.set_high(),
            LedDirection::NE => self.northeast.set_high(),
            LedDirection::E => self.east.set_high(),
            LedDirection::SE => self.southeast.set_high(),
            LedDirection::S => self.south.set_high(),
            LedDirection::SW => self.southwest.set_high(),
            LedDirection::W => self.west.set_high(),
            LedDirection::NW => self.northwest.set_high(),
        };
    }
    fn set_low_current_direction(&mut self) {
        let _ = match self.current_direction {
            LedDirection::N => self.north.set_low(),
            LedDirection::NE => self.northeast.set_low(),
            LedDirection::E => self.east.set_low(),
            LedDirection::SE => self.southeast.set_low(),
            LedDirection::S => self.south.set_low(),
            LedDirection::SW => self.southwest.set_low(),
            LedDirection::W => self.west.set_low(),
            LedDirection::NW => self.northwest.set_low(),
        };
    }
    fn turn_on_current_for(&mut self, duration: Duration) {
        self.set_high_current_direction();
        CurrentTask::delay(duration);
        self.set_low_current_direction();
    }
    fn get_next_direction(&self) -> LedDirection {
        match self.current_direction {
            LedDirection::N => LedDirection::NE,
            LedDirection::NE => LedDirection::E,
            LedDirection::E => LedDirection::SE,
            LedDirection::SE => LedDirection::S,
            LedDirection::S => LedDirection::SW,
            LedDirection::SW => LedDirection::W,
            LedDirection::W => LedDirection::NW,
            LedDirection::NW => LedDirection::N,
        }
    }
    fn to_next_direction(&mut self) {
        self.current_direction = self.get_next_direction();
    }
}

struct App<'a> {
    leds: &'a mut Leds,
    state: AppState,
}

enum AppState {
    Active(u32),
    PreAlarm(u32),
    Alarm,
}

impl AppState {
    fn new() -> AppState {
        AppState::Active(5)
    }
    fn reset(&mut self) {
        *self = AppState::new();
    }
    fn transition(&mut self) {
        *self = match self {
            AppState::Active(_) => AppState::PreAlarm(16),
            AppState::PreAlarm(_) => AppState::Alarm,
            AppState::Alarm => AppState::Active(5),
        };
    }
    fn decrement_counter(&mut self) {
        *self = match self {
            AppState::Active(counter) => AppState::Active(*counter - 1),
            AppState::PreAlarm(counter) => AppState::PreAlarm(*counter - 1),
            AppState::Alarm => AppState::Alarm,
        };
    }
}

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
/*
grand design:
2 task:
    1. For parsing message queue of commands, maintained by another task
    2. Sending commands, based on which state we're currently in
alternatively, you can use another task to send diagnostics.

what is weird is that: this kind of ensures you can reset the queue so that
                       it does not continue the execution!
2 interrupt:
    1. Button:
        - if state is alarm, then go to pre-active
        - reset queue
    2. Accelerometer
*/

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let mut exti = p.EXTI;
    let mut rcc = p.RCC.constrain();
    let mut syscfg = p.SYSCFG.constrain(&mut rcc.apb2);
    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let mut leds = Leds {
        current_direction: LedDirection::N,
        northwest: gpioe
            .pe8
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        north: gpioe
            .pe9
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        northeast: gpioe
            .pe10
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        east: gpioe
            .pe11
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        southeast: gpioe
            .pe12
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        south: gpioe
            .pe13
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        southwest: gpioe
            .pe14
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
        west: gpioe
            .pe15
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper),
    };
    let mut user_btn = gpioa
        .pa0
        .into_pull_down_input(&mut gpioa.moder, &mut gpioa.pupdr);
    let state = Arc::new(Mutex::new(AppState::new()).unwrap());
    let state_resetter_semaphore = Arc::new(Semaphore::new_binary().unwrap());

    syscfg.select_exti_interrupt_source(&user_btn);
    user_btn.trigger_on_edge(&mut exti, Edge::Rising);
    user_btn.enable_interrupt(&mut exti);

    unsafe {
        cortex_m::peripheral::NVIC::unmask(user_btn.interrupt());
    }

    cortex_m::interrupt::free(|cs| {
        *G_BTN.borrow(cs).borrow_mut() = Some(user_btn);
        *G_RESETTER_SEMAPHORE.borrow(cs).borrow_mut() = Some(Arc::clone(&state_resetter_semaphore));
    });

    Task::new()
        .name("state_resetter")
        .stack_size(128)
        .priority(TaskPriority(5))
        .start({
            let s_arc = Arc::clone(&state);
            let s_semaphore_arc = Arc::clone(&state_resetter_semaphore);
            move |_| loop {
                let _ = s_semaphore_arc.take(Duration::infinite());

                if let Ok(mut s) = s_arc.lock(Duration::infinite()) {
                    match *s {
                        AppState::Alarm => s.reset(),
                        _ => {}
                    }
                }
            }
        })
        .unwrap();

    Task::new()
        .name("blinky")
        .stack_size(128)
        .priority(TaskPriority(3))
        .start({
            let s_arc = Arc::clone(&state);
            move |_| loop {
                if let Ok(mut s) = s_arc.lock(Duration::infinite()) {
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
        })
        .unwrap();

    FreeRtosUtils::start_scheduler()
}

#[interrupt]
#[allow(non_snake_case)]
fn EXTI0() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut state_semaphore) = *G_RESETTER_SEMAPHORE.borrow(cs).borrow_mut() {
            state_semaphore.give_from_isr(&mut InterruptContext::new());
        } else {
            asm::bkpt();
        }

        if let Some(ref mut btn) = *G_BTN.borrow(cs).borrow_mut() {
            btn.clear_interrupt();
        } else {
            asm::bkpt();
        }
    });
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();
    loop {}
}

#[no_mangle]
fn vApplicationStackOverflowHook(pxTask: FreeRtosTaskHandle, pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
