use alloc::sync::Arc;
use core::{alloc::Layout, cell::RefCell};
use cortex_m::{
    asm,
    interrupt::{InterruptNumber, Mutex as CortexMMutex},
};
use cortex_m_rt::{exception, ExceptionFrame};
use freertos_rust::*;
use stm32f3xx_hal::{gpio::*, interrupt};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;
static G_BTN: CortexMMutex<RefCell<Option<Pin<Gpioa, U<0>, Input>>>> =
    CortexMMutex::new(RefCell::new(None));
static G_RESETTER_SEMAPHORE: CortexMMutex<RefCell<Option<Arc<Semaphore>>>> =
    CortexMMutex::new(RefCell::new(None));

pub fn setup_interrupt(interrupt_number: impl InterruptNumber) {
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt_number);
    }
}

pub fn setup_interrupt_resource(
    user_btn: Pin<Gpioa, U<0>, Input>,
    task_resetter_semaphore_arc: Arc<Semaphore>,
) {
    cortex_m::interrupt::free(|cs| {
        *G_BTN.borrow(cs).borrow_mut() = Some(user_btn);
        *G_RESETTER_SEMAPHORE.borrow(cs).borrow_mut() = Some(task_resetter_semaphore_arc);
    });
}

#[interrupt]
#[allow(non_snake_case)]
fn EXTI0() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut state_semaphore) = *G_RESETTER_SEMAPHORE.borrow(cs).borrow_mut() {
            state_semaphore.give_from_isr(&mut InterruptContext::new());
        }
        if let Some(ref mut btn) = *G_BTN.borrow(cs).borrow_mut() {
            btn.clear_interrupt();
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
