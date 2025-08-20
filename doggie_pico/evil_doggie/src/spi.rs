use crate::spi_device::CustomSpiDevice;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals;
use embassy_rp::spi::{Blocking, Config, Spi};

pub fn create_spi<'d>(
    spi: peripherals::SPI0,
    clk: peripherals::PIN_18,
    mosi: peripherals::PIN_19,
    miso: peripherals::PIN_16,
    cs: peripherals::PIN_17,
) -> CustomSpiDevice<'d, peripherals::SPI0, Blocking> {
    // Setup SPI
    let mut spi_config = Config::default();
    spi_config.frequency = 10_000_000;

    let rp_spi = Spi::new_blocking(spi, clk, mosi, miso, spi_config);
    let cs = Output::new(cs, Level::High);
    CustomSpiDevice::new(rp_spi, cs)
}

#[macro_export]
macro_rules! create_default_spi {
    ($p:expr) => {{
        spi::create_spi($p.SPI0, $p.PIN_18, $p.PIN_19, $p.PIN_16, $p.PIN_17)
    }};
}
