#![no_std]
#![no_main]

mod can_device;
mod soft_timer;
mod spi_device;
mod usb_device;

use can_device::CanWrapper;
use soft_timer::SoftTimer;
use spi_device::CustomSpiDevice;

use defmt::{error, info};
use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Pull, Speed},
    peripherals::{self, USB},
    spi::{self, MODE_0},
    time::Hertz,
    usart::{self, BufferedUart},
    Config as StmConfig,
};
use embassy_stm32::{mode, rcc::*};
use mcp2515::MCP2515;
use static_cell::StaticCell;
use usb_device::UsbWrapper;
use {defmt_rtt as _, panic_probe as _};

use embassy_futures::join::join;
use embassy_stm32::can::frame::Envelope;
use embassy_stm32::can::{
    filter, Can, Fifo, Frame, Id, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
    StandardId, TxInterruptHandler,
};
use embassy_stm32::peripherals::CAN;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{usb, Config};
use embassy_time::Timer;
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    UsbDevice,
};

use core::cell::RefCell;

#[cfg(feature = "usb")]
static mut STATE: Option<RefCell<State>> = None;

#[cfg(feature = "usb")]
bind_interrupts!(struct UsbIrqs {
    USB_LP_CAN1_RX0 => usb::InterruptHandler<peripherals::USB>;
});

#[cfg(feature = "uart")]
static mut UART2_BUF_TX: &mut [u8; 64] = &mut [0; 64];
#[cfg(feature = "uart")]
static mut UART2_BUF_RX: &mut [u8; 128] = &mut [0; 128];

#[cfg(feature = "uart")]
bind_interrupts!(struct UartIrqs {
    USART2 => usart::BufferedInterruptHandler<peripherals::USART2>;
});

#[cfg(feature = "internal_can")]
bind_interrupts!(struct CanIrqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

#[cfg(feature = "interenal_can")]
bind_interrupts!(struct CanIrqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

fn init_bluepill() -> embassy_stm32::Peripherals {
    let mut config = StmConfig::default();

    // Clock configuration to run at 72MHz (Max)
    {
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            // Oscillator for bluepill, Bypass for nucleos.
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }

    embassy_stm32::init(config)
}

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
    if !cfg!(feature = "uart") && !cfg!(feature = "usb") {
        panic!("Either 'uart' or 'usb' feature must be enabled.");
    }

    if cfg!(feature = "usb") && cfg!(feature = "internal_can") {
        panic!("Usb and internal_can features are incompatible.");
    }

    let mut p = init_bluepill();

    let led = Output::new(p.PC13, Level::High, Speed::Low);

    spawner.spawn(blink_task(led)).unwrap();

    #[cfg(feature = "uart")]
    let serial = {
        let mut uart_config = usart::Config::default();
        uart_config.baudrate = 115200;

        // Initialize UART
        unsafe {
            BufferedUart::new(
                p.USART2,
                UartIrqs,
                p.PA3,
                p.PA2,
                UART2_BUF_TX,
                UART2_BUF_RX,
                uart_config,
            )
            .unwrap()
        }
    };

    #[cfg(feature = "usb")]
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

    #[cfg(feature = "internal_can")]
    {
        // Set alternate pin mapping to B8/B9
        embassy_stm32::pac::AFIO
            .mapr()
            .modify(|w| w.set_can1_remap(2));

        static RX_BUF: StaticCell<embassy_stm32::can::RxBuf<10>> = StaticCell::new();
        static TX_BUF: StaticCell<embassy_stm32::can::TxBuf<10>> = StaticCell::new();

        let mut can = Can::new(p.CAN, p.PB8, p.PB9, CanIrqs);

        can.modify_filters()
            .enable_bank(0, Fifo::Fifo0, filter::Mask32::accept_all());

        can.modify_config()
            .set_loopback(false)
            .set_silent(false)
            .set_bitrate(250_000);

        can.enable().await;

        let can_wrapper = CanWrapper::new(can);

        let bsp = Bsp::new(can_wrapper, serial);

        // Create and run the Doggie core
        let core = Core::new(spawner, bsp);

        info!("About to run CORE");
        core_run!(core);
    }

    #[cfg(feature = "mcp2515")]
    {
        // Delay for the MCP2515
        let delay = SoftTimer {};

        // Setup SPI
        let mut spi_config = spi::Config::default();
        spi_config.mode = MODE_0;
        spi_config.frequency = Hertz(1_000_000);
        spi_config.miso_pull = Pull::Down;

        let stm_spi = spi::Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);

        let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);

        let spi = CustomSpiDevice::new(stm_spi, cs);

        let bsp = Bsp::new_with_mcp2515(spi, delay, serial);

        // Create and run the Doggie core
        let core = Core::new(spawner, bsp);

        info!("About to run CORE");
        core_run!(core);
    }
}

#[cfg(feature = "uart")]
type SerialType = BufferedUart<'static>;

#[cfg(feature = "usb")]
type SerialType = UsbWrapper<'static>;

#[cfg(feature = "mcp2515")]
type CanType = MCP2515<CustomSpiDevice<'static, mode::Blocking>>;

#[cfg(feature = "internal_can")]
type CanType = CanWrapper<'static>;

core_create_tasks!(SerialType, CanType);
