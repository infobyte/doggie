#![no_std]
#![no_main]

mod ble;
mod logging;
mod serial_mux;
mod spi_device;
mod soft_timer;

use ble::{BleSerial, BleServer, PIPE_CAPACITY};
use logging::init_logs;
use serial_mux::SerialMux;
use spi_device::CustomSpiDevice;
use soft_timer::SoftTimer;

use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pipe::Pipe;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    spi::{master::Spi, SpiMode},
    clock::CpuClock,
    gpio::{Level, Output},
    uart::Uart,
    Async,
    Blocking,
    prelude::*,
    usb_serial_jtag::UsbSerialJtag,
};

use defmt::info;
use ::mcp2515::MCP2515 as MCP;
use doggie_core::*;


static mut BLE_TX_PIPE: Pipe<CriticalSectionRawMutex, PIPE_CAPACITY> = Pipe::new();
static mut BLE_RX_PIPE: Pipe<CriticalSectionRawMutex, PIPE_CAPACITY> = Pipe::new();

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::task]
pub async fn ble_task(mut server: BleServer<'static>) {
    server.run().await;
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // info!("Device initialization started");
    // Board initialization
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    // Heap initialization needed by the BLE
    esp_alloc::heap_allocator!(72 * 1024);

    // Setup embassy timer
    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32")] {
            let timg1 = TimerGroup::new(peripherals.TIMG1);
            esp_hal_embassy::init(timg1.timer0);
        } else {
            use esp_hal::timer::systimer::{SystemTimer, Target};
            let systimer = SystemTimer::new(peripherals.SYSTIMER).split::<Target>();
            esp_hal_embassy::init(systimer.alarm0);
        }
    }

    // Blink initialization
    let led = Output::new(peripherals.GPIO8, Level::Low);
    spawner.spawn(blink_task(led)).unwrap();
    
    // Serial logging initialization
    // info!("Debug serial init");
    let dbg_serial = {
        let (tx_pin, rx_pin) = (peripherals.GPIO3, peripherals.GPIO2);
        let config = esp_hal::uart::Config::default().baudrate(115200);

        Uart::new_with_config(peripherals.UART1, config, rx_pin, tx_pin)
            .unwrap()
    };

    let (_, dbg_tx) = dbg_serial.split();
    init_logs(dbg_tx);

    // BLE initialization
    info!("BLE init");
    let (ble_tx_reader, ble_tx_writer) = unsafe { BLE_TX_PIPE.split() };
    let (ble_rx_reader, ble_rx_writer) = unsafe { BLE_RX_PIPE.split() };

    let ble_server = BleServer::new(
        peripherals.BT,
        peripherals.TIMG0,
        peripherals.RNG,
        peripherals.RADIO_CLK,
        ble_tx_reader,
        ble_rx_writer,
    );
    
    spawner.spawn(ble_task(ble_server)).unwrap();

    let ble_serial = BleSerial::new(
        ble_tx_writer, ble_rx_reader
    );

    info!("Wired serial init");
    // Wired serial initialization
    #[cfg(feature = "esp32c3")]
    let wired_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();

    // Setup UART (using these pins, also passes through USB)
    #[cfg(not(feature = "esp32c3"))]
    let wired_serial = {
        let (tx_pin, rx_pin) = (peripherals.GPIO1, peripherals.GPIO3);
        let config = esp_hal::uart::Config::default().rx_fifo_full_threshold(READ_BUF_SIZE as u16);

        Uart::new_with_config(peripherals.UART0, config, rx_pin, tx_pin)
            .unwrap()
            .into_async()
    };

    let serial = SerialMux::new(ble_serial, wired_serial);
    
    // CAN bus initialization
    info!("CAN Bus init");
    
    // Setup SPI
    #[cfg(feature = "esp32c3")]
    let (sclk, mosi, miso, cs, spi) = (peripherals.GPIO9, peripherals.GPIO6, peripherals.GPIO5, peripherals.GPIO7, peripherals.SPI2);
    #[cfg(feature = "esp32")]
    let (sclk, mosi, miso, cs, spi) = (peripherals.GPIO14, peripherals.GPIO13, peripherals.GPIO12, peripherals.GPIO15, peripherals.SPI2);
    let esp_spi = Spi::new_with_config(
        spi,
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

    // Create SoftTimer
    let delay = SoftTimer {};
    
    // Create the Bsp
    info!("BSP creation");
    let bsp = Bsp::new_with_mcp2515(spi, delay, serial);

    // Create and run the Doggie core
    info!("Core creation");
    let core = Core::new(spawner, bsp);

    info!("About to run core...");
    core_run!(core);
}

#[cfg(feature = "esp32c3")]
type UartType = UsbSerialJtag<'static, Async>;

#[cfg(not(feature = "esp32c3"))]
type UartType = Uart<'static, Async>;

core_create_tasks!(SerialMux<BleSerial, UartType>, MCP<CustomSpiDevice<'static, Blocking>>);
