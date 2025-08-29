#![no_std]
#![no_main]

#[cfg(all(feature = "twai", feature = "mcp"))]
core::compile_error!(
    "Features compatibility error: 'twai' and 'mcp' features can't be enabled at the same time."
);

#[cfg(all(not(feature = "twai"), not(feature = "mcp")))]
core::compile_error!(
    "Features error: You must enable one CAN interface feature ('twai' or 'mcp')."
);

#[cfg(feature = "twai")]
mod twai_can;

#[cfg(feature = "mcp")]
mod soft_timer;
#[cfg(feature = "mcp")]
mod spi_device;

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, gpio::Level, gpio::Output, uart::Uart, Async};
use esp_serial;

use defmt::info;
use doggie_core::*;

#[cfg(feature = "twai")]
use twai_can::CanWrapper;

#[cfg(feature = "mcp")]
use {
    ::mcp2515::MCP2515 as MCP,
    esp_hal::{
        spi::{
            master::{Config as SpiConfig, Spi},
            Mode as SpiMode,
        },
        time::RateExtU32,
        Blocking,
    },
    soft_timer::SoftTimer,
    spi_device::CustomSpiDevice,
};

#[cfg(feature = "esp32c3")]
#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}

esp_serial::init_globals!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // info!("Device initialization started");
    // Board initialization
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    // Setup embassy timer
    cfg_if::cfg_if! {
     if #[cfg(feature = "esp32")] {
            let timg1 = TimerGroup::new(peripherals.TIMG1);
            esp_hal_embassy::init(timg1.timer0);
        } else {
            use esp_hal::timer::systimer::SystemTimer;

            let systimer = SystemTimer::new(peripherals.SYSTIMER);
            esp_hal_embassy::init(systimer.alarm0);
        }
    }

    // Blink initialization
    #[cfg(feature = "esp32c3")]
    {
        use esp_hal::gpio::Level;
        let led = Output::new(peripherals.GPIO8, Level::Low);
        spawner.spawn(blink_task(led)).unwrap();
    }

    // EvilDoggie unshort bus
    let evil_pin = Output::new(peripherals.GPIO27, Level::High);

    // Serial logging initialization
    esp_serial::init_dbg!(peripherals);

    let serial = esp_serial::create_serial!(peripherals, spawner);

    info!("Serial init ok");

    #[cfg(feature = "faraday")]
    {
        // Eye LEDs
        let (l_r_p, l_g_p, l_b_p) = (peripherals.GPIO5, peripherals.GPIO33, peripherals.GPIO4);
        let (r_r_p, r_g_p, r_b_p) = (peripherals.GPIO19, peripherals.GPIO32, peripherals.GPIO18);

        let mut l_r = Output::new(l_r_p, Level::High);
        let mut l_g = Output::new(l_g_p, Level::High);
        let mut l_b = Output::new(l_b_p, Level::High);
        let mut r_r = Output::new(r_r_p, Level::High);
        let mut r_g = Output::new(r_g_p, Level::High);
        let mut r_b = Output::new(r_b_p, Level::High);

        l_b.set_low();
        r_b.set_low();
    }

    // Create the Bsp
    info!("BSP creation");

    #[cfg(feature = "twai")]
    let bsp = {
        info!("CAN Bus init");
        let can_device = CanWrapper::new();

        Bsp::new(can_device, serial)
    };

    #[cfg(feature = "mcp")]
    let bsp = {
        // Setup SPI
        #[cfg(feature = "esp32c3")]
        let (sclk, mosi, miso, cs, spi) = (
            peripherals.GPIO9,
            peripherals.GPIO6,
            peripherals.GPIO5,
            peripherals.GPIO7,
            peripherals.SPI2,
        );
        #[cfg(feature = "esp32")]
        let (sclk, mosi, miso, cs, spi) = (
            peripherals.GPIO14,
            peripherals.GPIO13,
            peripherals.GPIO12,
            peripherals.GPIO15,
            peripherals.SPI2,
        );

        let spi_config = SpiConfig::default()
            .with_frequency(1_u32.MHz())
            .with_mode(SpiMode::_0);

        let esp_spi = Spi::new(spi, spi_config)
            .unwrap()
            .with_sck(sclk)
            .with_mosi(mosi)
            .with_miso(miso)
            .with_cs(cs);

        let spi = CustomSpiDevice::new(esp_spi);

        // Create SoftTimer
        let delay = SoftTimer {};

        // Create the Bsp
        info!("BSP creation");

        Bsp::new_with_mcp2515(spi, delay, serial)
    };

    // Create and run the Doggie core
    info!("Core creation");
    let core = Core::new(spawner, bsp);

    info!("About to run core...");
    core_run!(core);
}

type SerialType = esp_serial::serial_type!();

#[cfg(all(feature = "twai", feature = "esp32c3"))]
type CanType = CanWrapper<'static>;

#[cfg(all(feature = "twai", not(feature = "esp32c3")))]
type CanType = CanWrapper<'static>;

#[cfg(all(feature = "mcp", not(feature = "twai")))]
type CanType = MCP<CustomSpiDevice<'static, Blocking>>;

core_create_tasks!(SerialType, CanType);
