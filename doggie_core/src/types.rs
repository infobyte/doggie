use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use slcan::SlcanCommand;

const CAN_CHANNEL_SIZE: usize = 16;

pub type CanChannel = Channel<CriticalSectionRawMutex, SlcanCommand, CAN_CHANNEL_SIZE>;
pub type CanChannelSender =
    Sender<'static, CriticalSectionRawMutex, SlcanCommand, CAN_CHANNEL_SIZE>;
pub type CanChannelReceiver =
    Receiver<'static, CriticalSectionRawMutex, SlcanCommand, CAN_CHANNEL_SIZE>;
