#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;

use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;

use defmt::{error, info};
use doggie_core::{Bsp, Core};
use embassy_executor::Spawner;
use embassy_stm32::rcc::*;
use embassy_stm32::{
    gpio::{Level, Output, Pull, Speed},
    spi,
    spi::MODE_0,
    time::Hertz,
    Config as StmConfig,
};
use mcp2515::{regs::OpMode, CanSpeed, McpSpeed, MCP2515};
use {defmt_rtt as _, panic_probe as _};

fn init_bluepill() -> embassy_stm32::Peripherals {
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init_bluepill();

    // let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    // Setup SPI
    let mut spi_config = spi::Config::default();
    spi_config.mode = MODE_0;
    spi_config.frequency = Hertz(1_000_000);
    spi_config.miso_pull = Pull::Down;

    let stm_spi = spi::Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);

    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);

    let spi = CustomSpiDevice::new(stm_spi, cs);

    // MCP2515 initialization
    let mut can = MCP2515::new(spi);
    let mut delay = SoftTimer {};

    match can.init(
        &mut delay,
        mcp2515::Settings {
            mode: OpMode::Normal,         // Loopback for testing and example
            can_speed: CanSpeed::Kbps250, // Many options supported.
            mcp_speed: McpSpeed::MHz8,    // Currently 16MHz and 8MHz chips are supported.
            clkout_en: false,
        },
    ) {
        Ok(_) => info!("MCP2515 Init success"),
        Err(_) => error!("MCP2515 Init Failed"),
    }

    let bsp = Bsp::new(can);
    let mut core = Core::new(bsp);

    core.run().await;
}
