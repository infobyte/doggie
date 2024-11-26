#![no_std]
#![no_main]
mod bluepill;
mod can_device;
mod uart;
mod uart_device;

use can_device::CanWrapper;
use uart_device::UartWrapper;

use doggie_core::{
    core_create_tasks, core_run, Bsp, CanChannel, CanChannelReceiver, CanChannelSender, Core,
};

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    can::{
        filter, Can, Fifo, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler,
        TxInterruptHandler,
    },
    gpio::{Level, Output, Speed},
    peripherals::CAN,
};
use embassy_time::Timer;

bind_interrupts!(struct CanIrqs {
    USB_LP_CAN1_RX0 => Rx0InterruptHandler<CAN>;
    CAN1_RX1 => Rx1InterruptHandler<CAN>;
    CAN1_SCE => SceInterruptHandler<CAN>;
    USB_HP_CAN1_TX => TxInterruptHandler<CAN>;
});

#[embassy_executor::task]
async fn blink_task(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(300).await;

        led.set_low();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = bluepill::init();

    let led = Output::new(p.PC13, Level::High, Speed::Low);

    spawner.spawn(blink_task(led)).unwrap();

    let serial = create_default_uart!(p);

    // Set alternate pin mapping to B8/B9
    embassy_stm32::pac::AFIO
        .mapr()
        .modify(|w| w.set_can1_remap(2));

    let mut can = Can::new(p.CAN, p.PB8, p.PB9, CanIrqs);

    can.modify_filters()
        .enable_bank(0, Fifo::Fifo0, filter::Mask32::accept_all());

    can.modify_config()
        .set_loopback(false)
        .set_silent(false)
        .set_bitrate(250_000);

    can.enable().await;

    let can_wrapper = CanWrapper::new(can);

    let bsp = Bsp::new(can_wrapper, serial);

    // Create and run the Doggie core
    let core = Core::new(spawner, bsp);

    info!("About to run CORE");
    core_run!(core);
}

type SerialType = UartWrapper<'static>;
type CanType = CanWrapper<'static>;

core_create_tasks!(SerialType, CanType);
