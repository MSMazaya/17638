#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use stm32f3xx_hal::{
    gpio::*,
    pac::{self, interrupt},
    prelude::*,
    timer,
};

type LedPin<const T: u8> = Pin<Gpioe, U<T>, Output<PushPull>>;

static G_LED_9: Mutex<RefCell<Option<LedPin<9>>>> = Mutex::new(RefCell::new(None));
static G_LED_11: Mutex<RefCell<Option<LedPin<11>>>> = Mutex::new(RefCell::new(None));
static G_LED_13: Mutex<RefCell<Option<LedPin<13>>>> = Mutex::new(RefCell::new(None));
static G_LED_15: Mutex<RefCell<Option<LedPin<15>>>> = Mutex::new(RefCell::new(None));
static TIMER: Mutex<RefCell<Option<timer::Timer<pac::TIM2>>>> = Mutex::new(RefCell::new(None));

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();
    let mut rcc = p.RCC.constrain();
    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
    let led_9 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led_11 = gpioe
        .pe11
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led_13 = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led_15 = gpioe
        .pe15
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let clocks = rcc.cfgr.freeze(&mut p.FLASH.constrain().acr);
    let mut timer = timer::Timer::new(p.TIM2, clocks, &mut rcc.apb1);

    timer.enable_interrupt(timer::Event::Update);
    timer.start(500.milliseconds());

    unsafe {
        cortex_m::peripheral::NVIC::unmask(timer.interrupt());
    }

    cortex_m::interrupt::free(|cs| *G_LED_9.borrow(cs).borrow_mut() = Some(led_9));
    cortex_m::interrupt::free(|cs| *G_LED_11.borrow(cs).borrow_mut() = Some(led_11));
    cortex_m::interrupt::free(|cs| *G_LED_13.borrow(cs).borrow_mut() = Some(led_13));
    cortex_m::interrupt::free(|cs| *G_LED_15.borrow(cs).borrow_mut() = Some(led_15));
    cortex_m::interrupt::free(|cs| *TIMER.borrow(cs).borrow_mut() = Some(timer));

    #[allow(clippy::empty_loop)]
    loop {}
}

#[interrupt]
fn TIM2() {
    static mut LED_9: Option<LedPin<9>> = None;
    static mut LED_11: Option<LedPin<11>> = None;
    static mut LED_13: Option<LedPin<13>> = None;
    static mut LED_15: Option<LedPin<15>> = None;
    static mut TIM: Option<timer::Timer<pac::TIM2>> = None;

    let led_9 = LED_9.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| G_LED_9.borrow(cs).replace(None).unwrap())
    });
    let led_11 = LED_11.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| G_LED_11.borrow(cs).replace(None).unwrap())
    });
    let led_13 = LED_13.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| G_LED_13.borrow(cs).replace(None).unwrap())
    });
    let led_15 = LED_15.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| G_LED_15.borrow(cs).replace(None).unwrap())
    });

    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| TIMER.borrow(cs).replace(None).unwrap())
    });

    let _ = led_9.toggle();
    let _ = led_11.toggle();
    let _ = led_13.toggle();
    let _ = led_15.toggle();
    let _ = tim.wait();
}
