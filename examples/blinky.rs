//! Blinks an LED

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_probe as _;
use defmt_rtt as _;

use stm32f4xx_hal as hal;

use crate::hal::{pac, prelude::*};
use cortex_m_rt::entry;
use defmt::info;

#[entry]
fn main() -> ! {
    let p = pac::Peripherals::take().unwrap();

    let gpioc = p.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    loop {
        info!("hello");
        // iprintln!("hello");
        for _ in 0..100_000 {
            led.set_high();
        }
        for _ in 0..100_000 {
            led.set_low();
        }
    }
}