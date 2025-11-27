use core::{cell::RefCell, hash::Hasher};
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use esp_hal::uart::UartTx;

static LOGGER: Mutex<CriticalSectionRawMutex, RefCell<Option<UartTx<'static, esp_hal::Blocking>>>> =
    Mutex::new(RefCell::new(None));
static ENCODER: Mutex<CriticalSectionRawMutex, RefCell<defmt::Encoder>> =
    Mutex::new(RefCell::new(defmt::Encoder::new()));

pub fn init_logs(dbg_tx: UartTx<'static, esp_hal::Blocking>) {
    LOGGER.lock(|logger| logger.replace(Some(dbg_tx)));
}

// Global defmt logger configuration
#[defmt::global_logger]
struct Logger;

impl Logger {
    fn do_write(bytes: &[u8]) {
        LOGGER.lock(|logger_opt| {
            if let Some(mut logger) = logger_opt.borrow_mut().as_mut() {
                let _ = logger.write(bytes);
            }
        });
    }
}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        ENCODER.lock(|encoder| {
            // (https://github.com/esp-rs/esp-hal/blob/ac5aab0353d084653173fa0870055af7ce2490c4/esp-println/src/defmt.rs#L42-L45)
            // If not disabled, write a non-UTF8 sequence to indicate the start of a defmt
            // frame. We need this to distinguish defmt frames from other data that
            // might be written to the printer.
            Logger::do_write(&[0xFF, 0x00]);

            encoder.borrow_mut().start_frame(Logger::do_write);
        });
    }

    unsafe fn flush() {
        LOGGER.lock(|logger_opt| {
            if let Some(logger) = logger_opt.borrow_mut().as_mut() {
                let _ = logger.flush();
            }
        });
    }

    unsafe fn release() {
        ENCODER.lock(|encoder| {
            encoder.borrow_mut().end_frame(Logger::do_write);
        });
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.lock(|encoder| {
            encoder.borrow_mut().write(bytes, Logger::do_write);
        });
    }
}
