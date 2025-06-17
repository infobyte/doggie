#![no_std]
#![no_main]

pub mod constants;
mod serial;
mod server;
pub mod types;

pub use serial::BleSerial;
pub use server::BleServer;
