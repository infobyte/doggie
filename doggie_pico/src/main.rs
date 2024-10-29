//! This example shows how to use UART (Universal asynchronous receiver-transmitter) in the RP2040 chip.
//!
//! No specific hardware is specified in this example. If you connect pin 0 and 1 you should get the same data back.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;

use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::SPI0;
use embassy_rp::peripherals::UART0;
use embassy_rp::spi::{Blocking, Spi};
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
use gpio::{Level, Output};
use doggie_core::*;
use embassy_rp::{gpio, spi};
use mcp2515::MCP2515;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Setup UART
    let (tx_pin, rx_pin, uart_no) = (p.PIN_0, p.PIN_1, p.UART0);

    let mut uart_config = Config::default();
    uart_config.baudrate = 115200;

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];

    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];
    let uart = BufferedUart::new(uart_no, Irqs, tx_pin, rx_pin, tx_buf, rx_buf, uart_config);

    info!("UART init ok");

    // Setup SPI
    let (clk_pin, tx_pin, rx_pin, cs_pin, spi_no) =
        (p.PIN_18, p.PIN_19, p.PIN_16, p.PIN_17, p.SPI0);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = 1_000_000;

    let rp_spi = Spi::new_blocking(spi_no, clk_pin, tx_pin, rx_pin, spi_config);
    let cs = Output::new(cs_pin, Level::High);
    let spi = CustomSpiDevice::new(rp_spi, cs);
    let delay = SoftTimer {};

    info!("SPI init ok");

    // Create the Bsp
    // let bsp = Bsp::new(can, uart);
    let bsp = Bsp::new_with_mcp2515(spi, delay, uart);

    info!("MCP2515 init ok");    

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

core_create_tasks!(
    BufferedUart<'static, UART0>,
    MCP2515<CustomSpiDevice<'static, SPI0, Blocking>>
);
