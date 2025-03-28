#![no_std]

mod bsp;
mod can;
mod macros;
mod mcp2515;
mod types;

pub use bsp::Bsp;
pub use can::{CanBitrates, CanDevice};
use defmt::warn;
use embedded_can::Error;
use embedded_can::ErrorKind;
pub use types::*;

use slcan::{SlcanCommand, SlcanError};

use defmt::{debug, error, info};

use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_futures::select::Either;
use embassy_futures::yield_now;

use embassy_time::Instant;
use embedded_can::Frame;
use embedded_io_async::{Read, Write};

// Struct to hold timestamp functionality
pub struct Timestamp {
    start: Option<Instant>,
}

impl Timestamp {
    // Initialize the timestamp with the current time as the start
    pub fn new() -> Self {
        Self { start: None }
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    // Get the elapsed time in milliseconds since initialization
    pub fn get_current(&self) -> Option<u16> {
        match self.start {
            Some(instant) => Some(instant.elapsed().as_micros() as u16),
            None => None,
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
    ) -> ! {
        let mut serial_in_buf: [u8; 256] = [0; 256];
        // let mut serial_out_buf: [u8; 64] = [0; 64]; // This is not used because slcan doesn't
        // need it now, but i hope in the future
        let mut slcan_serializer = slcan::SlcanSerializer::new();

        let mut listen_only = false;
        let mut timestamp_enabled = false;
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
                    let size = match serial_recv_size {
                        Ok(size) => size,
                        Err(_) => {
                            error!("Error reading from serial");
                            continue;
                        }
                    };

                    for byte in &serial_in_buf[0..size] {
                        let res: Option<&[u8]> = match slcan_serializer.from_byte(*byte) {
                            Ok(SlcanCommand::IncompleteMessage) => {
                                // Do nothing
                                // warn!("IncompleteMessage");
                                None
                            }
                            Ok(SlcanCommand::OpenChannel) => Some(b"\r"),
                            Ok(SlcanCommand::CloseChannel) => Some(b"\r"),
                            Ok(SlcanCommand::ReadStatusFlags) => Some(b"F00\r"),
                            Ok(SlcanCommand::Listen) => {
                                listen_only = true;
                                Some(b"\r")
                            }
                            Ok(SlcanCommand::Version) => Some(b"V1337\r"),
                            Ok(SlcanCommand::SerialNo) => Some(b"N1337\r"),
                            Ok(SlcanCommand::Timestamp(enabled)) => {
                                if !timestamp_enabled && enabled {
                                    info!("Timestamp started");
                                    timestamp.start();
                                }

                                timestamp_enabled = enabled;

                                Some(b"\r")
                            }
                            Ok(cmd) => {
                                if !listen_only {
                                    out_channel.send(cmd).await;
                                } else {
                                    error!("Cannot send frame in listen only mode")
                                }

                                None
                            }
                            Err(e) => {
                                match e {
                                    SlcanError::InvalidCommand => error!("Invalid slcan command"),
                                    SlcanError::CommandNotImplemented => {
                                        error!("Command not implemented")
                                    }
                                    SlcanError::MessageTooLong => error!("Command to long"),
                                };
                                None
                            }
                        };

                        match res {
                            Some(res_str) => match serial.write(res_str).await {
                                Ok(_) => {}
                                Err(_) => error!("Error writing response"),
                            },
                            None => {}
                        };
                    }
                }

                Either::Second(can_cmd) => {
                    match can_cmd {
                        SlcanCommand::Frame(mut frame) => {
                            // Serialize and send the frame
                            if timestamp_enabled {
                                frame.timestamp = timestamp.get_current();
                            }
                            if let Some((buffer, size)) =
                                slcan_serializer.to_bytes(SlcanCommand::Frame(frame))
                            {
                                let mut start = 0;
                                while start != size {
                                    start += match serial.write(&buffer[start..size]).await {
                                        Ok(size) => size,
                                        Err(_) => {
                                            error!("Error writing to serial, up to retry");
                                            0
                                        }
                                    }
                                }
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
    ) -> ! {
        info!("Init: can_task");
        loop {
            // Try to receive a message
            match can.receive() {
                Ok(frame) => {
                    debug!("New frame received");
                    let new_frame = slcan::CanFrame::new(
                        frame.id(),
                        frame.is_remote_frame(),
                        &frame.data()[0..frame.dlc()],
                    )
                    .unwrap();

                    out_channel.send(SlcanCommand::Frame(new_frame)).await;
                }
                Err(e) => match e.kind() {
                    ErrorKind::Overrun => {
                        error!("Overrun error received from CAN controller");
                    }
                    _ => {}
                },
            }

            if !in_channel.is_empty() {
                match in_channel.receive().await {
                    SlcanCommand::Frame(frame) => {
                        debug!("Sending new frame");

                        let new_frame =
                            CAN::Frame::new(frame.id, &frame.data[0..frame.dlc]).unwrap();

                        while let Err(e) = can.transmit(&new_frame) {
                            match e.kind() {
                                ErrorKind::Overrun => {
                                    error!("Overrun error received from CAN controller");
                                }
                                ErrorKind::Other => {
                                    error!("Other error received from CAN controller");
                                }
                                _ => {
                                    error!("Transmition failed, up to retry");
                                }
                            };

                            yield_now().await;
                        }
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
                        warn!("SlcanCommand not supported");
                    }
                }
            }

            // Avoid starvation
            yield_now().await;
        }
    }
}
