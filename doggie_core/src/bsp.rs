use embedded_can::blocking::Can;
use embedded_io::{Read, Write};

// pub struct Bsp<CAN, SERIAL>
// where
//     CAN: Can,
//     SERIAL: Read + Write,
// {
//     pub can: CAN,
//     pub serial: SERIAL,
// }
//
// impl<CAN, SERIAL> Bsp<CAN, SERIAL>
// where
//     CAN: Can,
//     SERIAL: Read + Write,
// {
//     pub fn new(can: CAN, serial: SERIAL) -> Self {
//         Bsp { can, serial }
//     }
//
//     pub fn split(self) -> (CAN, SERIAL) {
//         (self.can, self.serial)
//     }
// }

pub struct Bsp<CAN>
where
    CAN: Can,
{
    pub can: CAN,
}

impl<CAN> Bsp<CAN>
where
    CAN: Can,
{
    pub fn new(can: CAN) -> Self {
        Bsp { can }
    }

    pub fn split(self) -> CAN {
        self.can
    }
}
