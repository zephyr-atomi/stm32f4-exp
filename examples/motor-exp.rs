#![no_std]
#![no_main]

use panic_probe as _;
use defmt_rtt as _;

use cortex_m_rt::entry;
use defmt::{error, info};
use embedded_hal::delay::DelayNs;
use embedded_hal::pwm::SetDutyCycle;
use stm32f4xx_hal::pac;
use stm32f4xx_hal::prelude::{_fugit_RateExtU32, _stm32f4xx_hal_gpio_GpioExt, _stm32f4xx_hal_rcc_RccExt, _stm32f4xx_hal_timer_SysCounterExt};
use stm32f4xx_hal::timer::{Polarity, Timer};

/*
    EN_GATE: PB12
    nFAULT: PD2
    M0_SO1: PC0
    M0_SO2: PC1

    M0_AH: PA8
    M0_AL: PB13
    M0_BH: PA9
    M0_BL: PB14
    M0_CH: PA10
    M0_CL: PB15

    SPI_SCK: PC10
    SPI_MISO: PC11
    SPI_MOSI: PC12
    M0_SPI_nCS: PC13

    M0_TEMP: PC5
 */
#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // 时钟配置
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(168.MHz()).freeze();

    // 配置 GPIO
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpiod = dp.GPIOD.split();

    let tim1 = Timer::new(dp.TIM1, &clocks);
    let (_, (ch1, ch2, ch3, ..)) = tim1.pwm_hz(40.kHz());
    let mut ch_a = ch1.with(gpioa.pa8).with_complementary(gpiob.pb13);
    let mut ch_b = ch2.with(gpioa.pa9).with_complementary(gpiob.pb14);
    let mut ch_c = ch3.with(gpioa.pa10).with_complementary(gpiob.pb15);

    let max_duty = ch_a.get_max_duty();
    let duty = max_duty / 100 * 30;
    info!("max_duty = {}", max_duty);

    ch_a.set_duty(duty);
    ch_b.set_duty(duty);
    ch_c.set_duty(duty);

    ch_a.set_polarity(Polarity::ActiveHigh);
    ch_a.set_complementary_polarity(Polarity::ActiveHigh);
    ch_b.set_polarity(Polarity::ActiveHigh);
    ch_b.set_complementary_polarity(Polarity::ActiveHigh);
    ch_c.set_polarity(Polarity::ActiveHigh);
    ch_c.set_complementary_polarity(Polarity::ActiveHigh);

    ch_a.disable();
    ch_a.disable_complementary();
    ch_b.disable();
    ch_b.disable_complementary();
    ch_c.disable();
    ch_c.disable_complementary();

    let mut delay = cp.SYST.delay(&clocks);
    let mut led = gpiob.pb2.into_push_pull_output();
    let mut count: i32 = 0;

    let mut en_gate = gpiob.pb12.into_push_pull_output(); // EN_GATE 引脚
    let fault = gpiod.pd2.into_pull_up_input();  // nFAULT 引脚
    info!("before fault is_low: {}", fault.is_low());
    en_gate.set_high();
    delay.delay_ms(100);
    info!("after fault is_low: {}", fault.is_low());
    let mut step = 0u8;
    loop {
        if fault.is_low() {
            en_gate.set_low();
            error!("Driver fault detected");
            panic!("Driver fault detected");
        }

        /*
            AH/BL, AH/CL, BH/CL, BH/AL, CH/AL, CH/BL
         */
        match step {
            0 => {
                ch_c.disable();
                ch_a.enable();
                ch_b.enable_complementary();
            }
            1 => {
                ch_b.disable_complementary();
                ch_a.enable();
                ch_c.enable_complementary();
            }
            2 => {
                ch_a.disable();
                ch_b.enable();
                ch_c.enable_complementary();
            }
            3 => {
                ch_c.disable_complementary();
                ch_b.enable();
                ch_a.enable_complementary();
            }
            4 => {
                ch_b.disable();
                ch_c.enable();
                ch_a.enable_complementary();
            }
            5 => {
                ch_a.disable_complementary();
                ch_c.enable();
                ch_b.enable_complementary();
            }
            _ => {
                // error!("step out of range: {}", step);
                error!("Incorrect step: {}", step);
                panic!("Incorrect step: {}", step);
            }
        }

        step = step.wrapping_add(1) % 6;
        delay.delay_ms(10);
    }
}
