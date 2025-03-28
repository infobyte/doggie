use crate::uart_device::UartWrapper;
use embassy_stm32::{
    bind_interrupts, peripherals,
    usart::{self, Uart},
};

bind_interrupts!(struct UartIrqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

pub fn create_uart<'d>(
    uart: peripherals::USART2,
    tx: peripherals::PA2,
    rx: peripherals::PA3,
    dma1: peripherals::DMA1_CH7,
    dma2: peripherals::DMA1_CH6,
) -> UartWrapper<'d> {
    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 921_600;

    // Initialize UART
    UartWrapper::new(Uart::new(uart, rx, tx, UartIrqs, dma1, dma2, uart_config).unwrap())
}

#[macro_export]
macro_rules! create_default_uart {
    ($p:expr) => {{
        uart::create_uart($p.USART2, $p.PA2, $p.PA3, $p.DMA1_CH7, $p.DMA1_CH6)
    }};
}
