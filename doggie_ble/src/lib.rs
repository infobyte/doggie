#![no_std]
#![no_main]

pub mod constants;
mod serial;
mod serial_mux;
mod server;
pub mod types;

pub use serial::BleSerial;
pub use serial_mux::{SerialMux, SerialMuxError};
pub use server::BleServer;
