#![no_std]

mod bsp;

pub use bsp::Bsp;

use defmt::{error, info, println};
use embassy_futures::join::join;
use embassy_time::Timer;
use embedded_can::{blocking::Can, Frame, Id, StandardId};
use embedded_io_async::{Read, Write};

use heapless::String;

pub struct Core<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write,
{
    pub bsp: Bsp<CAN, SERIAL>,
}

impl<CAN, SERIAL> Core<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write,
{
    pub fn new(bsp: Bsp<CAN, SERIAL>) -> Self {
        Core { bsp }
    }

    pub async fn run(&mut self) {
        let id: u16 = 0x1FF;

        loop {
            // Send a message

            // let frame = CAN::Frame::new(
            //     Id::Standard(StandardId::new(id).unwrap()),
            //     &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            // )
            // .unwrap();
            //
            // match self.bsp.can.transmit(&frame) {
            //     Ok(_) => info!("Sent message!"),
            //     Err(_) => error!("Error sending message"),
            // }
            //
            // // Read the message back (we are in loopback mode)
            // match self.bsp.can.receive() {
            //     Ok(frame) => {
            //         info!("Frame received");
            //
            //         let id = frame.id(); // CAN ID
            //         let id_value = match id {
            //             embedded_can::Id::Standard(id) => id.as_raw() as u32, // Get raw standard CAN ID
            //             embedded_can::Id::Extended(id) => id.as_raw(), // Get raw extended CAN ID
            //         };
            //
            //         let dlc = frame.dlc(); // Data length code
            //         let data = frame.data(); // Data bytes
            //         let is_extended = frame.is_extended(); // Check if it's an extended frame
            //
            //         // Print the CAN frame details
            //         println!(
            //             "ID: {:x}, DLC: {}, Data: {:X}, Extended: {}",
            //             id_value, dlc, data, is_extended
            //         );
            //
            //         Timer::after_millis(1000).await;
            //     }
            //     Err(_) => info!("No message to read!"),
            // }

            // let mut buf: [u8; 128] = [0 as u8; 128];

            // let r = self.bsp.serial.read(&mut buf).await;

            // println!("{}", buf);

            //     Ok(_) => {
            //         self.bsp.serial.write(&buf).await.unwrap();
            //     }
            //     Err(_) => {}
            // }

            // let f = self.bsp.serial.write_all(DATA.as_bytes()).await;
            // let t = Timer::after_millis(1000);

            // join(f, t).await;

            // Echo
            let mut buf = [0; 64];
            loop {
                let n = self.bsp.serial.read(&mut buf).await.unwrap();

                let data = &buf[..n];
                self.bsp.serial.write(data).await.unwrap();
            }
        }
    }
}
