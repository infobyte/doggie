#![no_std]
#![no_main]

mod logging;
mod serial_mux;
mod twai_can;

use logging::init_logs;
use serial_mux::SerialMux;
use twai_can::CanWrapper;

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pipe::{Pipe, Reader, Writer},
};
use embassy_time::Timer;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output},
    peripheral::{self, Peripheral},
    uart::Uart,
    usb_serial_jtag::UsbSerialJtag,
    Async,
};

use bt_hci::controller::ExternalController;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::ble::controller::BleConnector;
use esp_wifi::EspWifiController;

use defmt::{error, info};
use doggie_ble;
use doggie_core::*;

use doggie_ble::host::{BleSerial, BleServer, L2CAP_MTU};

use embedded_io_async::{Read, Write};

static mut BLE_TX_PIPE: Pipe<CriticalSectionRawMutex, L2CAP_MTU> = Pipe::new();
static mut BLE_RX_PIPE: Pipe<CriticalSectionRawMutex, L2CAP_MTU> = Pipe::new();

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
async fn ble_task(
    timg0_p: esp_hal::peripherals::TIMG0,
    rng_p: esp_hal::peripherals::RNG,
    clk_p: esp_hal::peripherals::RADIO_CLK,
    bt: esp_hal::peripherals::BT,
    reader: Reader<'static, CriticalSectionRawMutex, L2CAP_MTU>,
    writer: Writer<'static, CriticalSectionRawMutex, L2CAP_MTU>,
) {
    let timg0 = TimerGroup::new(timg0_p);

    let init = esp_wifi::init(timg0.timer0, esp_hal::rng::Rng::new(rng_p), clk_p).unwrap();

    let connector = BleConnector::new(&init, bt);
    let controller: ExternalController<BleConnector<'_>, 20> = ExternalController::new(connector);

    let mut ble_server = BleServer::new(reader, writer);

    info!("[BLE] About to run BLE server");

    ble_server.run(controller).await;
    error!("[BLE] Ble task exited");
}

static mut BLE: Option<EspWifiController<'static>> = None;

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
            use esp_hal::timer::timg::TimerGroup,

            let timg1 = TimerGroup::new(peripherals.TIMG1);
            esp_hal_embassy::init(timg1.timer0);
        } else {
            use esp_hal::timer::systimer::SystemTimer;

            let systimer = SystemTimer::new(peripherals.SYSTIMER);
            esp_hal_embassy::init(systimer.alarm0);
        }
    }

    // Blink initialization
    let led = Output::new(peripherals.GPIO8, Level::Low);
    spawner.spawn(blink_task(led)).unwrap();

    // Serial logging initialization
    info!("Debug serial init");
    let dbg_serial = {
        let (tx_pin, rx_pin) = (peripherals.GPIO3, peripherals.GPIO2);
        let config = esp_hal::uart::Config::default().with_baudrate(115200);

        Uart::new(peripherals.UART1, config)
            .unwrap()
            .with_rx(rx_pin)
            .with_tx(tx_pin)
    };

    let (_, dbg_tx) = dbg_serial.split();
    init_logs(dbg_tx);

    // BLE initialization
    info!("BLE init");

    let (ble_tx_reader, ble_tx_writer) = unsafe { BLE_TX_PIPE.split() };
    let (ble_rx_reader, ble_rx_writer) = unsafe { BLE_RX_PIPE.split() };

    let mut ble_serial = BleSerial::new(ble_tx_writer, ble_rx_reader);

    spawner
        .spawn(ble_task(
            peripherals.TIMG0,
            peripherals.RNG,
            peripherals.RADIO_CLK,
            peripherals.BT,
            ble_tx_reader,
            ble_rx_writer,
        ))
        .unwrap();

    // loop {
    //     let mut buffer = [0; L2CAP_MTU];
    //     let size = ble_serial.read(&mut buffer).await.unwrap();
    //     ble_serial.write_all(&buffer[0..size]).await;
    // }

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
    #[cfg(feature = "esp32c3")]
    let (rx_pin, tx_pin) = (peripherals.GPIO0, peripherals.GPIO1);

    #[cfg(not(feature = "esp32c3"))]
    let (rx_pin, tx_pin) = (peripherals.GPIO3, peripherals.GPIO4);

    let twai_can = CanWrapper::new(peripherals.TWAI0, rx_pin, tx_pin);

    // Create the Bsp
    info!("BSP creation");
    let bsp = Bsp::new(twai_can, serial);

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

core_create_tasks!(SerialMux<BleSerial, UartType>, CanWrapper<'static>);
