use embedded_can::blocking::Can;
use embedded_io_async::{Read, Write};

pub struct Bsp<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write,
{
    pub can: CAN,
    pub serial: SERIAL,
}

impl<CAN, SERIAL> Bsp<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write,
{
    pub fn new(can: CAN, serial: SERIAL) -> Self {
        Bsp { can, serial }
    }

    pub fn split(self) -> (CAN, SERIAL) {
        (self.can, self.serial)
    }
}
