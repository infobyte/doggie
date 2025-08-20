#![no_std]
#![no_main]

mod soft_timer;
mod spi;
mod spi_device;

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    clocks::{ClockConfig, PllConfig, XoscConfig},
    gpio::{Input, Level, Output, Pull},
};
use rp;
use {defmt_rtt as _, panic_probe as _};

use evil_core::{
    bsp::{CanBitrates, EvilBsp, TicksClock, Tranceiver},
    EvilCore, EvilMenu,
};

use static_cell::StaticCell;

rp::init_globals!();

struct SystickClock {
    systick: cortex_m::peripheral::SYST,
}

impl SystickClock {
    pub fn new(mut systick: cortex_m::peripheral::SYST) -> Self {
        // Configure SysTick
        systick.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);

        // Set reload value (maximum 24-bit value)
        systick.set_reload(0x00FF_FFFF);

        // Clear current value by writing any value to CVR
        systick.clear_current();

        // Enable counter
        systick.enable_counter();

        Self { systick }
    }
}

impl TicksClock for SystickClock {
    const TICKS_PER_SEC: u32 = 120_000_000;

    #[inline(always)]
    fn ticks(&self) -> u32 {
        0xFF_FFFF - self.systick.cvr.read()
    }

    #[inline(always)]
    fn add_ticks(t1: u32, t2: u32) -> u32 {
        (t1 + t2) % 0x1000000
    }

    #[inline(always)]
    fn sub_ticks(t1: u32, t2: u32) -> u32 {
        t1.wrapping_sub(t2) % 0x1000000
    }
}

struct BpTr<'a> {
    _tx: Output<'a>,
    _rx: Input<'a>,
    _force: Output<'a>,
}

impl<'a> BpTr<'a> {
    pub fn new(tx: Output<'a>, rx: Input<'a>, force: Output<'a>) -> Self {
        BpTr {
            _tx: tx,
            _rx: rx,
            _force: force,
        }
    }
}

impl<'a> Tranceiver for BpTr<'a> {
    #[inline(always)]
    fn set_tx(&mut self, state: bool) {
        // Direct memory access for pin 20
        const GPIO_OUT_SET: *mut u32 = 0xD000_0014 as *mut u32;
        const GPIO_OUT_CLR: *mut u32 = 0xD000_0018 as *mut u32;
        unsafe {
            if state {
                core::ptr::write_volatile(GPIO_OUT_SET, 1 << 20);
            } else {
                core::ptr::write_volatile(GPIO_OUT_CLR, 1 << 20);
            }
        }
    }

    #[inline(always)]
    fn get_rx(&self) -> bool {
        // Direct memory access for pin 21
        const GPIO_IN: *const u32 = 0xD000_0004 as *const u32;
        unsafe { (core::ptr::read_volatile(GPIO_IN) & (1 << 21)) != 0 }
    }

    #[inline(always)]
    fn set_force(&mut self, state: bool) {
        // Direct memory access for 22
        const GPIO_OUT_SET: *mut u32 = 0xD000_0014 as *mut u32;
        const GPIO_OUT_CLR: *mut u32 = 0xD000_0018 as *mut u32;
        unsafe {
            if state {
                core::ptr::write_volatile(GPIO_OUT_SET, 1 << 22);
                // self._force.set_high();
            } else {
                core::ptr::write_volatile(GPIO_OUT_CLR, 1 << 22);
                // self._force.set_low();
            }
        }
    }

    #[inline(always)]
    fn set_debug(&mut self, _state: bool) {}
}

fn rp_attack<Clock: TicksClock, Tr: Tranceiver>(core: &mut EvilCore<Clock, Tr>) -> bool {
    let mut res = false;
    cortex_m::interrupt::free(|_| res = core.attack());

    res
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Device initialization");
    let mut clocks = ClockConfig::crystal(12_000_000);

    clocks.xosc.replace(XoscConfig {
        hz: 12_000_000,
        sys_pll: Some(PllConfig {
            refdiv: 1,
            fbdiv: 120,
            post_div1: 6,
            post_div2: 2,
        }),
        usb_pll: Some(PllConfig {
            refdiv: 1,
            fbdiv: 120,
            post_div1: 6,
            post_div2: 5,
        }),
        delay_multiplier: 128,
    });

    // Initialize with custom clocks
    let p = embassy_rp::init(embassy_rp::config::Config::new(clocks));

    let tx = Output::new(p.PIN_20, Level::High);
    let rx = Input::new(p.PIN_21, Pull::Up);
    let force = Output::new(p.PIN_22, Level::Low);

    let tranceiver = BpTr::new(tx, rx, force);

    let cp = cortex_m::Peripherals::take().unwrap();
    let systick = SystickClock::new(cp.SYST);

    let serial = rp::create_serial!(p, spawner);

    info!("Systick reload value: {:x}", systick.systick.rvr.read());

    // let bsp: EvilBsp<_, _> = EvilBsp::new_with_mcp2515(spi, delay, systick, tranceiver);
    let bsp: EvilBsp<_, _> = EvilBsp::new(systick, tranceiver);

    info!("BSP created");

    // Create and run the Doggie core
    let mut core = EvilCore::new(bsp, CanBitrates::Kbps500, 850, rp_attack);

    info!("Core created");

    info!("Evil core created, running evil menu");

    let mut menu = EvilMenu::new(serial, core);
    menu.run();
}
