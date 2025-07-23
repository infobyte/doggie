use doggie_core::{CanBitrates, CanDevice};
use embedded_can::{blocking::Can, Id};
use esp_hal::{
    gpio::GpioPin,
    peripherals,
    twai::{self, filter::SingleStandardFilter, Twai, TwaiMode},
    Blocking,
};
use nb::Error;
use defmt::info;

const MAX_TRIES: usize = 10;

pub struct CanWrapper<'d> {
    can_opt: Option<Twai<'d, Blocking>>,
    baudrate: twai::BaudRate,
}

impl<'d> CanWrapper<'d> {
    pub fn new() -> Self {
        const TWAI_BAUDRATE: twai::BaudRate = twai::BaudRate::B250K;

        let mut instance = CanWrapper {
            can_opt: None,
            baudrate: TWAI_BAUDRATE,
        };

        instance.init();

        instance
    }

    fn create_twai_config(new_bitrate: twai::BaudRate) -> twai::TwaiConfiguration<'d, Blocking> {
        let mut twai_config = unsafe {
            #[cfg(feature = "esp32c3")]
            {
                twai::TwaiConfiguration::new(
                    peripherals::TWAI0::steal(),
                    GpioPin::<0>::steal(),
                    GpioPin::<1>::steal(),
                    new_bitrate,
                    TwaiMode::Normal,
                )
            }
            #[cfg(not(feature = "esp32c3"))]
            {
                twai::TwaiConfiguration::new(
                    peripherals::TWAI0::steal(),
                    GpioPin::<25>::steal(),
                    GpioPin::<26>::steal(),
                    new_bitrate,
                    TwaiMode::Normal,
                )
            }
        };
        twai_config.set_filter(
            const { SingleStandardFilter::new(b"xxxxxxxxxxx", b"x", [b"xxxxxxxx", b"xxxxxxxx"]) },
        );
        twai_config
    }

    fn init(&mut self) {
        match self.can_opt.take() {
            None => {},
            Some(can) => {
                info!(
                    "Recovering TWAI: \
                    \n\tReceive error cnt: {} \
                    \n\tTransmit error cnt: {} \
                    \n\tIs bus-off: {} \
                    \n\tNum available msgs: {}",
                    can.receive_error_count(),
                    can.transmit_error_count(),
                    can.is_bus_off(),
                    can.num_available_messages(),
                );
                can.stop();
            }
        }

        let twai_config = Self::create_twai_config(self.baudrate);

        let can = twai_config.start();

        can.clear_receive_fifo();

        // Start the peripheral. This locks the configuration settings of the peripheral
        // and puts it into operation mode, allowing packets to be sent and
        // received.
        self.can_opt.replace(can);
    }
}

impl<'d> Can for CanWrapper<'d> {
    type Frame = <Twai<'d, Blocking> as embedded_can::nb::Can>::Frame;
    type Error = <Twai<'d, Blocking> as embedded_can::nb::Can>::Error;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        let must_init = match self.can_opt {
            Some(ref can) => {
                can.is_bus_off()
            },
            None => true,
        };

        if must_init {
            self.init();
        }

        match self.can_opt {
            Some(ref mut can) => {
                let mut count = 0;

                loop {
                    match can.transmit(frame) {
                        Ok(_) => return Ok(()),
                        Err(Error::WouldBlock) => {}
                        Err(Error::Other(e)) => return Err(e),
                    }
                    count += 1;

                    if count >= MAX_TRIES {
                        return Err(Self::Error::BusOff);
                    }
                }
            }
            None => Err(Self::Error::BusOff),
        }
    }

    fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        match self.can_opt {
            Some(ref mut can) => {
                let mut count = 0;

                loop {
                    match can.receive() {
                        Ok(frame) => return Ok(frame),
                        Err(Error::WouldBlock) => {}
                        Err(Error::Other(e)) => return Err(e),
                    }
                    count += 1;

                    if count >= MAX_TRIES {
                        return Err(Self::Error::BusOff);
                    }
                }
            }
            None => Err(Self::Error::BusOff),
        }
    }
}

impl<'d> CanDevice for CanWrapper<'d> {
    fn set_bitrate(&mut self, bitrate: CanBitrates) {
        let new_bitrate = match bitrate {
            CanBitrates::Kbps125 => twai::BaudRate::B125K,
            CanBitrates::Kbps250 => twai::BaudRate::B250K,
            CanBitrates::Kbps500 => twai::BaudRate::B500K,
            CanBitrates::Kbps1000 => twai::BaudRate::B1000K,
            _ => twai::BaudRate::B500K,
        };

        self.baudrate = new_bitrate;

        self.init();
    }

    fn set_filter(&mut self, _id: Id) {
        // TODO
    }

    fn set_mask(&mut self, _id: Id) {
        // TODO
    }
}
