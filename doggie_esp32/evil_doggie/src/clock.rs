use defmt::{info, println};
use esp_hal::{clock::Clocks, timer::timg::Timer as TimerX, timer::Timer};
use evil_core::bsp::TicksClock;

#[cfg(feature = "esp32")]
use core::arch::asm;

pub struct TimerBasedClock {
    _timer: esp_hal::timer::timg::Timer<'static>,
}

impl TimerBasedClock {
    #[cfg(feature = "esp32")]
    const TIMG1_BASE: u32 = 0x3FF6_0000;

    #[cfg(feature = "esp32c3")]
    const TIMG1_BASE: u32 = 0x6002_0000;

    const TIMG1_UPDATE_OFFSET: u32 = 0xC;
    const TIMG1_LO_OFFSET: u32 = 0x4;

    const TIMG1_UPDATE: *mut u32 = (Self::TIMG1_BASE + Self::TIMG1_UPDATE_OFFSET) as *mut u32;
    const TIMG1_LO: *mut u32 = (Self::TIMG1_BASE + Self::TIMG1_LO_OFFSET) as *mut u32;

    pub fn new(timer: TimerX<'static>) -> Self {
        timer.enable_auto_reload(true);
        timer.start();
        // self.set_alarm_active(true);

        let apb_freq = Clocks::get().apb_clock;
        // let divider = timer.divider();

        info!("TIMG1 initialization");
        println!("\tAPB clock freq: {} Hz", apb_freq);
        // println!("\tDivider: {}", divider);
        // println!("\tTimer freq: {} Hz", apb_freq / divider);

        Self { _timer: timer }
    }
}

impl TicksClock for TimerBasedClock {
    #[cfg(feature = "esp32c3")]
    const TICKS_PER_SEC: u32 = 40_000_000; // Adjust this to match your timer frequency

    #[cfg(feature = "esp32")]
    const TICKS_PER_SEC: u32 = 240_000_000; // Adjust this to match your timer frequency

    #[inline(always)]
    fn ticks(&self) -> u32 {
        #[cfg(feature = "esp32c3")]
        let res = unsafe {
            core::ptr::write_volatile(Self::TIMG1_UPDATE, 1);

            // We need to give some time to the timer register to be updated
            // The amount of nops are calculated for the board used in development
            // and may vary
            for _ in 0..11 {
                riscv::asm::nop();
            }
            core::ptr::read_volatile(Self::TIMG1_LO)
        };

        #[cfg(feature = "esp32")]
        let res = unsafe {
            let x: u32;
            asm!("rsr.ccount {0}", out(reg) x, options(nostack));
            x
        };

        res
    }

    // These only works on 32-bit timers
    #[inline(always)]
    fn add_ticks(t1: u32, t2: u32) -> u32 {
        // Handle potential overflow with wrapping_add
        t1.wrapping_add(t2)
    }

    #[inline(always)]
    fn sub_ticks(t1: u32, t2: u32) -> u32 {
        // Handle potential overflow with wrapping_add
        t1.wrapping_sub(t2)
    }
}
