use crate::twai_can::CanWrapper;
use mcp2515::{MCP2515, regs::OpMode, frame::CanFrame as McpCanFrame};
use embedded_hal::{spi::SpiDevice, delay::DelayNs};
use doggie_core::{
    {
        mcp2515::{MCP_CLOCK, MCP_CLOCK_ENABLE, MCP_INITIAL_BAUDRATE},
    },
    CanBitrates, CanDevice
};
use embedded_can::{Frame, Id, blocking::Can, Error, ErrorKind};
use esp_hal::twai::EspTwaiFrame;
use defmt::info;


#[derive(Debug)]
pub struct CanMuxError {}

impl<'d> Error for CanMuxError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

pub struct CanMuxFrame {
    pub id: Id,
    pub data: [u8; 8],
    pub dlc: usize,
    is_remote: bool,
}

impl CanMuxFrame {
    fn from<F: Frame>(frame: F) -> Self {
        let mut mux = Self {
            id: frame.id(),
            data: [0;8],
            dlc: frame.dlc(),
            is_remote: frame.is_remote_frame(),
        };

        let mut index = 0;
        for byte in frame.data() {
            mux.data[index] = *byte;
            index += 1;
        }

        mux
    }

    fn convert<F: Frame>(&self) -> F {
        F::new(self.id, &self.data).unwrap()
    }
}

impl Frame for CanMuxFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        let mut mux = Self {
            id: id.into(),
            dlc: data.len(),
            data: [0;8],
            is_remote: false,
        };
        
        let mut index = 0;
        for byte in data {
            mux.data[index] = *byte;
            index += 1;
        }

        Some(mux)
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        Some(Self {
            id: id.into(),
            data: [0;8],
            dlc,
            is_remote: true,
        })
    }
    
    fn is_extended(&self) -> bool {
        match self.id {
            Id::Standard(_) => false,
            _ => true,
        }   
    }
    
    fn is_remote_frame(&self) -> bool {
        self.is_remote
    }

    fn id(&self) -> Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.dlc
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}


impl<'d, Spi: SpiDevice> Can for CanMux<'d, Spi> {
    type Frame = CanMuxFrame;
    type Error = CanMuxError;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        match self.state {
            CanMuxState::Mcp => {
                let conv_frame: McpCanFrame = frame.convert();
                match self.mcp.transmit(&conv_frame) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(CanMuxError {}),
                }
            },
            CanMuxState::Twai => {
                let conv_frame: EspTwaiFrame = frame.convert();
                match self.twai.transmit(&conv_frame) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(CanMuxError {}),
                }
            },
        }
    }

    fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        match self.state {
            CanMuxState::Mcp => {
                match self.mcp.receive() {
                    Ok(frame) => Ok(CanMuxFrame::from(frame)),
                    Err(_) => Err(CanMuxError {}),
                }
            },
            CanMuxState::Twai => {
                match self.twai.receive() {
                    Ok(frame) => Ok(CanMuxFrame::from(frame)),
                    Err(_) => Err(CanMuxError {}),
                }
            },
        }
    }
}

enum CanMuxState {
    Twai,
    Mcp,
}

pub struct CanMux<'d, Spi: SpiDevice> {
    twai: CanWrapper<'d>,
    mcp: MCP2515<Spi>,
    state: CanMuxState,
}

impl<'d, Spi: SpiDevice> CanMux<'d, Spi> {
    pub fn new<Delay: DelayNs>(twai: CanWrapper<'d>, mut mcp: MCP2515<Spi>, mut delay: Delay) -> Self {
        let state = match mcp.init(
            &mut delay,
            mcp2515::Settings {
                mode: OpMode::Normal,
                can_speed: MCP_INITIAL_BAUDRATE,
                mcp_speed: MCP_CLOCK,
                clkout_en: MCP_CLOCK_ENABLE,
            },
        ) {
            Ok(_) => {
                info!("MCP Can selected");
                CanMuxState::Mcp
            },
            Err(_) => {
                info!("TWAI Can selected");
                CanMuxState::Twai
            },
        };

        Self { twai, mcp, state }
    }
}

impl<'d, Spi: SpiDevice> CanDevice for CanMux<'d, Spi> {
    fn set_bitrate(&mut self, bitrate: CanBitrates) {
        match self.state {
            CanMuxState::Mcp => CanDevice::set_bitrate(&mut self.mcp, bitrate),
            CanMuxState::Twai => CanDevice::set_bitrate(&mut self.twai, bitrate),
        }
    }

    fn set_filter(&mut self, id: Id) {
        match self.state {
            CanMuxState::Mcp => CanDevice::set_filter(&mut self.mcp, id),
            CanMuxState::Twai => CanDevice::set_filter(&mut self.twai, id),
        }
    }

    fn set_mask(&mut self, id: Id) {
        match self.state {
            CanMuxState::Mcp => CanDevice::set_mask(&mut self.mcp, id),
            CanMuxState::Twai => CanDevice::set_mask(&mut self.twai, id),
        }
    }
}
