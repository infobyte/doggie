#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

mod clock;
mod tranceiver;

use clock::TimerBasedClock;
use tranceiver::EspTranceiver;

use defmt::info;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Level, Output, Pull},
    timer::timg::TimerGroup,
    uart::Uart,
    ram,
    clock::CpuClock,
    Async,
};
use evil_core::{
    bsp::{CanBitrates, EvilBsp},
    EvilCore, EvilMenu,
};
use esp_serial;

esp_serial::init_globals!();


#[no_mangle]
#[ram]
fn esp32_attack(core: &mut EvilCore<TimerBasedClock, EspTranceiver<'_>>) {
    #[cfg(target_arch = "xtensa")]
    xtensa_lx::interrupt::free(|_| {
        // Interrupts disabled
        core.attack();
    });

    #[cfg(target_arch = "riscv32")]
    riscv::interrupt::free(|| {
        core.attack();
    });
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let mut config = esp_hal::Config::default();
    // esp32   => 240MHz
    // esp32c3 => 160MHz
    config.cpu_clock = CpuClock::max();

    let p = esp_hal::init(config);

    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    esp_serial::init_dbg!(p);

    info!("Evil Doggie initialization!");
    // info!("CPU clock: {}", config.cpu_clock.hz());

    info!("Wired serial init");
    let serial = esp_serial::create_wired_serial!(p);
    
    info!("Serial init ok");

    // Setup tx, rx, and force pins, and tranceiver
    #[cfg(feature = "esp32")]
    let (tx_pin, rx_pin, force_pin) = (p.GPIO26, p.GPIO25, p.GPIO27);

    #[cfg(feature = "esp32c3")]
    let (tx_pin, rx_pin, force_pin) = (p.GPIO1, p.GPIO0, p.GPIO10);

    let tx = Output::new(tx_pin, Level::High);
    let rx = Input::new(rx_pin, Pull::None);
    let force = Output::new(force_pin, Level::High);

    let tranceiver = EspTranceiver::new(tx, rx, force);
    info!("Tranceiver init ok");

    // Create clock
    let timg1_t0: esp_hal::timer::timg::Timer = TimerGroup::new(p.TIMG1).timer0;
    let clock = TimerBasedClock::new(timg1_t0);

    // Create the EvilBsp
    let bsp = EvilBsp::new(clock, tranceiver);
    info!("BSP created");

    // Create and run the EvilDoggie core
    #[cfg(feature = "esp32c3")]
    let sof_delay_ns = 450;

    #[cfg(feature = "esp32")]
    let sof_delay_ns = 400;

    let core = EvilCore::new(bsp, CanBitrates::Kbps250, sof_delay_ns, esp32_attack);
    info!("Evil core created, running evil menu");

    let mut menu = EvilMenu::new(serial, core);
    menu.run();
}
