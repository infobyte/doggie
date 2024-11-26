use embassy_stm32::peripherals;
use embassy_stm32::{gpio::Pull, mode, spi, spi::MODE_0};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    time::Hertz,
};

use crate::spi_device::CustomSpiDevice;

pub fn create_spi<'d>(
    spi: peripherals::SPI1,
    miso: peripherals::PA5,
    mosi: peripherals::PA7,
    clk: peripherals::PA6,
    cs: peripherals::PA4,
) -> CustomSpiDevice<'d, mode::Blocking> {
    // Setup SPI
    let mut spi_config = spi::Config::default();
    spi_config.mode = MODE_0;
    spi_config.frequency = Hertz(1_000_000);
    spi_config.miso_pull = Pull::Down;

    let stm_spi = spi::Spi::new_blocking(spi, miso, mosi, clk, spi_config);

    let cs = Output::new(cs, Level::High, Speed::VeryHigh);

    CustomSpiDevice::new(stm_spi, cs)
}

#[macro_export]
macro_rules! create_default_spi {
    ($p:expr) => {{
        spi::create_spi($p.SPI1, $p.PA5, $p.PA7, $p.PA6, $p.PA4)
    }};
}
