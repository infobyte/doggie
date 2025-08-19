#![no_std]
#![no_main]

#[cfg(all(feature = "usb", feature = "int"))]
core::compile_error!(
    "Fature compatibility error: 'usb' and 'int' features can't be enable at the same time"
);

#[cfg(all(not(feature = "uart"), not(feature = "usb")))]
core::compile_error!("Fature error: One serial interface must been selected ('uart' or 'usb')");

#[cfg(all(not(feature = "mcp"), not(feature = "int")))]
core::compile_error!("Fature error: One CAN interface must been selected ('int' or 'mcp')");

#[cfg(feature = "int")]
mod can_device;

#[cfg(feature = "mcp")]
mod soft_timer;
#[cfg(feature = "mcp")]
mod spi;
#[cfg(feature = "mcp")]
mod spi_device;

use bluepill::{self, serial_type};

use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;

#[cfg(feature = "int")]
use {
    can_device::CanWrapper,
    embassy_stm32::{
        can::{
            filter, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
            TxInterruptHandler,
        },
        peripherals::CAN,
    },
};

#[cfg(feature = "mcp")]
use {embassy_stm32::mode, mcp2515::MCP2515, soft_timer::SoftTimer, spi_device::CustomSpiDevice};

#[cfg(any(feature = "int", feature = "usb"))]
use embassy_stm32::bind_interrupts;

bluepill::init_globals!();

#[cfg(feature = "int")]
bind_interrupts!(struct CanIrqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

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
    let p = bluepill::board::init();

    let led = Output::new(p.PC13, Level::High, Speed::Low);

    spawner.spawn(blink_task(led)).unwrap();

    let serial = bluepill::create_serial!(p, spawner);

    #[cfg(feature = "int")]
    let bsp = {
        // Set alternate pin mapping to B8/B9
        embassy_stm32::pac::AFIO
            .mapr()
            .modify(|w| w.set_can1_remap(2));

        let mut can = Can::new(p.CAN, p.PB8, p.PB9, CanIrqs);

        can.modify_filters()
            .enable_bank(0, Fifo::Fifo0, filter::Mask32::accept_all());

        can.modify_config()
            .set_loopback(false)
            .set_silent(false)
            .set_bitrate(500_000);

        // can.enable().await;

        let can_wrapper = CanWrapper::new(can);

        Bsp::new(can_wrapper, serial)
    };

    #[cfg(feature = "mcp")]
    let bsp = {
        // Delay for the MCP2515
        let delay = SoftTimer {};

        // Setup SPI
        let spi = create_default_spi!(p);

        Bsp::new_with_mcp2515(spi, delay, serial)
    };

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    info!("About to run CORE");
    core_run!(core);
}

#[cfg(feature = "int")]
type CanType = CanWrapper<'static>;
#[cfg(feature = "mcp")]
type CanType = MCP2515<CustomSpiDevice<'static, mode::Blocking>>;

core_create_tasks!(serial_type!(), CanType);
