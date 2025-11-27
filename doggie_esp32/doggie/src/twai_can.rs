use defmt::{debug, info, warn};
use doggie_core::{CanBitrates, CanDevice};
use embedded_can::{blocking::Can, Id};
#[cfg(feature = "esp32c3")]
use esp_hal::peripherals::{GPIO0, GPIO1};
#[cfg(not(feature = "esp32c3"))]
use esp_hal::peripherals::{GPIO25, GPIO26};
use esp_hal::{
    peripherals,
    twai::{self, filter::SingleStandardFilter, ErrorKind, Twai, TwaiMode},
    Blocking,
};
use nb::Error;

const MAX_TRIES: usize = 10;

pub struct CanWrapper<'d> {
    can_opt: Option<Twai<'d, Blocking>>,
    baudrate: twai::BaudRate,
    mode: TwaiMode,
}

impl<'d> CanWrapper<'d> {
    pub fn new() -> Self {
        const TWAI_BAUDRATE: twai::BaudRate = twai::BaudRate::B250K;
        const TWAI_MODE: TwaiMode = TwaiMode::Normal;

        let instance = CanWrapper {
            can_opt: None,
            baudrate: TWAI_BAUDRATE,
            mode: TWAI_MODE,
        };

        instance
    }

    fn create_twai_config(
        new_bitrate: twai::BaudRate,
        mode: TwaiMode,
    ) -> twai::TwaiConfiguration<'d, Blocking> {
        let mut twai_config = unsafe {
            #[cfg(feature = "esp32c3")]
            {
                twai::TwaiConfiguration::new(
                    peripherals::TWAI0::steal(),
                    GPIO0::steal(),
                    GPIO1::steal(),
                    new_bitrate,
                    mode,
                )
            }
            #[cfg(not(feature = "esp32c3"))]
            {
                twai::TwaiConfiguration::new(
                    peripherals::TWAI0::steal(),
                    GPIO25::steal(),
                    GPIO26::steal(),
                    new_bitrate,
                    mode,
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
            None => {}
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

        let twai_config = Self::create_twai_config(self.baudrate, self.mode);

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
        // Drop if not initialized
        let must_init = match self.can_opt {
            Some(ref can) => can.is_bus_off(),
            None => {
                warn!("Trying to send a message in listen only mode, dropped");
                return Ok(());
            }
        };

        if must_init {
            self.init();
        }

        if let Some(ref mut can) = self.can_opt {
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
        } else {
            Ok(())
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
                        Err(Error::Other(Self::Error::EmbeddedHAL(ErrorKind::Overrun))) => {
                            warn!("TWAI Can Overrun, clearing receive fifo");
                            can.clear_receive_fifo();
                        }
                        Err(Error::Other(e)) => {
                            return Err(e);
                        }
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
    }

    fn set_filter(&mut self, _id: Id) {
        // TODO
    }

    fn set_mask(&mut self, _id: Id) {
        // TODO
    }

    fn open(&mut self) {
        debug!("TWAI can Open");
        self.mode = TwaiMode::Normal;
        self.init();
    }

    fn close(&mut self) {
        debug!("TWAI can Close");
        match self.can_opt.take() {
            Some(can) => {
                can.stop();
            }
            None => {}
        };
    }

    fn listen_only(&mut self) {
        debug!("TWAI can Listen Only mode");
        self.mode = TwaiMode::ListenOnly;
        self.init();
    }
}
