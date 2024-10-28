#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;

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
    peripherals, spi,
    spi::MODE_0,
    time::Hertz,
    usart,
    usart::BufferedUart,
    Config as StmConfig,
};
use embassy_stm32::{mode, rcc::*};
use mcp2515::MCP2515;
use {defmt_rtt as _, panic_probe as _};

static mut UART2_BUF_TX: &mut [u8; 64] = &mut [0; 64];
static mut UART2_BUF_RX: &mut [u8; 64] = &mut [0; 64];

bind_interrupts!(struct UartIrqs {
    USART2 => usart::BufferedInterruptHandler<peripherals::USART2>;
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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = init_bluepill();

    // let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    // Setup SPI
    let mut spi_config = spi::Config::default();
    spi_config.mode = MODE_0;
    spi_config.frequency = Hertz(10_000_000);
    spi_config.miso_pull = Pull::Down;

    let stm_spi = spi::Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);

    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);

    let spi = CustomSpiDevice::new(stm_spi, cs);

    // Delay for the MCP2515
    let delay = SoftTimer {};

    // Init UART
    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 921600;

    // Initialize UART
    let uart = unsafe {
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
    };

    // Create the Bsp
    let bsp = Bsp::new_with_mcp2515(spi, delay, uart);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    core_run!(core);
}

core_create_tasks!(
    BufferedUart<'static>,
    MCP2515<CustomSpiDevice<'static, mode::Blocking>>
);
