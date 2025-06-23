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

mod bluepill;

#[cfg(feature = "int")]
mod can_device;

#[cfg(feature = "uart")]
mod uart;
#[cfg(feature = "uart")]
mod uart_device;

#[cfg(feature = "mcp")]
mod soft_timer;
#[cfg(feature = "mcp")]
mod spi;
#[cfg(feature = "mcp")]
mod spi_device;

#[cfg(feature = "usb")]
mod usb_device;

use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;

#[cfg(feature = "usb")]
use {
    embassy_stm32::{peripherals::USB, usb, usb::Driver},
    embassy_usb::{
        class::cdc_acm::{CdcAcmClass, State},
        UsbDevice,
    },
    static_cell::StaticCell,
    usb_device::UsbWrapper,
};

#[cfg(feature = "uart")]
use uart_device::UartWrapper;

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

#[cfg(feature = "int")]
bind_interrupts!(struct CanIrqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

#[cfg(feature = "usb")]
bind_interrupts!(struct UsbIrqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<USB>;
});

#[cfg(feature = "usb")]
#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, embassy_stm32::usb::Driver<'static, USB>>) {
    usb.run().await;
}
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
    #[cfg(feature = "usb")]
    let mut p = bluepill::init();

    #[cfg(not(feature = "usb"))]
    let p = bluepill::init();

    let led = Output::new(p.PC13, Level::High, Speed::Low);

    spawner.spawn(blink_task(led)).unwrap();

    #[cfg(feature = "uart")]
    let serial = create_default_uart!(p);

    #[cfg(feature = "usb")]
    let serial = {
        {
            // BluePill board has a pull-up resistor on the D+ line.
            // Pull the D+ pin down to send a RESET condition to the USB bus.
            // This forced reset is needed only for development, without it host
            // will not reset your device when you upload new firmware.
            let _dp = Output::new(&mut p.PA12, Level::Low, Speed::Low);
            Timer::after_millis(10).await;
        }

        // Create the driver, from the HAL.
        let driver = Driver::new(p.USB, UsbIrqs, p.PA12, p.PA11);

        // Create embassy-usb Config
        let config = {
            let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
            config.manufacturer = Some("Aznarez/Gianatiempo");
            config.product = Some("DoggieBluepill");
            config.serial_number = Some("1337");
            config.max_power = 100;
            config.max_packet_size_0 = 64;
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

        info!("Building USB");
        // Build the builder.
        let usb = builder.build();

        // Run the USB device.
        spawner.spawn(usb_task(usb)).unwrap();

        info!("Waiting for USB connection");
        class.wait_connection().await;

        UsbWrapper::new(class)
    };

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
            .set_bitrate(250_000);

        can.enable().await;

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

#[cfg(feature = "uart")]
type SerialType = UartWrapper<'static>;
#[cfg(feature = "usb")]
type SerialType = UsbWrapper<'static>;

#[cfg(feature = "int")]
type CanType = CanWrapper<'static>;
#[cfg(feature = "mcp")]
type CanType = MCP2515<CustomSpiDevice<'static, mode::Blocking>>;

core_create_tasks!(SerialType, CanType);
