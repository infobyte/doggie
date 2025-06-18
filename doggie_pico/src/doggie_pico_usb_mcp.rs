#![no_std]
#![no_main]

mod soft_timer;
mod spi;
mod spi_device;
mod unique_id;
mod usb_device;

use cyw43::bluetooth::BtDriver;
use static_cell::StaticCell;
use unique_id::serial_number;

use defmt::info;
use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    peripherals::{SPI0, USB},
    pio::{self, Pio},
    spi::Blocking,
    usb::{self, Driver},
};
use embassy_time::Timer;
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    UsbDevice,
};
use mcp2515::MCP2515;
use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use usb_device::UsbWrapper;
use {defmt_rtt as _, panic_probe as _};

use bt_hci::controller::ExternalController;
use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, PIO0};

use doggie_ble::{types as ble_types, BleSerial, BleServer, SerialMux};

static mut BLE_TX_PIPE: ble_types::BlePipe = ble_types::BlePipe::new();
static mut BLE_RX_PIPE: ble_types::BlePipe = ble_types::BlePipe::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
    PIO0_IRQ_0 => pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(250).await;

        led.set_low();
        Timer::after_millis(250).await;
    }
}

#[embassy_executor::task]
async fn ble_task(mut server: BleServer, controller: ExternalController<BtDriver<'static>, 10>) {
    info!("[BLE] About to run BLE server");
    server.run(controller).await;
    error!("[BLE] Ble task exited");
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Device initialization");
    let p = embassy_rp::init(Default::default());

    // let led = Output::new(p.PIN_25, Level::Low);
    // spawner.spawn(blink_task(led)).unwrap();

    let device_id: &str = serial_number(p.FLASH, p.DMA_CH0);

    let (fw, clm, btfw) = {
        let fw = include_bytes!("../cyw43/43439A0.bin");
        let clm = include_bytes!("../cyw43/43439A0_clm.bin");
        let btfw = include_bytes!("../cyw43/43439A0_btfw.bin");
        (fw, clm, btfw)
    };

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        cyw43_pio::DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        unsafe { embassy_rp::peripherals::DMA_CH0::steal() },
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, bt_device, mut control, runner) =
        cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));
    control.init(clm).await;

    let controller: ExternalController<_, 10> = ExternalController::new(bt_device);

    let (ble_tx_reader, ble_tx_writer) = unsafe { BLE_TX_PIPE.split() };
    let (ble_rx_reader, ble_rx_writer) = unsafe { BLE_RX_PIPE.split() };

    let ble_server = BleServer::new(ble_tx_reader, ble_rx_writer);

    spawner.spawn(ble_task(ble_server, controller)).unwrap();

    let ble_serial = BleSerial::new(ble_tx_writer, ble_rx_reader);

    info!("Serial number: {}", device_id);

    let serial = {
        info!("USB init");

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

        info!("Waiting for USB connection");
        class.wait_connection().await;

        let serial = UsbWrapper::new(class);

        info!("USB init ok");

        serial
    };

    let serial_mux = SerialMux::new(serial, ble_serial);

    // Setup SPI
    let spi = create_default_spi!(p);
    info!("SPI init ok");

    // Create SoftTimer
    let delay = SoftTimer {};

    // Create the Bsp
    // let bsp = Bsp::new(can, uart);
    let bsp = Bsp::new_with_mcp2515(spi, delay, serial_mux);

    info!("MCP2515 init ok");

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

type SerialType = SerialMux<UsbWrapper<'static>, BleSerial>;
type CanType = MCP2515<CustomSpiDevice<'static, SPI0, Blocking>>;

core_create_tasks!(SerialType, CanType);
