use bleps::{
    ad_structure::{
        create_advertising_data,
        AdStructure,
        BR_EDR_NOT_SUPPORTED,
        LE_GENERAL_DISCOVERABLE,
    },
    async_attribute_server::AttributeServer,
    asynch::Ble,
    attribute_server::NotificationData,
    gatt,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pipe::{Reader, Writer},
};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    rng::Rng,
    time,
    timer::timg::TimerGroup,
};
use defmt::info;
use esp_wifi::{ble::controller::BleConnector, init, EspWifiController};
use embedded_io_async::{Read, Write};
// use esp_println as _;

pub const PIPE_CAPACITY: usize = 256;


// BLE Serial Structure implementing embedded_io_async traits
pub struct BleSerial {
    writer: Writer<'static, CriticalSectionRawMutex, PIPE_CAPACITY>,
    reader: Reader<'static,CriticalSectionRawMutex, PIPE_CAPACITY>,
}

impl BleSerial {
    pub fn new(
        writer: Writer<'static, CriticalSectionRawMutex, PIPE_CAPACITY>,
        reader: Reader<'static,CriticalSectionRawMutex, PIPE_CAPACITY>,
    ) -> Self {
        BleSerial {
            writer,
            reader,
        }
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


// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

pub struct BleServer<'a> {
    ble: Ble<BleConnector<'a>>,
    reader: Reader<'static, CriticalSectionRawMutex, PIPE_CAPACITY>,
    writer: Writer<'static, CriticalSectionRawMutex, PIPE_CAPACITY>,
}

impl<'a> BleServer<'a>{
    pub fn new(
        bluetooth: esp_hal::peripherals::BT,
        tg0: esp_hal::peripherals::TIMG0,
        rng: esp_hal::peripherals::RNG,
        radio_clk: esp_hal::peripherals::RADIO_CLK,
        reader: Reader<'static, CriticalSectionRawMutex, PIPE_CAPACITY>,
        writer: Writer<'static, CriticalSectionRawMutex, PIPE_CAPACITY>,
    ) -> Self {
        let timg0 = TimerGroup::new(tg0);

        let init = &*mk_static!(
            EspWifiController<'static>,
            init(
                timg0.timer0,
                Rng::new(rng),
                radio_clk,
            )
            .unwrap()
        );

        let connector = BleConnector::new(&init, bluetooth);

        let now = || time::now().duration_since_epoch().to_millis();
        let ble = Ble::new(connector, now);
        info!("Connector created");

        Self { ble, reader, writer }
    }

    pub async fn run(&mut self) {
        loop {
            self.ble.init().await;
            self.ble.cmd_set_le_advertising_parameters().await;
            self.ble.cmd_set_le_advertising_data(
                create_advertising_data(&[
                    AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                    AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                    AdStructure::CompleteLocalName(esp_hal::chip!()),
                ])
                .unwrap()
            )
            .await;
            self.ble.cmd_set_le_advertise_enable(true).await;

            info!("started advertising");

            let mut tx_read = |_offset: usize, data: &mut [u8]| {
                0
            };
            let mut rx_write = |offset: usize, data: &[u8]| {
                self.writer.try_write(&data[offset..]);
            };
        
            gatt!([service {
                uuid: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E",
                characteristics: [
                    characteristic {
                        uuid: "6E400002-B5A3-F393-E0A9-E50E24DCCA9E",
                        write: rx_write,
                    },
                    characteristic {
                        name: "tx_characteristic",
                        uuid: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E",
                        notify: true,
                        read: tx_read,
                    },
                ],
            },]);


            let mut rng = bleps::no_rng::NoRng;
            let mut srv = AttributeServer::new(&mut self.ble, &mut gatt_attributes, &mut rng);

            let mut notifier = || {
                async {

                    let mut buffer: [u8;PIPE_CAPACITY] = [0;PIPE_CAPACITY];
                    let len = self.reader.read(&mut buffer).await;

                    NotificationData::new(tx_characteristic_handle, &buffer[..len])
                }
            };

            srv.run(&mut notifier).await.unwrap();
        }
    }
}
