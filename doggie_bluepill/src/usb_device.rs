use defmt::{error, println};
use embassy_stm32::{peripherals::USB, usb::Driver};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embedded_io::{ErrorKind, ErrorType};
use embedded_io_async::{Error, Read, Write};

pub struct UsbWrapper<'d> {
    usb: CdcAcmClass<'d, Driver<'d, USB>>,
}

impl<'d> UsbWrapper<'d> {
    pub fn new(usb: CdcAcmClass<'d, Driver<'d, USB>>) -> Self {
        UsbWrapper { usb }
    }
}

#[derive(Debug)]
pub struct UsbError {}

impl<'d> Error for UsbError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl<'d> ErrorType for UsbWrapper<'d> {
    type Error = UsbError;
}

impl<'d> Read for UsbWrapper<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match self.usb.read_packet(buf).await {
            Err(_) => Err(UsbError {}),
            Ok(size) => Ok(size),
        }
    }
}

impl<'d> Write for UsbWrapper<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        println!("WRITING: {}", buf);
        match self.usb.write_packet(buf).await {
            Err(_) => {
                error!("Error on the usb write");
                Err(UsbError {})
            }
            Ok(_) => Ok(buf.len()),
        }
    }
}
