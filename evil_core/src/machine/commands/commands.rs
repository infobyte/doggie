#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FastBitQueue {
    value: u64,
    original_value: u64,
    len: u8,
    original_len: u8,
}

impl FastBitQueue {
    pub fn new(value: u64, len: usize) -> Self {
        Self {
            value: value << (64 - len),
            original_value: value << (64 - len),
            len: len as u8,
            original_len: len as u8,
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> bool {
        self.len -= 1;
        let res = (self.value & (1 << 63)) != 0;
        self.value = self.value << 1;
        res
    }

    #[inline(always)]
    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn restore(&mut self) {
        self.len = self.original_len;
        self.value = self.original_value;
    }
}

pub struct FastBitStack {
    value: u32,
}

impl FastBitStack {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    #[inline(always)]
    pub fn push(&mut self, value: bool) {
        self.value = (self.value << 1) | (value as u32);
    }

    #[inline(always)]
    pub fn push_num<T: Into<u32>>(&mut self, value: T, size: usize) {
        let ival: u32 = value.into();
        self.value = (self.value << size) | (ival & (u32::pow(2, size as u32) - 1));
    }

    #[inline(always)]
    pub fn value(&self) -> u32 {
        self.value
    }

    #[inline(always)]
    pub fn clean(&mut self) {
        self.value = 0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackCmd {
    Wait { bits: usize },
    Force { stream: FastBitQueue },
    Send { stream: FastBitQueue },
    Match { stream: FastBitQueue },
    Read { len: usize },
    WaitBuffered,
    MulBuffered { mult: u8 },
    SubBuffered { sub: u32 },
    WaitForSof,
    SetBitStuffing { state: bool },
    WaitBusFree { count: u8 },
    None,
}
