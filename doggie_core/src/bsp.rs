use embedded_io_async::{Read, Write};

use crate::can::CanDevice;

use core::cell::RefCell;

pub struct Bsp<CAN, SERIAL>
where
    CAN: CanDevice,
    SERIAL: Read + Write,
{
    pub can: RefCell<Option<CAN>>,
    pub serial: RefCell<Option<SERIAL>>,
}

impl<CAN, SERIAL> Bsp<CAN, SERIAL>
where
    CAN: CanDevice,
    SERIAL: Read + Write,
{
    pub fn new(can: CAN, serial: SERIAL) -> Self {
        Bsp {
            can: RefCell::new(Some(can)),
            serial: RefCell::new(Some(serial)),
        }
    }
}
