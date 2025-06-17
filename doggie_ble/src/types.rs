use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pipe::{Pipe, Reader, Writer},
};

use crate::constants::MAX_CMD_LEN;

pub type BlePipe = Pipe<CriticalSectionRawMutex, MAX_CMD_LEN>;
pub type BlePipeReader = Reader<'static, CriticalSectionRawMutex, MAX_CMD_LEN>;
pub type BlePipeWriter = Writer<'static, CriticalSectionRawMutex, MAX_CMD_LEN>;
