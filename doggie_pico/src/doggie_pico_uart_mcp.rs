#![no_std]
#![no_main]

mod soft_timer;
mod spi;
mod spi_device;

use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::{SPI0, UART0},
    spi::Blocking,
    uart::{BufferedInterruptHandler, BufferedUart, Config}
};
use mcp2515::MCP2515;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};
use doggie_core::{
    core_create_tasks,
    core_run,
    Bsp,
    Core,
    CanChannel,
    CanChannelReceiver,
    CanChannelSender
};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let serial = {
        // Setup UART
        let (tx_pin, rx_pin, uart_no) = (p.PIN_0, p.PIN_1, p.UART0);

        let mut uart_config = Config::default();
        uart_config.baudrate = 115200;

        static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
        let tx_buf = &mut TX_BUF.init([0; 16])[..];

        static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
        let rx_buf = &mut RX_BUF.init([0; 16])[..];
        let serial = BufferedUart::new(uart_no, Irqs, tx_pin, rx_pin, tx_buf, rx_buf, uart_config);

        info!("UART init ok");

        serial
    };
    
    // Setup SPI
    let spi = create_default_spi!(p);
    info!("SPI init ok");
    
    // Create SoftTimer
    let delay = SoftTimer {};

    // Create the Bsp
    // let bsp = Bsp::new(can, uart);
    let bsp = Bsp::new_with_mcp2515(spi, delay, serial);

    info!("MCP2515 init ok");    

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

type SerialType = BufferedUart<'static, UART0>;
type CanType = MCP2515<CustomSpiDevice<'static, SPI0, Blocking>>;

core_create_tasks!(
    SerialType,
    CanType
);
