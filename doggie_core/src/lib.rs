#![no_std]

mod bsp;
mod macros;
mod types;

pub use bsp::Bsp;
pub use types::*;

use slcan::{SlcanCommand, SlcanError};

use defmt::{error, info, println};

use embassy_executor::Spawner;
use embassy_futures::yield_now;

use embedded_can::{blocking::Can, Frame, Id, StandardId};
use embedded_io::{Read, ReadReady, Write};

pub struct Core<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write + ReadReady,
{
    pub bsp: Bsp<CAN, SERIAL>,
    pub spawner: Spawner,
}

impl<CAN, SERIAL> Core<CAN, SERIAL>
where
    CAN: Can,
    SERIAL: Read + Write + ReadReady,
{
    pub fn new(spawner: Spawner, bsp: Bsp<CAN, SERIAL>) -> Self {
        Core { bsp, spawner }
    }

    pub async fn slcan_task(
        mut serial: SERIAL,
        in_channel: CanChannelReceiver,
        out_channel: CanChannelSender,
    ) {
        info!("Starting 'slcan_task'");
        let mut serial_in_buf: [u8; 64] = [0; 64];
        // let mut serial_out_buf: [u8; 64] = [0; 64]; // This is not used because slcan doesn't
        // need it now, but i hope in the future
        let mut slcan_serializer = slcan::SlcanSerializer::new();

        loop {
            // info!("reading from serial");
            // Read from serial
            match serial.read_ready() {
                Ok(true) => {
                    match serial.read(&mut serial_in_buf) {
                        Ok(size) if size > 0 => {
                            match slcan_serializer.from_bytes(&serial_in_buf[0..size]) {
                                Ok(SlcanCommand::Frame(frame)) => {
                                    info!("New frame parsed correctlly");
                                    out_channel.send(SlcanCommand::Frame(frame)).await;
                                }
                                Ok(SlcanCommand::IncompleteMessage) => {
                                    // Do nothing
                                }
                                Ok(_) => {
                                    // TODO: Complete all the cases
                                    info!("OTHER OK")
                                }
                                Err(SlcanError::InvalidCommand) => {
                                    // Do nothing too
                                    error!("InvalidMessage");
                                }
                                Err(_) => {
                                    error!("Another error")
                                    // TODO: Complete all the cases
                                }
                            };
                        }
                        Ok(size) => {
                            // println!("Recv from serial {}", size);
                        }
                        Err(_) => {
                            // error!("Errorn on serial read");
                        }
                    }
                }
                _ => {}
            }
            // Check channel
            // info!("Checking channel");
            if !in_channel.is_empty() {
                match in_channel.receive().await {
                    SlcanCommand::Frame(frame) => {
                        // Serialize and send the frame
                        if let Some(buffer) = slcan_serializer.to_bytes(SlcanCommand::Frame(frame))
                        {
                            let mut index = 0;

                            while index < buffer.len() {
                                if let Ok(written) = serial.write(&buffer[index..]) {
                                    index += written;
                                }
                            }
                        }
                    }
                    _ => {
                        // We are not expecting other message
                    }
                }
            } else {
                // info!("Channel empty");
            }

            yield_now().await;
        }
    }

    pub async fn can_task(
        mut can: CAN,
        in_channel: CanChannelReceiver,
        out_channel: CanChannelSender,
    ) {
        loop {
            // Try to receive a message
            match can.receive() {
                Ok(frame) => {
                    // info!("New frame received");
                    let new_frame =
                        slcan::CanFrame::new(frame.id(), frame.is_remote_frame(), frame.data())
                            .unwrap();

                    out_channel.send(SlcanCommand::Frame(new_frame)).await;
                }
                Err(_) => {
                    // Do nothing for now, but it should log
                    // error!("Error reading can");
                }
            }

            if !in_channel.is_empty() {
                match in_channel.receive().await {
                    SlcanCommand::Frame(frame) => {
                        info!("Sending new frame");

                        let new_frame =
                            CAN::Frame::new(frame.id, &frame.data[0..frame.dlc]).unwrap();

                        can.transmit(&new_frame).unwrap();
                    }
                    _ => {
                        // TODO
                    }
                }
            } else {
                // info!("Can channel empty");
            }

            // Avoid starbation
            yield_now().await;
        }
    }
}
