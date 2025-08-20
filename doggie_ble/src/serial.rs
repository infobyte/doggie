use crate::types::{BlePipeReader, BlePipeWriter};
use embassy_futures::block_on;
use embedded_io;
use embedded_io_async::{Read, Write};

pub struct BleSerial {
    writer: BlePipeWriter,
    reader: BlePipeReader,
}

impl BleSerial {
    pub fn new(writer: BlePipeWriter, reader: BlePipeReader) -> Self {
        BleSerial { writer, reader }
    }
}

impl embedded_io_async::ErrorType for BleSerial {
    type Error = core::convert::Infallible;
}

impl Read for BleSerial {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(self.reader.read(buf).await)
    }
}

impl Write for BleSerial {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(self.writer.write(buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'d> embedded_io::Read for BleSerial {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        block_on(embedded_io_async::Read::read(self, buf))
    }
}

impl<'d> embedded_io::Write for BleSerial {
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        block_on(embedded_io_async::Write::write(self, buf))
    }
}
