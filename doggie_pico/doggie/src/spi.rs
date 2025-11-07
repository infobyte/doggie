use crate::spi_device::CustomSpiDevice;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{Blocking, Config, Spi};
use embassy_rp::{peripherals, Peri};

pub fn create_spi<'d>(
    spi: Peri<'d, peripherals::SPI0>,
    clk: Peri<'d, peripherals::PIN_18>,
    mosi: Peri<'d, peripherals::PIN_19>,
    miso: Peri<'d, peripherals::PIN_16>,
    cs: Peri<'d, peripherals::PIN_17>,
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
