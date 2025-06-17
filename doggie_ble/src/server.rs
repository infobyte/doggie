use defmt::{debug, error, info, warn, Debug2Format};
use embassy_futures::join::join;
use embassy_futures::select::select;
use embedded_io_async::{Read, Write};
use heapless::Vec;
use trouble_host::prelude::*;

use crate::constants::MAX_CMD_LEN;
use crate::types::{BlePipeReader, BlePipeWriter};

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att
const L2CAP_MTU: usize = 251;

// GATT Server definition
#[gatt_server]
struct Server {
    nus_service: NordicUartService,
}

/// Nordic UART Service
#[gatt_service(uuid = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E")]
struct NordicUartService {
    /// TX Characteristic - used to send data to central (notify)
    #[characteristic(uuid = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E", notify, read)]
    tx: Vec<u8, MAX_CMD_LEN>, // Maximum BLE packet size for data

    /// RX Characteristic - used to receive data from central (write)
    #[characteristic(
        uuid = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E",
        write,
        write_without_response
    )]
    rx: Vec<u8, MAX_CMD_LEN>,
}

pub struct BleServer {
    reader: Option<BlePipeReader>,
    writer: Option<BlePipeWriter>,
}

impl BleServer {
    pub fn new(reader: BlePipeReader, writer: BlePipeWriter) -> Self {
        Self {
            reader: Some(reader),
            writer: Some(writer),
        }
    }

    /// Run the BLE stack.
    pub async fn run<C>(&mut self, controller: C)
    where
        C: Controller,
    {
        let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
        info!("[BLE] Our address = {:?}", address);

        let mut resources: HostResources<CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU> =
            HostResources::new();
        let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
        let Host {
            mut peripheral,
            runner,
            ..
        } = stack.build();

        info!("[BLE] Starting advertising and GATT service");
        let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
            name: "Doggie BLE",
            appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
        }))
        .unwrap();

        // This should not faild
        let mut reader = self.reader.take().unwrap();
        let mut writer = self.writer.take().unwrap();

        let _ = join(Self::ble_task(runner), async {
            loop {
                match Self::advertise("Doggie BLE", &mut peripheral, &server).await {
                    Ok(conn) => {
                        // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                        let a = Self::gatt_events_task(&server, &conn, &mut writer);
                        let b = Self::custom_task::<C>(&server, &conn, &mut reader);
                        // run until any task ends (usually because the connection has been closed),
                        // then return to advertising state.
                        select(a, b).await;
                    }
                    Err(e) => {
                        let e = defmt::Debug2Format(&e);
                        panic!("[BLE | adv] error: {:?}", e);
                    }
                }
            }
        })
        .await;
    }

    /// This is a background task that is required to run forever alongside any other BLE tasks.
    async fn ble_task<C: Controller>(mut runner: Runner<'_, C>) {
        loop {
            if let Err(e) = runner.run().await {
                let e = defmt::Debug2Format(&e);
                error!("{}", e);
                panic!("[BLE | ble_task] error: {:?}", e);
            }
        }
    }

    async fn gatt_events_task(
        server: &Server<'_>,
        conn: &GattConnection<'_, '_>,
        writer: &mut BlePipeWriter,
    ) -> Result<(), Error> {
        let rx_char = &server.nus_service.rx;
        let tx_char = &server.nus_service.tx;

        loop {
            match conn.next().await {
                GattConnectionEvent::Disconnected { reason } => {
                    info!("[BLE | gatt] disconnected: {:?}", reason);
                    break;
                }
                GattConnectionEvent::Gatt { event } => match event {
                    Ok(event) => {
                        match &event {
                            GattEvent::Read(event) => {
                                if event.handle() == tx_char.handle {
                                    warn!("[BLE | nus] Read request on TX characteristic");
                                } else if event.handle() == rx_char.handle {
                                    warn!("[BLE | nus] Read request on RX characteristic");
                                }
                            }
                            GattEvent::Write(event) => {
                                if event.handle() == rx_char.handle {
                                    let data = event.data();
                                    debug!("[BLE | nus] Received data: {:?}", data);

                                    match writer.write_all(data).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("[BLE] tx_task: Error writing: {}", e);
                                        }
                                    }
                                }
                            }
                        }

                        // Send reply
                        match event.accept() {
                            Ok(reply) => {
                                reply.send().await;
                            }
                            Err(e) => warn!("[BLE | gatt] error sending response: {:?}", e),
                        }
                    }
                    Err(e) => warn!("[BLE | gatt] error processing event: {:?}", e),
                },
                _ => {}
            }
        }
        info!("[BLE | gatt] task finished");
        Ok(())
    }

    /// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
    async fn advertise<'a, 'b, C: Controller>(
        name: &'a str,
        peripheral: &mut Peripheral<'a, C>,
        server: &'b Server<'_>,
    ) -> Result<GattConnection<'a, 'b>, BleHostError<C::Error>> {
        let mut advertiser_data = [0; 31];
        AdStructure::encode_slice(
            &[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[[0x0f, 0x18]]),
                AdStructure::CompleteLocalName(name.as_bytes()),
            ],
            &mut advertiser_data[..],
        )?;
        let advertiser = peripheral
            .advertise(
                &Default::default(),
                Advertisement::ConnectableScannableUndirected {
                    adv_data: &advertiser_data[..],
                    scan_data: &[],
                },
            )
            .await?;
        info!("[adv] advertising");
        let conn = advertiser.accept().await?.with_attribute_server(server)?;
        info!("[adv] connection established");
        Ok(conn)
    }

    /// Example task to use the BLE notifier interface.
    /// This task will notify the connected central of a counter value every 2 seconds.
    /// It will also read the RSSI value every 2 seconds.
    /// and will stop when the connection is closed by the central or an error occurs.
    async fn custom_task<C: Controller>(
        server: &Server<'_>,
        conn: &GattConnection<'_, '_>,
        reader: &mut BlePipeReader,
    ) {
        let tx_char = &server.nus_service.tx;
        let mut buffer = [0; MAX_CMD_LEN];

        loop {
            match reader.read(&mut buffer).await {
                Err(e) => {
                    error!("[BLE] tx_task: Error reading: {}", e);
                }
                Ok(size) => match tx_char
                    .notify(conn, &Vec::from_slice(&buffer[0..size]).unwrap())
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        error!("[BLE] tx_task: Error writing: {}", Debug2Format(&e));
                    }
                },
            }
        }
    }
}
