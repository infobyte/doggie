use embedded_hal::delay::DelayNs;

pub struct SoftTimer {}

impl DelayNs for SoftTimer {
    // Required method
    fn delay_ns(&mut self, ns: u32) {
        // Aprox
        cortex_m::asm::delay(ns / 20);
    }
}
