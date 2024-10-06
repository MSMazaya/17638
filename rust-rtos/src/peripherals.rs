use freertos_rust::{CurrentTask, Duration};
use lsm303dlhc::Lsm303dlhc;
use stm32f3xx_hal::{
    gpio::*,
    i2c::I2c,
    pac::{self, I2C1},
    prelude::*,
};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum LedDirection {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

pub type LedPin<const T: u8> = Pin<Gpioe, U<T>, Output<PushPull>>;

pub struct Leds {
    pub current_direction: LedDirection,
    pub northwest: LedPin<8>,
    pub north: LedPin<9>,
    pub northeast: LedPin<10>,
    pub east: LedPin<11>,
    pub southeast: LedPin<12>,
    pub south: LedPin<13>,
    pub southwest: LedPin<14>,
    pub west: LedPin<15>,
}

impl Leds {
    pub fn set_high_all_direction(&mut self) {
        let _ = self.north.set_high();
        let _ = self.northeast.set_high();
        let _ = self.east.set_high();
        let _ = self.southeast.set_high();
        let _ = self.south.set_high();
        let _ = self.southwest.set_high();
        let _ = self.west.set_high();
        let _ = self.northwest.set_high();
    }
    pub fn set_low_all_direction(&mut self) {
        let _ = self.north.set_low();
        let _ = self.northeast.set_low();
        let _ = self.east.set_low();
        let _ = self.southeast.set_low();
        let _ = self.south.set_low();
        let _ = self.southwest.set_low();
        let _ = self.west.set_low();
        let _ = self.northwest.set_low();
    }
    pub fn set_high_current_direction(&mut self) {
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
    pub fn set_low_current_direction(&mut self) {
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
    pub fn turn_on_current_for(&mut self, duration: Duration) {
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
    pub fn to_next_direction(&mut self) {
        self.current_direction = self.get_next_direction();
    }
}

pub type Accelerometer = Lsm303dlhc<
    I2c<
        I2C1,
        (
            Pin<Gpiob, U<6>, Alternate<OpenDrain, 4>>,
            Pin<Gpiob, U<7>, Alternate<OpenDrain, 4>>,
        ),
    >,
>;

pub fn setup() -> (Leds, Pin<Gpioa, U<0>, Input>, Accelerometer) {
    let p = pac::Peripherals::take().unwrap();
    let mut exti = p.EXTI;
    let mut rcc = p.RCC.constrain();
    let mut syscfg = p.SYSCFG.constrain(&mut rcc.apb2);
    let mut gpioe = p.GPIOE.split(&mut rcc.ahb);
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb);
    let leds = Leds {
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
    let mut flash = p.FLASH.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut scl =
        gpiob
            .pb6
            .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let mut sda =
        gpiob
            .pb7
            .into_af_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    scl.internal_pull_up(&mut gpiob.pupdr, true);
    sda.internal_pull_up(&mut gpiob.pupdr, true);
    let i2c = stm32f3xx_hal::i2c::I2c::new(
        p.I2C1,
        (scl, sda),
        100.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );
    let mut accelerometer = lsm303dlhc::Lsm303dlhc::new(i2c).unwrap();
    let _ = accelerometer.accel_odr(lsm303dlhc::AccelOdr::Hz100);
    let _ = accelerometer.set_accel_sensitivity(lsm303dlhc::Sensitivity::G12);
    syscfg.select_exti_interrupt_source(&user_btn);
    user_btn.trigger_on_edge(&mut exti, Edge::Rising);
    user_btn.enable_interrupt(&mut exti);

    (leds, user_btn, accelerometer)
}
