#![no_std]
#![no_main]

pub mod board;
pub mod uart;
pub mod uart_device;
pub mod usb_device;

#[macro_export]
macro_rules! create_default_uart {
    ($p:expr) => {{
        bluepill::uart::create_uart($p.USART2, $p.PA2, $p.PA3, $p.DMA1_CH7, $p.DMA1_CH6)
    }};
}

#[macro_export]
macro_rules! init_globals {
    () => {
        #[cfg(feature = "uart")]
        use bluepill::uart_device::UartWrapper;

        #[cfg(feature = "usb")]
        use {
            embassy_stm32::{peripherals::USB, usb, usb::Driver},
            embassy_usb::{
                class::cdc_acm::{CdcAcmClass, State},
                UsbDevice,
            },
            static_cell::StaticCell,
            bluepill::usb_device::UsbWrapper,
        };

        #[cfg(feature = "usb")]
        bind_interrupts!(struct UsbIrqs {
            USB_LP_CAN1_RX0 => usb::InterruptHandler<USB>;
        });

        #[cfg(feature = "usb")]
        #[embassy_executor::task]
        async fn usb_task(mut usb: UsbDevice<'static, embassy_stm32::usb::Driver<'static, USB>>) {
            usb.run().await;
        }

        #[cfg(feature = "uart")]
        type InnerSerialType = UartWrapper<'static>;
        #[cfg(feature = "usb")]
        type InnerSerialType = UsbWrapper<'static>;
    }
}

#[macro_export]
macro_rules! create_serial {
    ($p:expr, $s:expr) => {{
        #[cfg(feature = "uart")]
        let uart_serial = bluepill::create_default_uart!($p);

        let mut pa12 = $p.PA12;

        #[cfg(feature = "usb")]
        let usb_serial = {
            {
                // BluePill board has a pull-up resistor on the D+ line.
                // Pull the D+ pin down to send a RESET condition to the USB bus.
                // This forced reset is needed only for development, without it host
                // will not reset your device when you upload new firmware.
                let _dp = Output::new(pa12, Level::Low, Speed::Low);
                Timer::after_millis(10).await;
                drop(_dp);
                pa12 = unsafe { embassy_stm32::peripherals::PA12::steal() };
            }

            // Create the driver, from the HAL.
            let driver = Driver::new($p.USB, UsbIrqs, pa12, $p.PA11);

            // Create embassy-usb Config
            let config = {
                let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
                config.manufacturer = Some("Aznarez/Gianatiempo");
                config.product = Some("DoggieBluepill");
                config.serial_number = Some("1337");
                config.max_power = 100;
                config.max_packet_size_0 = 64;
                config.device_class = 0xEF;
                config.device_sub_class = 0x02;
                config.device_protocol = 0x01;
                config.composite_with_iads = true;
                config
            };

            // Create embassy-usb DeviceBuilder using the driver and config.
            // It needs some buffers for building the descriptors.
            let mut builder = {
                static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
                static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
                static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

                let builder = embassy_usb::Builder::new(
                    driver,
                    config,
                    CONFIG_DESCRIPTOR.init([0; 256]),
                    BOS_DESCRIPTOR.init([0; 256]),
                    &mut [], // no msos descriptors
                    CONTROL_BUF.init([0; 64]),
                );
                builder
            };

            // Create classes on the builder.
            let mut class = {
                static STATE: StaticCell<State> = StaticCell::new();
                let state = STATE.init(State::new());
                CdcAcmClass::new(&mut builder, state, 64)
            };

            info!("Building USB");
            // Build the builder.
            let usb = builder.build();

            // Run the USB device.
            $s.spawn(usb_task(usb)).unwrap();

            info!("Waiting for USB connection");
            class.wait_connection().await;

            UsbWrapper::new(class)
        };

        #[cfg(feature = "uart")]
        let serial = uart_serial;

        #[cfg(feature = "usb")]
        let serial = usb_serial;

        serial
    }};
}

#[macro_export]
macro_rules! serial_type {
    () => {
        InnerSerialType
    };
}
