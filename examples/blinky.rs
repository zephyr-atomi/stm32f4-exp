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
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    // Create a delay abstraction based on SysTick
    let mut delay = cp.SYST.delay(&clocks);

    let gpiob = dp.GPIOB.split();
    let mut led = gpiob.pb2.into_push_pull_output();

    let mut count: i32 = 0;
    loop {
        count = count.wrapping_add(1);
        info!("hello: {}", count);

        delay.delay_ms(1000);
        led.toggle();
    }
}