#![no_std]
#![no_main]

mod ble;
mod twai_can;

use twai_can::CanWrapper;
use ble::{BleSerial, BleServer, PIPE_CAPACITY};

use doggie_core::*;

use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pipe::Pipe;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output},
};
use esp_println::println;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use embedded_io_async::{Read, Write};

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
    esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    // let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    // let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(72 * 1024);

    // cfg_if::cfg_if! {
    //     if #[cfg(feature = "esp32")] {
    //         let timg1 = TimerGroup::new(peripherals.TIMG1);
    //         esp_hal_embassy::init(timg1.timer0);
    //     } else {
    //         use esp_hal::timer::systimer::SystemTimer;
    //         let systimer = SystemTimer::new(peripherals.SYSTIMER);
    //         esp_hal_embassy::init(systimer.alarm0);
    //     }
    // }
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

    
    let led = Output::new(peripherals.GPIO8, Level::Low);
    spawner.spawn(blink_task(led)).unwrap();

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

//     let mut buffer: [u8;128] = [0;128];
    
//     loop {
//         let len = ble_serial.read(&mut buffer).await.unwrap();
//         println!("{:?}", &buffer[0..len]);

//         ble_serial.write(&buffer[0..len]).await.unwrap();
//     }
// }


    #[cfg(feature = "esp32c3")]
    let (rx_pin, tx_pin) = (peripherals.GPIO0, peripherals.GPIO1);

    #[cfg(not(feature = "esp32c3"))]
    let (rx_pin, tx_pin) = (peripherals.GPIO3, peripherals.GPIO4);

    // Create the Bsp
    let bsp = Bsp::new(CanWrapper::new(peripherals.TWAI0, rx_pin, tx_pin), ble_serial);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}


core_create_tasks!(BleSerial, CanWrapper<'static>);

