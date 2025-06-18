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

use static_cell::StaticCell;

static TX_PIPE: StaticCell<types::BlePipe> = StaticCell::new();
static RX_PIPE: StaticCell<types::BlePipe> = StaticCell::new();

pub fn create_ble_pipe() -> (BleServer, BleSerial) {
    let tx_pipe = TX_PIPE.init(types::BlePipe::new());
    let rx_pipe = RX_PIPE.init(types::BlePipe::new());

    let (ble_tx_reader, ble_tx_writer) = tx_pipe.split();
    let (ble_rx_reader, ble_rx_writer) = rx_pipe.split();

    let ble_server = BleServer::new(ble_tx_reader, ble_rx_writer);
    let ble_serial = BleSerial::new(ble_tx_writer, ble_rx_reader);

    (ble_server, ble_serial)
}
