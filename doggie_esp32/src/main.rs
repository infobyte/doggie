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

mod logging;

#[cfg(feature = "twai")]
mod twai_can;

#[cfg(feature = "mcp")]
mod soft_timer;
#[cfg(feature = "mcp")]
mod spi_device;

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
};

use defmt::info;
use doggie_core::*;

#[cfg(any(feature = "esp32", feature = "ble"))]
use esp_hal::timer::timg::TimerGroup;

#[cfg(not(feature = "esp32"))]
use esp_hal::usb_serial_jtag::UsbSerialJtag;

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

#[cfg(feature = "ble")]
use {
    bt_hci::controller::ExternalController,
    defmt::error,
    doggie_ble::{create_ble_pipe, BleSerial, BleServer, SerialMux},
    esp_wifi::ble::controller::BleConnector,
    esp_wifi::EspWifiController,
    static_cell::StaticCell,
};

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[cfg(feature = "ble")]
#[embassy_executor::task]
async fn ble_task(
    mut ble_server: BleServer,
    controller: ExternalController<BleConnector<'static>, 20>,
) {
    info!("[BLE] About to run BLE server");
    ble_server
        .run::<ExternalController<BleConnector<'_>, 20>>(controller)
        .await;
    error!("[BLE] Ble task exited");
}

#[cfg(feature = "ble")]
static BLE_CONT_AUX: StaticCell<EspWifiController<'static>> = StaticCell::new();

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

    #[cfg(feature = "esp32c3")]
    let (dbg_tx_pin, dbg_rx_pin) = (peripherals.GPIO3, peripherals.GPIO2);
    #[cfg(feature = "esp32")]
    let (dbg_tx_pin, dbg_rx_pin) = (peripherals.GPIO10, peripherals.GPIO9);

    let dbg_serial = {
        let config = esp_hal::uart::Config::default().with_baudrate(115200);

        Uart::new(peripherals.UART1, config)
            .unwrap()
            .with_rx(dbg_rx_pin)
            .with_tx(dbg_tx_pin)
    };

    let (_, dbg_tx) = dbg_serial.split();
    init_logs(dbg_tx);

    info!("Wired serial init");
    // Wired serial initialization
    #[cfg(feature = "esp32c3")]
    let wired_serial = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();

    // Setup UART (using these pins, also passes through USB)
    #[cfg(not(feature = "esp32c3"))]
    let wired_serial = {
        let (tx_pin, rx_pin) = (peripherals.GPIO1, peripherals.GPIO3);
        let config = esp_hal::uart::Config::default().with_rx_fifo_full_threshold(256);

        Uart::new(peripherals.UART0, config)
            .unwrap()
            .with_rx(rx_pin)
            .with_tx(tx_pin)
            .into_async()
    };

    // BLE initialization
    #[cfg(feature = "ble")]
    let ble_serial = {
        info!("BLE init");

        let timg0 = TimerGroup::new(peripherals.TIMG0);

        let init = BLE_CONT_AUX.init(
            esp_wifi::init(
                timg0.timer0,
                esp_hal::rng::Rng::new(peripherals.RNG),
                peripherals.RADIO_CLK,
            )
            .unwrap(),
        );

        let connector = BleConnector::new(init, peripherals.BT);
        let controller: ExternalController<BleConnector<'static>, 20> =
            ExternalController::new(connector);

        let (ble_server, ble_serial) = create_ble_pipe();

        spawner.spawn(ble_task(ble_server, controller)).unwrap();

        SerialMux::new(ble_serial, wired_serial)
    };

    // Create the Bsp
    info!("BSP creation");

    #[cfg(feature = "twai")]
    let bsp = {
        info!("CAN Bus init");
        #[cfg(feature = "esp32c3")]
        let (rx_pin, tx_pin) = (peripherals.GPIO0, peripherals.GPIO1);

        #[cfg(not(feature = "esp32c3"))]
        let (rx_pin, tx_pin) = (peripherals.GPIO25, peripherals.GPIO26);

        let can_device = CanWrapper::new(peripherals.TWAI0, rx_pin, tx_pin);

        #[cfg(feature = "ble")]
        let bsp = Bsp::new(can_device, ble_serial);

        #[cfg(not(feature = "ble"))]
        let bsp = Bsp::new(can_device, wired_serial);

        bsp
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

        #[cfg(feature = "ble")]
        let bsp = Bsp::new_with_mcp2515(spi, delay, ble_serial);

        #[cfg(not(feature = "ble"))]
        let bsp = Bsp::new_with_mcp2515(spi, delay, wired_serial);

        bsp
    };

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

#[cfg(feature = "ble")]
type SerialType = SerialMux<BleSerial, UartType>;

#[cfg(not(feature = "ble"))]
type SerialType = UartType;

#[cfg(feature = "twai")]
type CanType = CanWrapper<'static>;

#[cfg(all(feature = "mcp", not(feature = "twai")))]
type CanType = MCP<CustomSpiDevice<'static, Blocking>>;

core_create_tasks!(SerialType, CanType);
