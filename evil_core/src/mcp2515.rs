use crate::{can::{CanBitrates, CanDevice}, clock::TicksClock, tranceiver::Tranceiver};
use defmt::{error, info};
use embedded_can::Id;
use embedded_hal::{delay::DelayNs, spi::SpiDevice, digital::{InputPin, OutputPin}};

use crate::bsp::EvilBsp;

use mcp2515::{
    filter::{RxFilter, RxMask},
    regs::OpMode,
    CanSpeed, McpSpeed, MCP2515,
};

// TODO: The next 3 options should be features
const MCP_CLOCK: McpSpeed = McpSpeed::MHz8;
const MCP_CLOCK_ENABLE: bool = false;
const MCP_INITIAL_BAUDRATE: CanSpeed = CanSpeed::Kbps100;

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
        info!("Setting bitrate to {} Kbps", bitrate as u16);
        match self.set_mode(OpMode::Configuration) {
            Ok(_) => info!("Switching to Configuration Mode"),
            Err(_) => error!("Failed to switch to Configuration Mode"),
        }

        match self.set_bitrate(convert_bitrate(bitrate), MCP_CLOCK, MCP_CLOCK_ENABLE) {
            Ok(_) => info!("Bitrate set!"),
            Err(_) => error!("Failed to set bitrate!!!"),
        };
        match self.set_mode(OpMode::Normal) {
            Ok(_) => info!("Switching to Normal Mode"),
            Err(_) => error!("Failed to switch to Normal Mode"),
        }
    }

    fn set_filter(&mut self, id: Id) {
        self.set_filter(RxFilter::F0, id).unwrap();
    }

    fn set_mask(&mut self, id: Id) {
        self.set_mask(RxMask::Mask0, id).unwrap();
    }
}

impl<Clock, Tr> EvilBsp<Clock, Tr>
where
    Clock: TicksClock,
    Tr: Tranceiver,
{
    pub fn new_with_mcp2515<DELAY: DelayNs, SPI: SpiDevice>(
        spi: SPI, mut delay: DELAY, clock: Clock, tr: Tr,
    ) -> Self {
        let mut can = MCP2515::new(spi);

        can.init(
            &mut delay,
            mcp2515::Settings {
                mode: OpMode::Configuration,
                can_speed: MCP_INITIAL_BAUDRATE,
                mcp_speed: MCP_CLOCK,
                clkout_en: MCP_CLOCK_ENABLE,
            },
        )
        .unwrap();

        let mut canctrl: mcp2515::regs::CanCtrl = can.read_register().unwrap();
        canctrl.set_clken(false);
        can.write_register(canctrl).unwrap();

        let mut cnf3: mcp2515::regs::Cnf3 = can.read_register().unwrap();
        cnf3.set_sof(true);
        can.write_register(cnf3).unwrap();

        can.set_mode(OpMode::ListenOnly).unwrap();

        EvilBsp::new(clock, tr)
    }
}
