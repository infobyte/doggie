#![no_std]

mod bsp;
mod can;
mod macros;
mod mcp2515;
mod types;

use core::time;

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
use embassy_time::Instant;

// Struct to hold timestamp functionality
pub struct Timestamp {
    start: Option<Instant>,
}

impl Timestamp {
    // Initialize the timestamp with the current time as the start
    pub fn new() -> Self {
        Self {
            start: None,
        }
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    // Get the elapsed time in milliseconds since initialization
    pub fn get_current(&self) -> Option<u16> {
        match self.start {
            Some(instant) => Some(instant.elapsed().as_micros() as u16),
            None => None
        }
    }
}

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

        let mut listen_only = false;
        let mut timestamp_enabled = true;
        let mut timestamp = Timestamp::new();

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
                        Ok(SlcanCommand::IncompleteMessage) => {
                            // Do nothing
                        }
                        Ok(SlcanCommand::OpenChannel) => {
                            serial.write(b"\r").await.unwrap();
                        },
                        Ok(SlcanCommand::CloseChannel) => {
                            serial.write(b"\r").await.unwrap();
                        },
                        Ok(SlcanCommand::ReadStatusFlags) => {
                            serial.write(b"F00\r").await.unwrap();
                        },
                        Ok(SlcanCommand::Listen) => {
                            listen_only = true;
                            serial.write(b"\r").await.unwrap();
                        },
                        Ok(SlcanCommand::Version) => {
                            serial.write(b"V1337\r").await.unwrap();
                        },
                        Ok(SlcanCommand::SerialNo) => {
                            serial.write(b"N1337\r").await.unwrap();
                        },
                        Ok(SlcanCommand::Timestamp(enabled)) => {
                            if !timestamp_enabled && enabled {
                                timestamp.start();
                            }
                            
                            timestamp_enabled = enabled;
                            
                            serial.write(b"\r").await.unwrap();
                        },
                        Ok(cmd) => {
                            if !listen_only {
                                out_channel.send(cmd).await;
                            } else {
                                error!("Cannot send frame in listen only mode")
                            }
                        }
                        Err(e) => {
                            match e {
                                SlcanError::InvalidCommand => error!("Invalid slcan command"),
                                SlcanError::CommandNotImplemented => error!("Command not implemented"),
                                SlcanError::MessageTooLong => error!("Command to long")
                            }
                        }
                    };
                }

                Either::Second(can_cmd) => {
                    match can_cmd {
                        SlcanCommand::Frame(mut frame) => {
                            // Serialize and send the frame
                            if timestamp_enabled {
                                frame.timestamp = timestamp.get_current();
                            }
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
