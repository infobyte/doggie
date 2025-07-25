use esp_hal::gpio::{Input, Output};
use evil_core::bsp::Tranceiver;

#[derive(Copy, Clone)]
pub struct EspTranceiver {}

impl EspTranceiver {
    #[cfg(feature = "esp32")]
    const GPIO_OUT_W1TS_REG: *mut u32 = 0x3FF4_4008 as *mut u32; // GPIO bit set register
    #[cfg(feature = "esp32")]
    const GPIO_OUT_W1TC_REG: *mut u32 = 0x3FF4_400C as *mut u32; // GPIO bit clear register
    #[cfg(feature = "esp32")]
    const GPIO_IN_REG: *mut u32 = 0x3FF4_403C as *mut u32; // GPIO input register
    #[cfg(feature = "esp32")]
    const TX_OFFSET: u32 = 26;
    #[cfg(feature = "esp32")]
    const RX_OFFSET: u32 = 25;
    #[cfg(feature = "esp32")]
    const FORCE_OFFSET: u32 = 27;
    #[cfg(feature = "esp32")]
    const DEBUG_OFFSET: u32 = 2;

    #[cfg(feature = "esp32c3")]
    const GPIO_OUT_W1TS_REG: *mut u32 = 0x6000_4008 as *mut u32; // GPIO bit set register
    #[cfg(feature = "esp32c3")]
    const GPIO_OUT_W1TC_REG: *mut u32 = 0x6000_400C as *mut u32; // GPIO bit clear register
    #[cfg(feature = "esp32c3")]
    const GPIO_IN_REG: *mut u32 = 0x6000_403C as *mut u32; // GPIO input register
    #[cfg(feature = "esp32c3")]
    const TX_OFFSET: u32 = 1;
    #[cfg(feature = "esp32c3")]
    const RX_OFFSET: u32 = 0;
    #[cfg(feature = "esp32c3")]
    const FORCE_OFFSET: u32 = 10;
    #[cfg(feature = "esp32c3")]
    const DEBUG_OFFSET: u32 = 2;

    pub fn new() -> Self {
        EspTranceiver {}
    }
}

impl Tranceiver for EspTranceiver {
    #[inline(always)]
    fn set_tx(&mut self, state: bool) {
        unsafe {
            if state {
                core::ptr::write_volatile(Self::GPIO_OUT_W1TS_REG, 1 << Self::TX_OFFSET);
            } else {
                core::ptr::write_volatile(Self::GPIO_OUT_W1TC_REG, 1 << Self::TX_OFFSET);
            }
        }
    }

    #[inline(always)]
    fn get_rx(&self) -> bool {
        unsafe { (core::ptr::read_volatile(Self::GPIO_IN_REG) & (1 << Self::RX_OFFSET)) != 0 }
    }

    #[inline(always)]
    fn set_force(&mut self, state: bool) {
        unsafe {
            if state {
                core::ptr::write_volatile(Self::GPIO_OUT_W1TC_REG, 1 << Self::FORCE_OFFSET);
            } else {
                core::ptr::write_volatile(Self::GPIO_OUT_W1TS_REG, 1 << Self::FORCE_OFFSET);
            }
        }
    }

    #[inline(always)]
    fn set_debug(&mut self, state: bool) {
        unsafe {
            if state {
                core::ptr::write_volatile(Self::GPIO_OUT_W1TC_REG, 1 << Self::DEBUG_OFFSET);
            } else {
                core::ptr::write_volatile(Self::GPIO_OUT_W1TS_REG, 1 << Self::DEBUG_OFFSET);
            }
        }
    }
}
