#![no_std]
#![no_main]

mod soft_timer;
mod spi;
mod spi_device;

use rp::{self, serial_type};

use static_cell::StaticCell;

use defmt::info;
use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    peripherals::SPI0,
    spi::Blocking,
};
use mcp2515::MCP2515;
use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;

rp::init_globals!();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Device initialization");
    let p = embassy_rp::init(Default::default());

    let serial = rp::create_serial!(p, spawner);

    // Setup SPI
    let spi = create_default_spi!(p);
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

type SerialType = serial_type!();

type CanType = MCP2515<CustomSpiDevice<'static, SPI0, Blocking>>;

core_create_tasks!(SerialType, CanType);
