//! This example shows how to use UART (Universal asynchronous receiver-transmitter) in the RP2040 chip.
//!
//! No specific hardware is specified in this example. If you connect pin 0 and 1 you should get the same data back.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;

use embedded_io_async::Write;
use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;

// use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::peripherals::SPI0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
use embassy_rp::spi::{Spi, Blocking};
use gpio::{Level, Output};
// use embedded_io_async::{Read, Write};
use embassy_rp::{gpio, spi};
use mcp2515::{regs::OpMode, CanSpeed, McpSpeed, MCP2515};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};
use doggie_core::{Bsp, Core};

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
    let mut uart = BufferedUart::new(uart_no, Irqs, tx_pin, rx_pin, tx_buf, rx_buf, uart_config);

    uart.write(b"\r\nUART init ok\r\n").await.unwrap();

    // Setup SPI
    let (clk_pin, tx_pin, rx_pin, cs_pin, spi_no) = (
        p.PIN_18,
        p.PIN_19,
        p.PIN_16,
        p.PIN_17,
        p.SPI0
    );
    
    let mut spi_config = spi::Config::default();
    spi_config.frequency = 1_000_000;
    
    let rp_spi = Spi::new_blocking(
        spi_no,
        clk_pin,
        tx_pin,
        rx_pin,
        spi_config
    );
    let cs = Output::new(cs_pin, Level::High);
    let spi = CustomSpiDevice::new(rp_spi, cs);

    // MCP2515 init
    let mut can = MCP2515::new(spi);
    let mut delay = SoftTimer {};

    can.init(
        &mut delay,
        mcp2515::Settings {
            mode: OpMode::Normal,         // Loopback for testing and example
            can_speed: CanSpeed::Kbps250, // Many options supported.
            mcp_speed: McpSpeed::MHz8,    // Currently 16MHz and 8MHz chips are supported.
            clkout_en: false,
        }
    ).unwrap();

    uart.write(b"CAN init ok\r\n").await.unwrap();
    uart.flush().await.unwrap();

    // Create the Bsp
    let bsp = Bsp::new(can, uart);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    // TODO: This should be replaced with a macro
    // core_run!(core)

    let serial = core.bsp.serial.replace(None).unwrap();
    let can = core.bsp.can.replace(None).unwrap();

    core.spawner.spawn(echo_task(serial)).unwrap();
    // core.spawner.spawn(can_task(can)).unwrap();
}

#[embassy_executor::task]
async fn echo_task(serial: BufferedUart<'static, UART0>) {
    Core::<MCP2515<CustomSpiDevice<SPI0, Blocking>>, BufferedUart<'_, UART0>>::echo(serial).await;
}

#[embassy_executor::task]
async fn can_task(can: MCP2515<CustomSpiDevice<'static, SPI0, Blocking>>) {
    Core::<MCP2515<CustomSpiDevice<'_, SPI0, Blocking>>, BufferedUart<'_, UART0>>::can(can)
        .await;
}