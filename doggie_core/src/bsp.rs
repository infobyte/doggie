use embedded_can::blocking::Can;
use embedded_io_async::{Read, Write};

use core::cell::RefCell;

pub struct Bsp<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write,
{
    pub can: RefCell<Option<CAN>>,
    pub serial: RefCell<Option<SERIAL>>,
}

impl<CAN, SERIAL> Bsp<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write,
{
    pub fn new(can: CAN, serial: SERIAL) -> Self {
        Bsp {
            can: RefCell::new(Some(can)),
            serial: RefCell::new(Some(serial)),
        }
    }

    // pub fn split(self) -> (CAN, SERIAL) {
    //     (
    //         self.can.replace(None).unwrap(),
    //         self.serial.replace(None).unwrap(),
    //     )
    // }
}
