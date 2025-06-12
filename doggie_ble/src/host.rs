use defmt::{error, info, Debug2Format};
use embassy_futures::select::{select, Either};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pipe::{Reader, Writer},
};
use embedded_io_async::{Read, Write};
use trouble_host::prelude::*;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 3; // Signal + att + CoC
pub const L2CAP_MTU: usize = 255;

// BLE Serial Structure implementing embedded_io_async traits
pub struct BleSerial {
    writer: Writer<'static, CriticalSectionRawMutex, L2CAP_MTU>,
    reader: Reader<'static, CriticalSectionRawMutex, L2CAP_MTU>,
}

impl BleSerial {
    pub fn new(
        writer: Writer<'static, CriticalSectionRawMutex, L2CAP_MTU>,
        reader: Reader<'static, CriticalSectionRawMutex, L2CAP_MTU>,
    ) -> Self {
        BleSerial { writer, reader }
    }
}

impl embedded_io_async::ErrorType for BleSerial {
    type Error = core::convert::Infallible;
}

impl Read for BleSerial {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(self.reader.read(buf).await)
    }
}

impl Write for BleSerial {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(self.writer.write(buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct BleServer {
    reader: Option<Reader<'static, CriticalSectionRawMutex, L2CAP_MTU>>,
    writer: Option<Writer<'static, CriticalSectionRawMutex, L2CAP_MTU>>,
}

impl BleServer {
    pub fn new(
        reader: Reader<'static, CriticalSectionRawMutex, L2CAP_MTU>,
        writer: Writer<'static, CriticalSectionRawMutex, L2CAP_MTU>,
    ) -> Self {
        Self {
            reader: Some(reader),
            writer: Some(writer),
        }
    }

    async fn tx_task<'a, C: Controller>(
        ch_writter: &mut L2capChannelWriter<'a>,
        reader: &mut Reader<'a, CriticalSectionRawMutex, L2CAP_MTU>,
        stack: &Stack<'a, C>,
    ) -> Result<(), ()> {
        let mut buffer = [0; L2CAP_MTU];

        loop {
            match reader.read(&mut buffer).await {
                Err(e) => {
                    error!("[BLE] tx_task: Error reading: {}", e);
                    return Err(());
                }
                Ok(size) => match ch_writter
                    .send::<C, L2CAP_MTU>(stack, &buffer[0..size])
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

    async fn rx_task<'a, C: Controller>(
        ch_reader: &mut L2capChannelReader<'a>,
        writer: &mut Writer<'a, CriticalSectionRawMutex, L2CAP_MTU>,

        stack: &Stack<'a, C>,
    ) -> Result<(), ()> {
        let mut buffer = [0; L2CAP_MTU];

        loop {
            match ch_reader.receive(stack, &mut buffer).await {
                Err(e) => {
                    error!("[BLE] tx_task: Error reading: {}", Debug2Format(&e));
                    return Err(());
                }
                Ok(size) => match writer.write_all(&buffer[0..size]).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("[BLE] tx_task: Error writing: {}", e);
                    }
                },
            }
        }
    }

    pub async fn run<C: Controller>(&mut self, controller: C) {
        // Hardcoded peripheral address
        let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
        info!("Our address = {:?}", address);

        let mut resources: HostResources<CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU> =
            HostResources::new();
        let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
        let Host {
            mut peripheral,
            mut runner,
            ..
        } = stack.build();

        let mut adv_data = [0; 31];
        AdStructure::encode_slice(
            &[AdStructure::Flags(
                LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED,
            )],
            &mut adv_data[..],
        )
        .unwrap();

        let mut scan_data = [0; 31];
        AdStructure::encode_slice(
            &[AdStructure::CompleteLocalName(b"Doggie")],
            &mut scan_data[..],
        )
        .unwrap();

        let mut reader = self.reader.take().unwrap();
        let mut writer = self.writer.take().unwrap();

        match select(runner.run(), async {
            loop {
                info!("Advertising, waiting for connection...");
                let advertiser = peripheral
                    .advertise(
                        &Default::default(),
                        Advertisement::ConnectableScannableUndirected {
                            adv_data: &adv_data[..],
                            scan_data: &scan_data[..],
                        },
                    )
                    .await
                    .unwrap();
                let conn = advertiser.accept().await.unwrap();

                info!("Connection established");

                let ch1 = L2capChannel::accept(&stack, &conn, &[123], &Default::default())
                    .await
                    .unwrap();

                info!("L2CAP channel accepted");

                let (mut ch_writer, mut ch_reader) = ch1.split();

                match select(
                    Self::tx_task(&mut ch_writer, &mut reader, &stack),
                    Self::rx_task(&mut ch_reader, &mut writer, &stack),
                )
                .await
                {
                    Either::First(_) => error!("[BLE] TX task exited"),
                    Either::Second(_) => error!("[BLE] RX task exited"),
                }
            }
        })
        .await
        {
            Either::First(res) => match res {
                Ok(_) => error!("[BLE] Runner exited without error"),
                Err(e) => error!("[BLE] Runner exited with error: {}", Debug2Format(&e)),
            },
            Either::Second(_) => {
                error!("[BLE] IO Task exited without error");
            }
        }
    }
}
