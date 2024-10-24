use embassy_rp::{
    gpio::Output,
    spi::{Error as SpiError, Spi, Mode, Instance},
};
use embedded_hal::spi::{ErrorType, Operation, SpiDevice};

pub struct CustomSpiDevice<'d, T: Instance, MODE: Mode> {
    spi: Spi<'d, T, MODE>,
    cs: Output<'d>,
}

impl<'d, T: Instance, MODE: Mode> CustomSpiDevice<'d, T, MODE> {
    pub fn new(spi: Spi<'d, T, MODE>, mut cs: Output<'d>) -> Self {
        cs.set_high();
        CustomSpiDevice { spi, cs }
    }
}

impl<'d, T: Instance, MODE: Mode> ErrorType for CustomSpiDevice<'d, T, MODE> {
    type Error = SpiError;
}

impl<'d, T: Instance, MODE: Mode> SpiDevice<u8> for CustomSpiDevice<'d, T, MODE> {
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
                    cortex_m::asm::delay(*ns / 10); // Aproximation
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
