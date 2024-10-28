#![no_std]

mod bsp;
mod can;
mod macros;
mod mcp2515;
mod types;

pub use bsp::Bsp;
pub use can::CanDevice;
use mcp2515::can_speed_from_raw;
pub use types::*;

use slcan::{SlcanCommand, SlcanError};

use defmt::{error, info, println};

use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_futures::select::Either;
use embassy_futures::yield_now;

use embedded_can::{blocking::Can, Frame, Id, StandardId};
use embedded_io_async::{Read, Write};

pub struct Core<CAN, SERIAL>
where
    CAN: CanDevice,
    SERIAL: Read + Write,
{
    pub bsp: Bsp<CAN, SERIAL>,
    pub spawner: Spawner,
}

impl<CAN, SERIAL> Core<CAN, SERIAL>
where
    CAN: CanDevice,
    SERIAL: Read + Write,
{
    pub fn new(spawner: Spawner, bsp: Bsp<CAN, SERIAL>) -> Self {
        Core { bsp, spawner }
    }

    pub async fn slcan_task(
        mut serial: SERIAL,
        in_channel: CanChannelReceiver,
        out_channel: CanChannelSender,
    ) {
        let mut serial_in_buf: [u8; 64] = [0; 64];
        // let mut serial_out_buf: [u8; 64] = [0; 64]; // This is not used because slcan doesn't
        // need it now, but i hope in the future
        let mut slcan_serializer = slcan::SlcanSerializer::new();

        loop {
            let serial_future = serial.read(&mut serial_in_buf);
            let can_future = in_channel.receive();

            // This will wait for only one future to finish and drop the other one
            // So, in a loop it should work.
            // TODO: Check if no packets are dropped
            match select(serial_future, can_future).await {
                // n bytes has ben received from serial
                Either::First(serial_recv_size) => {
                    let size = serial_recv_size.unwrap();
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
                        }
                        Err(SlcanError::InvalidCommand) => {
                            // Do nothing too
                            error!("InvalidMessage");
                        }
                        Err(_) => {
                            // TODO: Complete all the cases
                        }
                    };
                }

                Either::Second(can_cmd) => {
                    match can_cmd {
                        SlcanCommand::Frame(frame) => {
                            // Serialize and send the frame
                            if let Some(buffer) =
                                slcan_serializer.to_bytes(SlcanCommand::Frame(frame))
                            {
                                serial.write(&buffer).await.unwrap();
                            }
                        }
                        _ => {
                            // We are not expecting other message
                        }
                    }
                }
            };
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
                    info!("New frame received");
                    let new_frame =
                        slcan::CanFrame::new(frame.id(), frame.is_remote_frame(), frame.data())
                            .unwrap();

                    out_channel.send(SlcanCommand::Frame(new_frame)).await;
                }
                Err(_) => {
                    // Do nothing for now, but it should log
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
                    SlcanCommand::FilterId(id) => can.set_filter(id),
                    SlcanCommand::FilterMask(mask) => can.set_filter(mask),
                    SlcanCommand::SetBitrate(bitrate) => {
                        can.set_bitrate(can::CanBitrates::from(bitrate as u16))
                    }
                    SlcanCommand::SetBitTimeRegister(_) => {
                        // TODO: Implement
                    }
                    _ => {
                        // We don't expect other message type
                    }
                }
            }

            // Avoid starbation
            yield_now().await;
        }
    }
}
