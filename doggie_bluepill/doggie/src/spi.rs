use embassy_stm32::peripherals;
use embassy_stm32::{gpio::Pull, mode, spi, spi::MODE_0};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    time::Hertz,
};

use crate::spi_device::CustomSpiDevice;

pub fn create_spi<'d>(
    spi: peripherals::SPI2,
    miso: peripherals::PB14,
    mosi: peripherals::PB15,
    clk: peripherals::PB13,
    cs: peripherals::PB12,
) -> CustomSpiDevice<'d, mode::Blocking> {
    // Setup SPI
    let mut spi_config = spi::Config::default();
    spi_config.mode = MODE_0;
    spi_config.frequency = Hertz(10_000_000);
    spi_config.miso_pull = Pull::Down;

    let stm_spi = spi::Spi::new_blocking(spi, clk, mosi, miso, spi_config);

    let cs = Output::new(cs, Level::High, Speed::VeryHigh);

    CustomSpiDevice::new(stm_spi, cs)
}

#[macro_export]
macro_rules! create_default_spi {
    ($p:expr) => {{
        spi::create_spi($p.SPI2, $p.PB14, $p.PB15, $p.PB13, $p.PB12)
    }};
}
