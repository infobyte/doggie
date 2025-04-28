#![no_std]
#![no_main]

mod soft_timer;
mod spi_device;
use soft_timer::SoftTimer;

use evil_core::{
    clock::TicksClock, tranceiver::Tranceiver, AttackCmd, BitStream, CanBitrates, EvilBsp,
    EvilCore, FastBitQueue,
};

use evil_menu::EvilMenu;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    clock::Clocks,
    delay::Delay,
    gpio::{Input, Level, Output, Pull},
    peripheral::Peripheral,
    prelude::*,
    spi::{master::Spi, SpiMode},
    timer::timg::TimerGroup,
    uart::Uart,
};
use esp_println as _;

use defmt::{info, println};
use spi_device::CustomSpiDevice;

const READ_BUF_SIZE: usize = 64;

struct TimerBasedClock {
    timer: esp_hal::timer::timg::Timer<
        esp_hal::timer::timg::TimerX<<esp_hal::peripherals::TIMG1 as Peripheral>::P>,
        esp_hal::Blocking,
    >,
}

impl TimerBasedClock {
    pub fn new(
        mut timer: esp_hal::timer::timg::Timer<
            esp_hal::timer::timg::TimerX<<esp_hal::peripherals::TIMG1 as Peripheral>::P>,
            esp_hal::Blocking,
        >,
    ) -> Self {
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
        println!("Timer freq: {} Hz", apb_freq / divider);

        Self { timer }
    }
}

static mut COUNTER: u32 = 0;

impl TicksClock for TimerBasedClock {
    const TICKS_PER_SEC: u32 = 40_000_000; // Adjust this to match your timer frequency

    #[inline(always)]
    fn ticks(&self) -> u32 {
        // // Access timer registers to read current count
        // let regs = self.timer.register_block().t(self.timer.timer_number().into());

        // regs.update().write(|w| w.update().set_bit());
        // while regs.update().read().update().bit_is_set() {
        //     // Wait for the update to complete
        // }
        // regs.lo().read().bits()
        // unsafe {
        //     COUNTER
        // }
        unsafe {
            let timg1_t0_update: *mut u32 = 0x3ff6_000c as *mut u32;
            let timg1_t0_lo: *mut u32 = 0x3ff6_0004 as *mut u32;
            core::ptr::write_volatile(timg1_t0_update, 1);
            core::ptr::read_volatile(timg1_t0_lo)
        }
    }

    #[inline(always)]
    fn add_ticks(t1: u32, t2: u32) -> u32 {
        // Handle potential overflow with wrapping_add
        t1 + t2
        // unsafe {
        //     COUNTER += t2 + 1;
        //     COUNTER
        // }
    }
}

const GPIO_OUT_REG: *mut u32 = 0x3FF4_4004 as *mut u32; // GPIO output register
const GPIO_OUT_W1TS_REG: *mut u32 = 0x3FF4_4008 as *mut u32; // GPIO bit set register
const GPIO_OUT_W1TC_REG: *mut u32 = 0x3FF4_400C as *mut u32; // GPIO bit clear register
const GPIO_IN_REG: *mut u32 = 0x3FF4_403c as *mut u32; // GPIO input register

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
    #[inline(always)]
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

    #[inline(always)]
    fn get_rx(&self) -> bool {
        // Direct memory access to GPIO26 as input
        unsafe {
            // Read PA9 (bit 9) and check if it's high
            (core::ptr::read_volatile(GPIO_IN_REG) & (1 << 26)) != 0
        }
    }

    #[inline(always)]
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
#[ram]
fn esp32_attack(core: &mut EvilCore<TimerBasedClock, EspTranceiver<'_>>) {
    xtensa_lx::interrupt::free(|_| {
        // Interrupts disabled
        core.attack();
    });
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    info!("Init!");

    let mut cfg = esp_hal::Config::default();
    cfg.cpu_clock = CpuClock::Clock240MHz;

    info!("CPU clock: {}", cfg.cpu_clock.hz());

    let p = esp_hal::init(cfg);

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Setup serial
    let (tx_pin, rx_pin) = (p.GPIO1, p.GPIO3);
    let config = esp_hal::uart::Config::default().rx_fifo_full_threshold(READ_BUF_SIZE as u16);
    let serial = Uart::new_with_config(p.UART0, config, rx_pin, tx_pin).unwrap();

    info!("Serial init ok");

    // Setup tx, rx, and force pins, and tranceiver
    let tx = Output::new(p.GPIO25, Level::Low);
    let rx = Input::new(p.GPIO26, Pull::None);
    let force = Output::new(p.GPIO27, Level::Low);

    let tranceiver = EspTranceiver::new(tx, rx, force);
    info!("Tranceiver init ok");

    // Create clock
    let timg1_t0: esp_hal::timer::timg::Timer<
        esp_hal::timer::timg::TimerX<<esp_hal::peripherals::TIMG1 as Peripheral>::P>,
        esp_hal::Blocking,
    > = TimerGroup::new(p.TIMG1).timer0;
    let clock = TimerBasedClock::new(timg1_t0);

    // Create the EvilBsp
    let bsp = EvilBsp::new(clock, tranceiver);
    info!("BSP created");

    // Create and run the EvilDoggie core
    let core = EvilCore::new(bsp, CanBitrates::Kbps1000, 0, esp32_attack);
    info!("Core created");

    let mut menu = EvilMenu::new(serial, core);
    menu.run();

    // Delay::new().delay_millis(100);

    // loop {
    //     core.arm(&[
    //         // AttackCmd::Wait { bits: 1 },
    //         AttackCmd::Force {
    //             stream: FastBitQueue::new(0b1010_101, 7),
    //         },
    //         AttackCmd::Wait { bits: 1 },
    //         AttackCmd::Force {
    //             stream: FastBitQueue::new(0b1010_101, 7),
    //         },
    //         // AttackCmd::Wait { bits: 1 },
    //         // AttackCmd::Match { stream: FastBitQueue::new(0x123, 11) },
    //         // AttackCmd::Wait { bits: 3 },
    //         // AttackCmd::Read { len: 4 },
    //         // AttackCmd::WaitBuffered,
    //         // AttackCmd::Wait { bits: 16 },
    //         // AttackCmd::Force { stream: FastBitQueue::new(0b101, 3) },
    //     ])
    //     .unwrap();

    //     info!("Attack armed");

    //     esp32_attack(&mut core);

    //     info!("Attack has finished");
    // }
}

// core_create_tasks!(
//     Uart<'static, Async>,
//     MCP2515<CustomSpiDevice<'static, Blocking>>
// );
