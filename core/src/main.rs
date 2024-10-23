#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;

use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::mode::Mode;
use embassy_stm32::spi;
use embassy_stm32::spi::{Error as SpiError, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config as StmConfig;
use embassy_stm32::{
    gpio::{Level, Output, Pull, Speed},
    spi::MODE_0,
};

use embedded_can;

use mcp2515;

use embedded_hal::{
    delay::DelayNs,
    spi::{ErrorType, Operation, SpiDevice as SpiDeviceTrait},
};

pub struct CustomSpiDevice<'d, MODE: Mode> {
    spi: Spi<'d, MODE>,
    cs: Output<'d>,
}

impl<'d, MODE: Mode> CustomSpiDevice<'d, MODE> {
    pub fn new(spi: Spi<'d, MODE>, cs: Output<'d>) -> Self {
        CustomSpiDevice { spi, cs }
    }
}

impl<'d, MODE: Mode> ErrorType for CustomSpiDevice<'d, MODE> {
    type Error = SpiError;
}

impl<'d, MODE: Mode> SpiDeviceTrait<u8> for CustomSpiDevice<'d, MODE> {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        self.cs.set_low();

        for operation in operations {
            match operation {
                Operation::Read(buffer) => {
                    self.spi.blocking_read(buffer)?;
                }
                Operation::Write(data) => {
                    self.spi.blocking_write(data)?;
                }
                Operation::Transfer(read_buffer, write_data) => {
                    self.spi.blocking_transfer(read_buffer, write_data)?;
                }
                Operation::TransferInPlace(buffer) => {
                    self.spi.blocking_transfer_in_place(buffer)?;
                }
                Operation::DelayNs(ns) => {
                    // Implement delay here. This depends on your specific setup.
                    // For example, you might use a timer or a busy-wait loop.
                    // Here's a placeholder implementation:
                    // Timer::after_nanos(ns).await();
                    cortex_m::asm::delay(*ns / 10); // Assuming 100MHz clock, adjust as needed
                }
            }
        }

        self.cs.set_high();

        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        let ret = self.spi.blocking_read(buf);
        self.cs.set_high();

        ret
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        let ret = self.spi.blocking_write(buf);
        self.cs.set_high();

        ret
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        let ret = self.spi.blocking_transfer(read, write);
        self.cs.set_high();

        ret
    }

    fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        let ret = self.spi.blocking_transfer_in_place(buf);
        self.cs.set_high();

        ret
    }
}

struct SoftTimer {}

impl DelayNs for SoftTimer {
    // Required method
    fn delay_ns(&mut self, ns: u32) {
        // Aprox
        cortex_m::asm::delay(ns / 20);
    }
}

static DATA: [u8; 5] = [1, 2, 3, 4, 5];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = StmConfig::default();

    // Clock configuration to run at 72MHz (Max)
    {
        use embassy_stm32::rcc::*;
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

    let p = embassy_stm32::init(config);

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    // Setup SPI
    let mut spi_config = spi::Config::default();
    spi_config.mode = MODE_0;
    spi_config.frequency = Hertz(1_000_000);
    spi_config.miso_pull = Pull::Down;

    let mut spi = spi::Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, spi_config);

    let mut cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);

    cs.set_high();

    Timer::after_millis(5000).await;

    cs.set_low();
    SoftTimer {}.delay_ms(1);
    cs.set_high();

    info!("HAL SPI initialized");

    let mut custom_spi = CustomSpiDevice { spi, cs };

    let mut can = MCP2515::new(custom_spi);

    let mut delay = SoftTimer {};

    use embedded_can::{ExtendedId, Frame, Id};
    use mcp2515::{
        error::Error, frame::CanFrame, regs::OpMode, CanSpeed, McpSpeed, Settings, MCP2515,
    };

    match can.init(
        &mut delay,
        mcp2515::Settings {
            mode: OpMode::Normal,         // Loopback for testing and example
            can_speed: CanSpeed::Kbps250, // Many options supported.
            mcp_speed: McpSpeed::MHz8,    // Currently 16MHz and 8MHz chips are supported.
            clkout_en: false,
        },
    ) {
        Ok(_) => info!("Init success"),
        Err(_) => error!("Init Failed"),
    }

    let mut id: u16 = 0x1FF;

    // let &'static data = [0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe];

    loop {
        // Send a message

        // let mut frame = CanFrame::new(Id::Standard(StandardId::new(id).unwrap()), &DATA).unwrap();
        // let frame = CanFrame::new(Id::Standard(StandardId::new(id).unwrap()), &DATA).unwrap();
        // let frame = CanFrame::new(
        //     Id::Standard(StandardId::new(id).unwrap()),
        //     &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        // )
        // .unwrap();
        //
        // match can.send_message(frame) {
        //     Ok(_) => info!("Sent message!"),
        //     Err(spi_error) => error!("Error sending message"),
        // }
        //
        // Read the message back (we are in loopback mode)
        match can.read_message() {
            Ok(frame) => {
                info!("Frame received");

                let id = frame.id(); // CAN ID
                                     //
                let id_value = match id {
                    embedded_can::Id::Standard(id) => id.as_raw() as u32, // Get raw standard CAN ID
                    embedded_can::Id::Extended(id) => id.as_raw(),        // Get raw extended CAN ID
                };

                let dlc = frame.dlc(); // Data length code
                let data = frame.data(); // Data bytes
                let is_extended = frame.is_extended(); // Check if it's an extended frame

                // Print the CAN frame details
                println!(
                    "ID: {:x}, DLC: {}, Data: {:X}, Extended: {}",
                    id_value, dlc, data, is_extended
                );

                Timer::after_millis(1000).await;
            }
            Err(Error::NoMessage) => info!("No message to read!"),
            Err(_) => error!("Oh no!"),
        }

        Timer::after_millis(1000).await;
    }
}
