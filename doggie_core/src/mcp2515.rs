use crate::can::{CanBitrates, CanDevice};
use embedded_can::Id;
use embedded_hal::{delay::DelayNs, spi::SpiDevice};
use embedded_io_async::{Read, Write};

use crate::bsp::Bsp;

use mcp2515::{
    filter::{RxFilter, RxMask},
    regs::OpMode,
    CanSpeed, McpSpeed, MCP2515,
};

// TODO: The next 3 options should be features
const MCP_CLOCK: McpSpeed = McpSpeed::MHz8;
const MCP_CLOCK_ENABLE: bool = false;
const MCP_INITIAL_BAUDRATE: CanSpeed = CanSpeed::Kbps250;

fn convert_bitrate(from: CanBitrates) -> CanSpeed {
    can_speed_from_raw(from as u16)
}

pub fn can_speed_from_raw(speed: u16) -> CanSpeed {
    match speed {
        5 => CanSpeed::Kbps5,
        10 => CanSpeed::Kbps10,
        20 => CanSpeed::Kbps20,
        3125 => CanSpeed::Kbps31_25,
        333 => CanSpeed::Kbps33_3,
        40 => CanSpeed::Kbps40,
        50 => CanSpeed::Kbps50,
        80 => CanSpeed::Kbps80,
        100 => CanSpeed::Kbps100,
        125 => CanSpeed::Kbps125,
        200 => CanSpeed::Kbps200,
        250 => CanSpeed::Kbps250,
        500 => CanSpeed::Kbps500,
        1000 => CanSpeed::Kbps1000,
        // Default
        _ => CanSpeed::Kbps250,
    }
}

impl<SPI: SpiDevice> CanDevice for MCP2515<SPI> {
    fn set_bitrate(&mut self, bitrate: CanBitrates) {
        self.set_bitrate(convert_bitrate(bitrate), MCP_CLOCK, MCP_CLOCK_ENABLE)
            .unwrap();
    }

    fn set_filter(&mut self, id: Id) {
        self.set_filter(RxFilter::F0, id).unwrap();
    }

    fn set_mask(&mut self, id: Id) {
        self.set_mask(RxMask::Mask0, id).unwrap();
    }
}

impl<SPI, SERIAL> Bsp<MCP2515<SPI>, SERIAL>
where
    SPI: SpiDevice,
    SERIAL: Read + Write,
{
    pub fn new_with_mcp2515<DELAY: DelayNs>(spi: SPI, mut delay: DELAY, serial: SERIAL) -> Self {
        let mut can = MCP2515::new(spi);

        can.init(
            &mut delay,
            mcp2515::Settings {
                mode: OpMode::Normal,
                can_speed: MCP_INITIAL_BAUDRATE,
                mcp_speed: MCP_CLOCK,
                clkout_en: MCP_CLOCK_ENABLE,
            },
        )
        .unwrap();

        Bsp::new(can, serial)
    }
}
