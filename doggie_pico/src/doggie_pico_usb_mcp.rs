#![no_std]
#![no_main]

mod soft_timer;
mod spi;
mod spi_device;
mod unique_id;
mod usb_device;

use unique_id::serial_number;

use defmt::info;
use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::{SPI0, USB},
    spi::Blocking,
    usb::{Driver, InterruptHandler},
};
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    UsbDevice,
};
use mcp2515::MCP2515;
use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use static_cell::StaticCell;
use usb_device::UsbWrapper;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let device_id: &str = serial_number(p.FLASH, p.DMA_CH0);

    let serial = {
        // Create the driver, from the HAL.
        let driver = Driver::new(p.USB, Irqs);

        // Create embassy-usb Config
        let config = {
            let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
            config.manufacturer = Some("Aznarez/Gianatiempo");
            config.product = Some("DoggiePico");
            config.serial_number = Some(device_id);
            config.max_power = 100;
            config.max_packet_size_0 = 64;

            // Required for windows compatibility.
            // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
            config.device_class = 0xEF;
            config.device_sub_class = 0x02;
            config.device_protocol = 0x01;
            config.composite_with_iads = true;
            config
        };

        // Create embassy-usb DeviceBuilder using the driver and config.
        // It needs some buffers for building the descriptors.
        let mut builder = {
            static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
            static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
            static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

            let builder = embassy_usb::Builder::new(
                driver,
                config,
                CONFIG_DESCRIPTOR.init([0; 256]),
                BOS_DESCRIPTOR.init([0; 256]),
                &mut [], // no msos descriptors
                CONTROL_BUF.init([0; 64]),
            );
            builder
        };

        // Create classes on the builder.
        let mut class = {
            static STATE: StaticCell<State> = StaticCell::new();
            let state = STATE.init(State::new());
            CdcAcmClass::new(&mut builder, state, 64)
        };

        // Build the builder.
        let usb = builder.build();

        // Run the USB device.
        spawner.spawn(usb_task(usb)).unwrap();

        class.wait_connection().await;

        let serial = UsbWrapper::new(class);

        info!("USB init ok with serial number: {}", device_id);

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

type SerialType = UsbWrapper<'static>;
type CanType = MCP2515<CustomSpiDevice<'static, SPI0, Blocking>>;

core_create_tasks!(SerialType, CanType);
