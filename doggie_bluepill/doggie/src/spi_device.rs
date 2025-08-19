use embassy_stm32::{
    gpio::Output,
    mode::Mode,
    spi::{Error as SpiError, Spi},
};
use embedded_hal::spi::{ErrorType, Operation, SpiDevice};

pub struct CustomSpiDevice<'d, MODE: Mode> {
    spi: Spi<'d, MODE>,
    cs: Output<'d>,
}

impl<'d, MODE: Mode> CustomSpiDevice<'d, MODE> {
    pub fn new(spi: Spi<'d, MODE>, mut cs: Output<'d>) -> Self {
        cs.set_high();
        CustomSpiDevice { spi, cs }
    }
}

impl<'d, MODE: Mode> ErrorType for CustomSpiDevice<'d, MODE> {
    type Error = SpiError;
}

impl<'d, MODE: Mode> SpiDevice<u8> for CustomSpiDevice<'d, MODE> {
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
