#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;

// use defmt_rtt as _;
use esp_println;

// use defmt::info;
use doggie_core::*;
use embassy_executor;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_io_async::*;
use esp_backtrace as _;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{
    gpio::{Level, Output},
    prelude::*,
    spi::{master::Spi, SpiMode},
    timer::timg::TimerGroup,
    // timer::Timer,
    uart::Uart,
    Async,
    Blocking,
};
use mcp2515::MCP2515;
use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;

const READ_BUF_SIZE: usize = 64;

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // info!("Init!");

    let mut conf = esp_hal::Config::default();

    let p = esp_hal::init(conf);

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let led = Output::new(p.GPIO8, Level::Low);
    spawner.spawn(blink_task(led)).unwrap();

    #[cfg(feature = "esp32c3")]
    let mut serial = UsbSerialJtag::new(p.USB_DEVICE).into_async();

    // Setup UART (using these pins, also passes through USB)
    #[cfg(not(feature = "esp32c3"))]
    let serial = {
        let (tx_pin, rx_pin) = (p.GPIO1, p.GPIO3);
        let config = esp_hal::uart::Config::default().rx_fifo_full_threshold(READ_BUF_SIZE as u16);

        Uart::new_with_config(p.UART0, config, rx_pin, tx_pin)
            .unwrap()
            .into_async()
    };

    // info!("Serial init ok");

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

    // info!("SPI init ok");

    // Create SoftTimer
    let delay = SoftTimer {};

    // Create the Bsp
    let bsp = Bsp::new_with_mcp2515(spi, delay, serial);

    // info!("MCP2515 init ok");

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

#[cfg(feature = "esp32c3")]
type UartType = UsbSerialJtag<'static, Async>;

#[cfg(not(feature = "esp32c3"))]
type UartType = Uart<'static, Async>;

core_create_tasks!(UartType, MCP2515<CustomSpiDevice<'static, Blocking>>);
