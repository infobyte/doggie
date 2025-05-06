#![no_std]
#![no_main]

mod ble;
mod twai_can;
mod logging;
mod serial_mux;

use twai_can::CanWrapper;
use ble::{BleSerial, BleServer, PIPE_CAPACITY};
use logging::init_logs;
use serial_mux::SerialMux;

use doggie_core::*;

use core::cell::RefCell;

use defmt::info;
use critical_section::Mutex;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pipe::Pipe;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output},
    uart::{
        Uart, UartTx,
    },
    peripherals,
    Async,
};
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;



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
    let mut dbg_serial = {
        let (tx_pin, rx_pin) = (peripherals.GPIO3, peripherals.GPIO2);
        let mut config = esp_hal::uart::Config::default().baudrate(115200);

        Uart::new_with_config(peripherals.UART1, config, rx_pin, tx_pin)
            .unwrap()
    };

    let (_, dbg_tx) = dbg_serial.split();
    init_logs(dbg_tx);

    // BLE initialization
    let (ble_tx_reader, ble_tx_writer) = unsafe { BLE_TX_PIPE.split() };
    let (ble_rx_reader, ble_rx_writer) = unsafe { BLE_RX_PIPE.split() };

    let mut ble_server = BleServer::new(
        peripherals.BT,
        peripherals.TIMG0,
        peripherals.RNG,
        peripherals.RADIO_CLK,
        ble_tx_reader,
        ble_rx_writer,
    );
    
    spawner.spawn(ble_task(ble_server)).unwrap();

    let mut ble_serial = BleSerial::new(
        ble_tx_writer, ble_rx_reader
    );


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
    #[cfg(feature = "esp32c3")]
    let (rx_pin, tx_pin) = (peripherals.GPIO0, peripherals.GPIO1);

    #[cfg(not(feature = "esp32c3"))]
    let (rx_pin, tx_pin) = (peripherals.GPIO3, peripherals.GPIO4);

    let can = CanWrapper::new(peripherals.TWAI0, rx_pin, tx_pin);
    // Create the Bsp
    let bsp = Bsp::new(can, serial);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

#[cfg(feature = "esp32c3")]
type UartType = UsbSerialJtag<'static, Async>;

#[cfg(not(feature = "esp32c3"))]
type UartType = Uart<'static, Async>;

core_create_tasks!(SerialMux<BleSerial, UartType>, CanWrapper<'static>);
