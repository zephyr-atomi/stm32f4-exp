#![no_std]
#![no_main]

use panic_probe as _;
use defmt_rtt as _;

use cortex_m_rt::entry;
use defmt::info;
use stm32f4xx_hal::{pac, prelude::*, qei::Qei};
use stm32f4xx_hal::pac::EXTI;
use stm32f4xx_hal::gpio::Edge;
use stm32f4xx_hal::interrupt;

static mut Z_SIGNAL_DETECTED: bool = false;
static mut LAST_Z_STATE: bool = false;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().expect("Failed to get stm32 peripherals");
    let cp = cortex_m::peripheral::Peripherals::take().expect("Failed to get cortex_m peripherals");

    // Set up the system clock.
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Create a delay abstraction based on SysTick.
    let mut delay = cp.SYST.delay(&clocks);

    let mut sys_cfg = dp.SYSCFG.constrain();
    let mut exti = dp.EXTI;
    let gpioc = dp.GPIOC.split();

    let mut z_pin = gpioc.pc9.into_floating_input();
    z_pin.make_interrupt_source(&mut sys_cfg);
    z_pin.enable_interrupt(&mut exti);
    z_pin.trigger_on_edge(&mut exti, Edge::RisingFalling); // 配置上升沿触发中断

    // 启用全局中断
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI9_5);
    }

    let gpiob = dp.GPIOB.split();

    // Connect a rotary encoder to pins A0 and A1.
    let rotary_encoder_pins = (gpiob.pb4.into_floating_input(), gpiob.pb5.into_floating_input());
    let rotary_encoder_timer = dp.TIM3;
    let rotary_encoder = Qei::new(rotary_encoder_timer, rotary_encoder_pins);

    let mut current_count = rotary_encoder.count();

    loop {
        let new_count = rotary_encoder.count();
        // info!("curr: {}, new: {}", current_count, new_count);

        if new_count != current_count {
            current_count = new_count;
        }
        unsafe {
            if Z_SIGNAL_DETECTED {
                Z_SIGNAL_DETECTED = false;
                // 在这里处理 Z 信号，例如记录当前状态
                let z_state = LAST_Z_STATE;

                info!("Trigger z signal, z_state = {}", z_state);
            }
        }


        delay.delay_ms(10);
    }
}

#[interrupt]
fn EXTI9_5() {
    // 处理 Z 信号的中断
    let exti = unsafe { &*EXTI::ptr() };
    let gpioc = unsafe { &*pac::GPIOC::ptr() };

    // 读取当前 Z 信号的电平状态
    unsafe {
        LAST_Z_STATE = gpioc.idr().read().idr9().bit(); // 读取 PC9 的电平状态
        Z_SIGNAL_DETECTED = true;
    }

    // 清除中断标志
    exti.pr().write(|w| w.pr9().clear_bit_by_one());
}
