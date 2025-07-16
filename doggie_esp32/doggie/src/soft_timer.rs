use embedded_hal::delay::DelayNs;

pub struct SoftTimer {}

impl DelayNs for SoftTimer {
    // Required method
    fn delay_ns(&mut self, ns: u32) {
        let delay = esp_hal::delay::Delay::new();
        delay.delay_nanos(ns);
    }
}
