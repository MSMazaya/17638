/**
 * File Name: init.c
 * Author: Scott Pavetti
 * Date: 2-Sept 2024
 * Carnegie Mellon University
 * Course# 17638 Engineeing Embedded Systems
 *
 *
 * Description: Sample code scaffolding for 'hello led' blinky application.
 *
 * Usage: This demo code is intended for the STM Discovery with an STM32F303VTC
 * chip. All ISRs except ResetHandler call into the Default_Handler().
 *
 */

int main(void);
void ResetHandler(void);
void DefaultHandler(void);

// Symbols are declared in the linker script
extern unsigned char INIT_DATA_VALUES;
extern unsigned char INIT_DATA_START;
extern unsigned char INIT_DATA_END;
extern unsigned char BSS_START;
extern unsigned char BSS_END;

// Vectors is placed at the beginning of flash by the linker
// Using the __attribute((section(".vectors"))) we can instruct
// the linker what section to place this array into.
const void *Vectors[] __attribute__((section(".vectors"))) = {
    (void *)0x2000a000, /* Top of stack */
    ResetHandler,       /* Reset Handler */
    DefaultHandler,     /* NMI */
    DefaultHandler,     /* Hard Fault */
    0,                  /* Reserved */
    0,                  /* Reserved */
    0,                  /* Reserved */
    0,                  /* Reserved */
    0,                  /* Reserved */
    0,                  /* Reserved */
    0,                  /* Reserved */
    DefaultHandler,     /* SVC */
    0,                  /* Reserved */
    0,                  /* Reserved */
    DefaultHandler,     /* PendSV */
    DefaultHandler,     /* SysTick */
    /* External interrupt handlers follow */
    DefaultHandler, /* WWDG */
    DefaultHandler, /* PVD */
    DefaultHandler, /* RTC */
    DefaultHandler, /* FLASH */
    DefaultHandler, /* RCC */
    DefaultHandler, /* EXTI0_1 */
    DefaultHandler, /* EXTI2_3 */
    DefaultHandler, /* EXTI4_15 */
    DefaultHandler, /* TSC */
    DefaultHandler, /* DMA_CH1 */
    DefaultHandler, /* DMA_CH2_3 */
    DefaultHandler, /* DMA_CH4_5 */
    DefaultHandler, /* ADC_COMP */
    DefaultHandler, /* TIM1_BRK_UP_TRG_COM */
    DefaultHandler, /* TIM1_CC */
    DefaultHandler, /* TIM2 */
    DefaultHandler, /* TIM3 */
    DefaultHandler, /* TIM6_DAC */
    DefaultHandler, /* RESERVED */
    DefaultHandler, /* TIM14 */
    DefaultHandler, /* TIM15 */
    DefaultHandler, /* TIM16 */
    DefaultHandler, /* TIM17 */
    DefaultHandler, /* I2C1 */
    DefaultHandler, /* I2C2 */
    DefaultHandler, /* SPI1 */
    DefaultHandler, /* SPI2 */
    DefaultHandler, /* USART1 */
    DefaultHandler, /* USART2 */
    DefaultHandler, /* RESERVED */
    DefaultHandler, /* CEC */
    DefaultHandler  /* RESERVED */
};
/**
 * ResetHandler()
 * Set up as the function called when the processor resets.
 * Uses a couple of symbols required to be in the linker file
 * that represent the start and end addresses that are used
 * in the copying and zeroing sequence.
 *
 * 1. Get the addresses from our linker symbols
 * 2. Copy contents of flash to sram
 * 3. Zero the .bss section
 * 4. Call main()
 */
void ResetHandler() {
  // do global/static data initialization
  unsigned char *src;
  unsigned char *dest;
  unsigned len;
  src = &INIT_DATA_VALUES;
  dest = &INIT_DATA_START;
  len = &INIT_DATA_END - &INIT_DATA_START;
  while (len) {
    *dest = *src;
    dest++;
    src++;
    len--;
  }

  // zero out the uninitialized global/static variable locations
  dest = &BSS_START;
  len = &BSS_END - &BSS_START;
  while (len) {
    *dest = 0;
    dest++;
    len--;
  }
  main();
}

/**
 * DefaultHandler()
 *
 * Simple while(1) trap.
 *
 */
void DefaultHandler() {
  while (1)
    ;
}
