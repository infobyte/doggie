use embassy_rp::{
    flash::Async,
    peripherals::{DMA_CH0, FLASH},
};

const FLASH_SIZE: usize = 2 * 1024 * 1024;

fn get_serial_number(flash: FLASH, dma: DMA_CH0) -> u64 {
    let mut flash = embassy_rp::flash::Flash::<_, Async, FLASH_SIZE>::new(flash, dma);
    // Get unique id
    let mut uid = [0; 8];
    flash.blocking_unique_id(&mut uid).unwrap();

    u64::from_be_bytes(uid)
}

fn u64_to_str(mut raw_number: u64) -> &'static str {
    // A static buffer to hold the string
    static mut BUF: [u8; 20] = [0; 20]; // 20 bytes is enough to hold the max length of u64
    let mut digits: [u8; 20] = [b'0'; 20];

    if raw_number == 0 {
        unsafe {
            BUF[0] = b'0';
        }
        return unsafe { core::str::from_utf8_unchecked(&BUF[0..1]) };
    }

    let mut pos = 0;
    while raw_number > 0 && pos < 20 {
        digits[pos] = b'0' + (raw_number % 10) as u8;
        raw_number /= 10;
        pos += 1;
    }

    for i in 0..pos {
        unsafe {
            BUF[pos - 1 - i] = digits[i];
        }
    }

    // Return the slice that points to the start of the number string
    unsafe { core::str::from_utf8_unchecked(&BUF[0..pos]) }
}

pub fn serial_number(flash: FLASH, dma: DMA_CH0) -> &'static str {
    let raw_serial_number = get_serial_number(flash, dma);
    u64_to_str(raw_serial_number)
}
