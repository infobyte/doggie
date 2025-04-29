use doggie_core::{CanBitrates, CanDevice};
use embedded_can::{blocking::Can, Id};
use esp_hal::{
    gpio::GpioPin,
    peripherals,
    twai::{self, filter::SingleStandardFilter, Twai, TwaiMode},
    Blocking,
};
use nb::Error;

const MAX_TRIES: usize = 10;

pub struct CanWrapper<'d> {
    can_opt: Option<Twai<'d, Blocking>>,
}

impl<'d> CanWrapper<'d> {
    pub fn new(can: peripherals::TWAI0, rx_pin: GpioPin<0>, tx_pin: GpioPin<1>) -> Self {
        const TWAI_BAUDRATE: twai::BaudRate = twai::BaudRate::B250K;

        let mut twai_config =
            twai::TwaiConfiguration::new(can, rx_pin, tx_pin, TWAI_BAUDRATE, TwaiMode::Normal);

        twai_config.set_filter(
            const { SingleStandardFilter::new(b"xxxxxxxxxxx", b"x", [b"xxxxxxxx", b"xxxxxxxx"]) },
        );

        // Start the peripheral. This locks the configuration settings of the peripheral
        // and puts it into operation mode, allowing packets to be sent and
        // received.
        let twai = twai_config.start();

        CanWrapper {
            can_opt: Some(twai),
        }
    }
}

impl<'d> Can for CanWrapper<'d> {
    type Frame = <Twai<'d, Blocking> as embedded_can::nb::Can>::Frame;
    type Error = <Twai<'d, Blocking> as embedded_can::nb::Can>::Error;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
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

        let old_can = self.can_opt.take().unwrap();
        old_can.stop();

        unsafe {
            let mut twai_config = twai::TwaiConfiguration::new(
                peripherals::TWAI0::steal(),
                GpioPin::<0>::steal(),
                GpioPin::<1>::steal(),
                new_bitrate,
                TwaiMode::Normal,
            );

            twai_config.set_filter(
                const { SingleStandardFilter::new(b"xxxxxxxxxxx", b"x", [b"xxxxxxxx", b"xxxxxxxx"]) },
            );

            // Start the peripheral. This locks the configuration settings of the peripheral
            // and puts it into operation mode, allowing packets to be sent and
            // received.
            self.can_opt.replace(twai_config.start());
        }
    }

    fn set_filter(&mut self, _id: Id) {
        // TODO
    }

    fn set_mask(&mut self, _id: Id) {
        // TODO
    }
}
