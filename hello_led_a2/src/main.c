/**
 * File Name: main.c
 * Author: Scott Pavetti
 * Date: 2-Sept 2024
 * Carnegie Mellon University
 * Course# 17638 Engineeing Embedded Systems
 *
 * Description: Sample code scaffolding for 'hello led' blinky application.
 * Usage: This demo code is intended for the STM Discovery with an STM32F303VTC
 * chip.
 */

#include "stm32f303xx.h"

// GPIO ODR address is for setting or reading the state of the GPIO

// GPIO MODER register - sets the mode for the gpio port we're using to input or
// output

// RCC - Reset and Clock Control, this has to be set to enable the gpio bank we
// need for the LED's

// https://wiki.st.com/stm32mpu/wiki/RCC_internal_peripheral
// https://fastbitlab.com/microcontroller-embedded-c-programming-lecture-110-enabling-peripheral-clock/
//

/**
 * delay
 * Simple busy decrementing type of delay not intended for serious use.
 */
void delay(int dly) {
  while (dly--)
    ;
}

/**
 * simple_blinky
 * Super simple approach to turning on and off some LEDs
 * on the Discovery board writing directly to the gpio
 * and using the not-recommended-version of delay.
 */
void simple_blinky(void) {
  RCC_AHBENR |= BIT21;
  GPIOE_MODER = BIT16;
  GPIOE_MODER |= GPIOE_MODER << 2;
  GPIOE_MODER |= GPIOE_MODER << 4;
  GPIOE_MODER |= GPIOE_MODER << 8;

  // Superloop we never exit from.
  while (1) {
    GPIOE_ODR = 0;
    delay(500000);
    GPIOE_ODR = (0b10101010 << 8);
    delay(500000);
  }
}

/**
 * main
 * Calls into our superloop and doesn't come back.
 */
int main() {
  simple_blinky();

  return 0;
}
