#![no_std]
#![no_main]

mod bluepill;
mod soft_timer;
mod spi;
mod spi_device;
mod uart;
mod uart_device;

use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use uart_device::UartWrapper;

use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};

use defmt::info;
use mcp2515::MCP2515;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode;
use embassy_time::Timer;

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = bluepill::init();

    let led = Output::new(p.PC13, Level::High, Speed::Low);

    spawner.spawn(blink_task(led)).unwrap();

    let serial = create_default_uart!(p);

    // Delay for the MCP2515
    let delay = SoftTimer {};

    // Setup SPI
    let spi = create_default_spi!(p);

    let bsp = Bsp::new_with_mcp2515(spi, delay, serial);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    info!("About to run CORE");
    core_run!(core);
}

type SerialType = UartWrapper<'static>;
type CanType = MCP2515<CustomSpiDevice<'static, mode::Blocking>>;

core_create_tasks!(SerialType, CanType);
