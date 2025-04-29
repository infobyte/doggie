#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;
mod twai_can;

use esp_backtrace as _;
use esp_println as _;

use doggie_core::*;
use embassy_executor;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::usb_serial_jtag::UsbSerialJtag;
use esp_hal::{
    gpio::{Level, Output},
    timer::timg::TimerGroup,
    Async,
};
use twai_can::CanWrapper;

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
    let p = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let led = Output::new(p.GPIO8, Level::Low);
    spawner.spawn(blink_task(led)).unwrap();

    #[cfg(feature = "esp32c3")]
    let serial = UsbSerialJtag::new(p.USB_DEVICE).into_async();

    // Setup UART (using these pins, also passes through USB)
    #[cfg(not(feature = "esp32c3"))]
    let serial = {
        let (tx_pin, rx_pin) = (p.GPIO1, p.GPIO3);
        let config = esp_hal::uart::Config::default().rx_fifo_full_threshold(READ_BUF_SIZE as u16);

        Uart::new_with_config(p.UART0, config, rx_pin, tx_pin)
            .unwrap()
            .into_async()
    };

    #[cfg(feature = "esp32c3")]
    let (rx_pin, tx_pin) = (p.GPIO0, p.GPIO1);

    #[cfg(not(feature = "esp32c3"))]
    let (rx_pin, tx_pin) = (p.GPIO3, p.GPIO4);

    // Create the Bsp
    let bsp = Bsp::new(CanWrapper::new(p.TWAI0, rx_pin, tx_pin), serial);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

#[cfg(feature = "esp32c3")]
type UartType = UsbSerialJtag<'static, Async>;

#[cfg(not(feature = "esp32c3"))]
type UartType = Uart<'static, Async>;

core_create_tasks!(UartType, CanWrapper<'static>);
