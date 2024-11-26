use esp_hal::{
    Mode,
    spi::master::Spi,
    spi::Error
};

use embedded_hal::spi::{ErrorType, Operation, SpiDevice, SpiBus};

pub struct CustomSpiDevice<'d, MODE: Mode> {
    spi: Spi<'d, MODE>
}

impl<'d, MODE: Mode> CustomSpiDevice<'d, MODE> {
    pub fn new(spi: Spi<'d, MODE>) -> Self {
        CustomSpiDevice { spi }
    }
}

impl<'d, MODE: Mode> ErrorType for CustomSpiDevice<'d, MODE> {
    type Error = Error;
}

impl<'d, MODE: Mode> SpiDevice<u8> for CustomSpiDevice<'d, MODE> {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                Operation::Read(buffer) => {
                    self.spi.read(buffer)?;
                }
                Operation::Write(data) => {
                    self.spi.write(data)?;
                }
                Operation::Transfer(read_buffer, write_data) => {
                    embedded_hal::spi::SpiBus::transfer(&mut self.spi, read_buffer, write_data)?;
                }
                Operation::TransferInPlace(buffer) => {
                    self.spi.transfer_in_place(buffer)?;
                }
                Operation::DelayNs(ns) => {
                    let delay = esp_hal::delay::Delay::new();
                    delay.delay_nanos(*ns);
                }
            }
        }

        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(buf)
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        embedded_hal::spi::SpiBus::transfer(&mut self.spi, read, write)
    }

    fn transfer_in_place(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.transfer_in_place(buf)
    }
}
