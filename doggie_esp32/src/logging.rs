use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output},
    uart::{
        Uart, UartTx,
    }
};

static mut LOGGER: Option<UartTx<'static, esp_hal::Blocking>> = None;
static mut ENCODER: defmt::Encoder = defmt::Encoder::new();


pub fn init_logs(dbg_tx: UartTx<'static, esp_hal::Blocking>) {
    unsafe {
        LOGGER.replace(dbg_tx);
    }
}


// Global defmt logger configuration
#[defmt::global_logger]
struct Logger;

impl Logger {
    fn do_write(bytes: &[u8]) {
        unsafe {
            let logger = LOGGER.as_mut().unwrap();
            logger.write_bytes(bytes).unwrap();
        }
    }
}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        unsafe { ENCODER.start_frame(Logger::do_write) }
    }

    unsafe fn flush() {
        let logger = LOGGER.as_mut().unwrap();

        logger.flush_tx().unwrap();
    }

    unsafe fn release() {
        ENCODER.end_frame(Logger::do_write);
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, Logger::do_write);
    }
}
