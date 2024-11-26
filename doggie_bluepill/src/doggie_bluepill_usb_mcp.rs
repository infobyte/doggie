#![no_std]
#![no_main]

mod bluepill;
mod soft_timer;
mod spi;
mod spi_device;
mod usb_device;

use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;
use usb_device::UsbWrapper;

use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};

use core::cell::RefCell;
use defmt::info;
use mcp2515::MCP2515;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    mode, peripherals,
    peripherals::USB,
    usb,
    usb::Driver,
};
use embassy_time::Timer;
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    Builder, UsbDevice,
};

static mut STATE: Option<RefCell<State>> = None;

bind_interrupts!(struct UsbIrqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<peripherals::USB>;
});

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
    let mut p = bluepill::init();

    let led = Output::new(p.PC13, Level::High, Speed::Low);

    spawner.spawn(blink_task(led)).unwrap();

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
        let config = embassy_usb::Config::new(0xc0de, 0xcafe);
        //config.max_packet_size_0 = 64;

        // Create embassy-usb DeviceBuilder using the driver and config.
        // It needs some buffers for building the descriptors.
        static mut USB_CONFIG_DESC: &mut [u8; 256] = &mut [0; 256];
        static mut USB_BOS_DESC: &mut [u8; 256] = &mut [0; 256];
        static mut USB_CTRL_BUF: &mut [u8; 7] = &mut [0; 7];

        // let mut state: State = State::new();
        unsafe {
            STATE.replace(RefCell::new(State::new()));
        }

        let mut builder = unsafe {
            Builder::new(
                driver,
                config,
                USB_CONFIG_DESC,
                USB_BOS_DESC,
                &mut [], // no msos descriptors
                USB_CTRL_BUF,
            )
        };

        // Create classes on the builder.
        let mut class =
            unsafe { CdcAcmClass::new(&mut builder, STATE.as_mut().unwrap().get_mut(), 64) };

        // Build the builder.
        let usb = builder.build();

        // Run the USB device.
        spawner.spawn(usb_task(usb)).unwrap();

        class.wait_connection().await;

        UsbWrapper::new(class)
    };

    // Delay for the MCP2515
    let delay = SoftTimer {};

    // Setup SPI
    let spi = create_default_spi!(p);

    let bsp = Bsp::new_with_mcp2515(spi, delay, serial);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    info!("About to run CORE");
    core_run!(core);
}

type SerialType = UsbWrapper<'static>;
type CanType = MCP2515<CustomSpiDevice<'static, mode::Blocking>>;

core_create_tasks!(SerialType, CanType);
