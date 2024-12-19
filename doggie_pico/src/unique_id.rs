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

fn u64_to_str(mut n: u64) -> &'static str {
    // A static buffer to hold the string
    static mut BUF: [u8; 20] = [0; 20]; // 20 bytes is enough to hold the max length of u64
    let mut i = 19;

    // Handle the case when n is 0
    if n == 0 {
        unsafe {
            BUF[19] = b'0';
        }
        return unsafe { core::str::from_utf8_unchecked(&BUF[19..20]) };
    }

    // Convert the number to a string in reverse order
    while n > 0 {
        let digit = (n % 10) as u8 + b'0';
        unsafe {
            BUF[i] = digit;
        }
        i -= 1;
        n /= 10;
    }

    // Return the slice that points to the start of the number string
    unsafe { core::str::from_utf8_unchecked(&BUF[i + 1..20]) }
}

pub fn serial_number(flash: FLASH, dma: DMA_CH0) -> &'static str {
    u64_to_str(get_serial_number(flash, dma))
}
