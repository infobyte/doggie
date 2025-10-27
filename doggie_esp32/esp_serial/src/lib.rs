#![no_std]
#![no_main]

pub mod logging;

#[macro_export]
macro_rules! init_globals {
    () => {
        #[cfg(any(feature = "esp32", feature = "ble"))]
        use esp_hal::timer::timg::TimerGroup;

        #[cfg(not(feature = "esp32"))]
        use esp_hal::usb_serial_jtag::UsbSerialJtag;

        #[cfg(feature = "ble")]
        use {
            bt_hci::controller::ExternalController,
            defmt::error,
            doggie_ble::{create_ble_pipe, BleSerial, BleServer, SerialMux},
            esp_alloc as _,
            esp_wifi::ble::controller::BleConnector,
            esp_wifi::EspWifiController,
            static_cell::StaticCell,
        };

        #[cfg(feature = "esp32c3")]
        type UartType = UsbSerialJtag<'static, Async>;

        #[cfg(not(feature = "esp32c3"))]
        type UartType = Uart<'static, Async>;

        #[cfg(feature = "ble")]
        type InnerSerialType = SerialMux<BleSerial, UartType>;

        #[cfg(not(feature = "ble"))]
        type InnerSerialType = UartType;

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
    };
}

#[macro_export]
macro_rules! init_dbg {
    ($p:expr) => {{
        #[cfg(feature = "esp32c3")]
        let (dbg_tx_pin, dbg_rx_pin, dbg_uart) = ($p.GPIO3, $p.GPIO2, $p.UART0);
        #[cfg(feature = "esp32")]
        let (dbg_tx_pin, dbg_rx_pin, dbg_uart) = ($p.GPIO17, $p.GPIO16, $p.UART2);

        let dbg_serial = {
            let config = esp_hal::uart::Config::default().with_baudrate(115200);

            Uart::new(dbg_uart, config)
                .unwrap()
                .with_rx(dbg_rx_pin)
                .with_tx(dbg_tx_pin)
        };

        let (_, dbg_tx) = dbg_serial.split();
        esp_serial::logging::init_logs(dbg_tx);
    }};
}

#[macro_export]
macro_rules! create_wired_serial {
    ($p:expr) => {{
        info!("Wired serial init");
        // Wired serial initialization
        #[cfg(feature = "esp32c3")]
        let wired_serial = UsbSerialJtag::new($p.USB_DEVICE).into_async();

        // Setup UART (using these pins, also passes through USB)
        #[cfg(not(feature = "esp32c3"))]
        let wired_serial = {
            let (tx_pin, rx_pin) = ($p.GPIO1, $p.GPIO3);
            let config = esp_hal::uart::Config::default().with_baudrate(115200);

            Uart::new($p.UART0, config)
                .unwrap()
                .with_rx(rx_pin)
                .with_tx(tx_pin)
                .into_async()
        };

        wired_serial
    }};
}

#[macro_export]
macro_rules! create_serial {
    ($p:expr, $s:expr) => {{
        let wired_serial = esp_serial::create_wired_serial!($p);

        #[cfg(feature = "ble")]
        let ble_serial = {
            // BLE initialization
            info!("BLE init");

            // Heap initialization needed by the BLE
            esp_alloc::heap_allocator!(72 * 1024);

            let timg0 = TimerGroup::new($p.TIMG0);

            let init = BLE_CONT_AUX.init(
                esp_wifi::init(timg0.timer0, esp_hal::rng::Rng::new($p.RNG), $p.RADIO_CLK).unwrap(),
            );

            let connector = BleConnector::new(init, $p.BT);
            let controller: ExternalController<BleConnector<'static>, 20> =
                ExternalController::new(connector);

            let (ble_server, ble_serial) = create_ble_pipe();

            $s.spawn(ble_task(ble_server, controller)).unwrap();

            SerialMux::new(ble_serial, wired_serial)
        };

        #[cfg(feature = "ble")]
        let serial = ble_serial;

        #[cfg(not(feature = "ble"))]
        let serial = wired_serial;

        serial
    }};
}

#[macro_export]
macro_rules! serial_type {
    () => {
        InnerSerialType
    };
}
