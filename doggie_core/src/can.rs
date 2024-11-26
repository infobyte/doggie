use embedded_can::{blocking::Can, Id};

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum CanBitrates {
    Kbps5 = 5,
    Kbps10 = 10,
    Kbps20 = 20,
    Kbps31_25 = 31,
    Kbps33_3 = 33,
    Kbps40 = 40,
    Kbps50 = 50,
    Kbps80 = 80,
    Kbps100 = 100,
    Kbps125 = 125,
    Kbps200 = 200,
    Kbps250 = 250,
    Kbps500 = 500,
    Kbps1000 = 1000,
}

impl From<u16> for CanBitrates {
    fn from(value: u16) -> CanBitrates {
        match value {
            5 => CanBitrates::Kbps5,
            10 => CanBitrates::Kbps10,
            20 => CanBitrates::Kbps20,
            31 => CanBitrates::Kbps31_25,
            33 => CanBitrates::Kbps33_3,
            40 => CanBitrates::Kbps40,
            50 => CanBitrates::Kbps50,
            80 => CanBitrates::Kbps80,
            100 => CanBitrates::Kbps100,
            125 => CanBitrates::Kbps125,
            200 => CanBitrates::Kbps200,
            250 => CanBitrates::Kbps250,
            500 => CanBitrates::Kbps500,
            1000 => CanBitrates::Kbps1000,
            _ => CanBitrates::Kbps250,
        }
    }
}

pub trait CanDevice: Can {
    fn set_bitrate(&mut self, bitrate: CanBitrates);

    fn set_filter(&mut self, id: Id);

    fn set_mask(&mut self, id: Id);
}
