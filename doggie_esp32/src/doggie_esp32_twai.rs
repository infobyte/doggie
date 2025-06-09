#![no_std]
#![no_main]

mod twai_can;
mod logging;

use twai_can::CanWrapper;
use logging::init_logs;

use embassy_executor::Spawner;
use embassy_time::Timer;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output},
    uart::Uart,
    Async,
    usb_serial_jtag::UsbSerialJtag,
};

use defmt::info;
use doggie_core::*;


#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
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

        Uart::new(peripherals.UART1, config).unwrap().with_rx(rx_pin).with_tx(tx_pin)
    };

    let (_, dbg_tx) = dbg_serial.split();
    init_logs(dbg_tx);

    // BLE initialization
    info!("BLE init");

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

    // let serial = SerialMux::new(ble_serial, wired_serial);
    
    // CAN bus initialization
    info!("CAN Bus init");
    #[cfg(feature = "esp32c3")]
    let (rx_pin, tx_pin) = (peripherals.GPIO0, peripherals.GPIO1);

    #[cfg(not(feature = "esp32c3"))]
    let (rx_pin, tx_pin) = (peripherals.GPIO3, peripherals.GPIO4);

    let twai_can = CanWrapper::new(peripherals.TWAI0, rx_pin, tx_pin);

    // Create the Bsp
    info!("BSP creation");
    let bsp = Bsp::new(twai_can, wired_serial);

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

core_create_tasks!(UartType,  CanWrapper<'static>);
