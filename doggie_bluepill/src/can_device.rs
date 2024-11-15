use doggie_core::{CanBitrates, CanDevice};
use embassy_stm32::can::Can as StmCan;
use embassy_stm32::can::{filter, Fifo, Id};
use embedded_can::{blocking::Can, ErrorKind, ExtendedId, StandardId};

pub struct CanWrapper<'d> {
    can: StmCan<'d>,
}

impl<'d> CanWrapper<'d> {
    pub fn new(can: StmCan<'d>) -> Self {
        CanWrapper { can }
    }
}

#[derive(Debug)]
pub struct CanError {}

impl<'d> embedded_can::Error for CanError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl<'d> Can for CanWrapper<'d> {
    type Frame = embassy_stm32::can::frame::Frame;
    type Error = CanError;

    fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
        match self.can.try_write(frame) {
            Ok(_) => Ok(()),
            Err(_) => Err(CanError {}),
        }
    }
    fn receive(&mut self) -> Result<Self::Frame, Self::Error> {
        match self.can.try_read() {
            Ok(envelope) => Ok(envelope.frame),
            Err(_) => Err(CanError {}),
        }
    }
}

impl<'d> CanDevice for CanWrapper<'d> {
    fn set_bitrate(&mut self, bitrate: CanBitrates) {
        self.can.set_bitrate((bitrate as u32) * 1_000);
    }

    fn set_filter(&mut self, id: Id) {
        self.can.modify_filters().enable_bank(
            0,
            Fifo::Fifo0,
            [
                filter::ListEntry32::data_frames_with_id(id),
                filter::ListEntry32::data_frames_with_id(id),
            ],
        );
    }

    fn set_mask(&mut self, id: Id) {
        let new_filter = match id {
            Id::Standard(id) => filter::Mask32::frames_with_std_id(StandardId::MAX, id),
            Id::Extended(id) => filter::Mask32::frames_with_ext_id(ExtendedId::MAX, id),
        };

        self.can
            .modify_filters()
            .enable_bank(0, Fifo::Fifo0, new_filter);
    }
}
