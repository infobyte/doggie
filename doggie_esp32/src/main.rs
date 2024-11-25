//! embassy serial
//!
//! This is an example of running the embassy executor and asynchronously
//! writing to and reading from UART.

//% CHIPS: esp32 esp32c2 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3
//% FEATURES: embassy embassy-generic-timers

#![no_std]
#![no_main]

mod spi_device;
mod soft_timer;

use soft_timer::SoftTimer;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_println as _;
use esp_hal::{
    prelude::*,
    spi::{
        master::Spi,
        SpiMode,
    },
    timer::timg::TimerGroup,
    uart::Uart,
    Async,
    Blocking
};
use mcp2515::MCP2515;
use defmt::info;
use doggie_core::*;
use spi_device::CustomSpiDevice;

const READ_BUF_SIZE: usize = 64;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    info!("Init!");
    let p = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Setup UART (using these pins, also passes through USB)
    let (tx_pin, rx_pin) = (p.GPIO1, p.GPIO3);
    
    let config = esp_hal::uart::Config::default().rx_fifo_full_threshold(READ_BUF_SIZE as u16);

    let serial = Uart::new_with_config(p.UART0, config, rx_pin, tx_pin)
        .unwrap()
        .into_async();

    info!("Serial init ok");

    // Setup SPI
    let (sclk, mosi, miso, cs) = (p.GPIO14, p.GPIO13, p.GPIO12, p.GPIO15);
    let esp_spi = Spi::new_with_config(
        p.SPI2,
        esp_hal::spi::master::Config {
            frequency: 1.MHz(),
            mode: SpiMode::Mode0,
            ..esp_hal::spi::master::Config::default()
        },
    )
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso)
    .with_cs(cs);


    let spi = CustomSpiDevice::new(esp_spi);

    info!("SPI init ok");

    // Create SoftTimer
    let delay = SoftTimer {};

    // Create the Bsp
    let bsp = Bsp::new_with_mcp2515(spi, delay, serial);

    info!("MCP2515 init ok");    

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

core_create_tasks!(
    Uart<'static, Async>,
    MCP2515<CustomSpiDevice<'static, Blocking>>
);
