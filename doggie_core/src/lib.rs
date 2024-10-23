#![no_std]

mod bsp;

pub use bsp::Bsp;

use embassy_time::Timer;
use embedded_can::{blocking::Can, ExtendedId, Frame, Id, StandardId};
// use embedded_io::{Read, Write};

// pub struct Core<CAN, SERIAL>
// where
//     CAN: Can,
//     SERIAL: Read + Write,
// {
//     pub bsp: Bsp<CAN, SERIAL>,
// }
//
// impl<CAN, SERIAL> Core<CAN, SERIAL>
// where
//     CAN: Can,
//     SERIAL: Read + Write,
// {
//     pub fn new(bsp: Bsp<CAN, SERIAL>) -> Self {
//         Core { bsp }
//     }
//
//     pub async fn run(self) {}
// }

pub struct Core<CAN>
where
    CAN: Can,
{
    pub bsp: Bsp<CAN>,
}

impl<CAN> Core<CAN>
where
    CAN: Can,
{
    pub fn new(bsp: Bsp<CAN>) -> Self {
        Core { bsp }
    }

    pub async fn run(&mut self) {
        let mut id: u16 = 0x1FF;

        loop {
            // Send a message

            let frame = CAN::Frame::new(
                Id::Standard(StandardId::new(id).unwrap()),
                &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
            )
            .unwrap();

            match self.bsp.can.transmit(&frame) {
                Ok(_) => {}
                Err(_) => {} // Ok(_) => info!("Sent message!"),
                             // Err(spi_error) => error!("Error sending message"),
            }

            // Read the message back (we are in loopback mode)
            match self.bsp.can.receive() {
                Ok(frame) => {
                    // info!("Frame received");

                    let id = frame.id(); // CAN ID
                    let id_value = match id {
                        embedded_can::Id::Standard(id) => id.as_raw() as u32, // Get raw standard CAN ID
                        embedded_can::Id::Extended(id) => id.as_raw(), // Get raw extended CAN ID
                    };

                    let dlc = frame.dlc(); // Data length code
                    let data = frame.data(); // Data bytes
                    let is_extended = frame.is_extended(); // Check if it's an extended frame

                    // Print the CAN frame details
                    // println!(
                    //     "ID: {:x}, DLC: {}, Data: {:X}, Extended: {}",
                    //     id_value, dlc, data, is_extended
                    // );

                    Timer::after_millis(1000).await;
                }
                Err(_) => {} // Err(Error::NoMessage) => info!("No message to read!"),
                             // Err(_) => error!("Oh no!"),
            }

            Timer::after_millis(1000).await;
        }
    }
}
