use embassy_stm32::rcc::*;
use embassy_stm32::{time::Hertz, Config as StmConfig};

pub fn init() -> embassy_stm32::Peripherals {
    let mut config = StmConfig::default();

    // Clock configuration to run at 72MHz (Max)
    {
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            // Oscillator for bluepill, Bypass for nucleos.
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }

    embassy_stm32::init(config)
}
