//! This example shows how to use UART (Universal asynchronous receiver-transmitter) in the RP2040 chip.
//!
//! No specific hardware is specified in this example. If you connect pin 0 and 1 you should get the same data back.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;
#[cfg(feature = "usb")]
mod usb_device;

use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{Blocking, Spi};
use gpio::{Level, Output};
use doggie_core::*;
use embassy_rp::{gpio, spi};
use mcp2515::MCP2515;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "uart")]
use {
    embassy_rp::peripherals::UART0,
    embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config},
};

#[cfg(feature = "usb")]
use {
    usb_device::UsbWrapper,
    embassy_rp::peripherals::USB,
    embassy_usb::UsbDevice,
    embassy_rp::usb::{Driver, InterruptHandler},
    embassy_usb::class::cdc_acm::{CdcAcmClass, State},
};

#[cfg(feature = "uart")]
bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[cfg(feature = "usb")]
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[cfg(feature = "usb")]
#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    if !cfg!(feature = "uart") && !cfg!(feature = "usb") {
        panic!("Either 'uart' or 'usb' feature must be enabled.");
    }

    let p = embassy_rp::init(Default::default());

    #[cfg(feature = "uart")]
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

    #[cfg(feature = "usb")]
    let serial = {
        // Create the driver, from the HAL.
        let driver = Driver::new(p.USB, Irqs);

        // Create embassy-usb Config
        let config = {
            let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
            config.manufacturer = Some("Aznarez/Gianatiempo");
            config.product = Some("DoggiePico");
            config.serial_number = Some("1337");
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

        info!("USB init ok");

        serial
    };

    // Setup SPI
    let (clk_pin, tx_pin, rx_pin, cs_pin, spi_no) =
        (p.PIN_18, p.PIN_19, p.PIN_16, p.PIN_17, p.SPI0);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = 1_000_000;

    let rp_spi = Spi::new_blocking(spi_no, clk_pin, tx_pin, rx_pin, spi_config);
    let cs = Output::new(cs_pin, Level::High);
    let spi = CustomSpiDevice::new(rp_spi, cs);
    
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

#[cfg(feature = "uart")]
type SerialType = BufferedUart<'static, UART0>;

#[cfg(feature = "usb")]
type SerialType = UsbWrapper<'static>;

core_create_tasks!(
    SerialType,
    MCP2515<CustomSpiDevice<'static, SPI0, Blocking>>
);
