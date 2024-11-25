use embassy_stm32::{
    mode::Async,
    usart::{Error as UartError, Uart},
};
use embedded_io::ErrorType;
use embedded_io_async::{Read, Write};

pub struct UartWrapper<'d> {
    uart: Uart<'d, Async>,
}

impl<'d> UartWrapper<'d> {
    pub fn new(uart: Uart<'d, Async>) -> Self {
        UartWrapper { uart }
    }
}

impl<'d> ErrorType for UartWrapper<'d> {
    type Error = UartError;
}

impl<'d> Read for UartWrapper<'d> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.uart.read_until_idle(buf).await
    }
}

impl<'d> Write for UartWrapper<'d> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        embedded_io_async::Write::write(&mut self.uart, buf).await
    }
}
