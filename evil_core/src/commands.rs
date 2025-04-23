#[derive(Clone, Copy)]
pub struct BitStream<const SIZE: usize> {
    data: [bool; SIZE],
    start: usize,
    end: usize,
}

impl<const SIZE: usize> BitStream<SIZE> {
    pub fn new() -> Self {
        Self {
            data: [false; SIZE],
            start: 0,
            end: 0
        }
    }

    pub fn from_u32(data: u32, len: usize) -> Self {
        let mut bs = Self {
            data: [false; SIZE],
            start: 0,
            end: len.min(SIZE),
        };

        // Invert bit order
        for index in 0..bs.end {
            bs.data[bs.end - index - 1] = (data >> index) & 1 != 0;
        }

        bs
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.end >= self.start {
            self.end - self.start
        } else {
            SIZE - self.start + self.end
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Result<bool, ()> {
        if self.start == self.end {
            return Err(());
        }

        let res = self.data[self.start];
        self.start = (self.start + 1) % SIZE;

        Ok(res)
    }

    #[inline]
    pub fn push(&mut self, value: bool) -> Result<(), ()> {
        let next_end = (self.end + 1) % SIZE;
        if next_end == self.start {
            return Err(());
        }

        self.data[self.end] = value;
        self.end = next_end;

        Ok(())
    }

    #[inline]
    pub fn clean(&mut self) {
        self.end = self.start;
    }

    #[inline]
    pub fn to_u32(&self) -> u32 {
        let mut result = 0u32;
        let len = self.len();
        let max_bits = len.min(32); // Limit to 32 bits for u32

        for i in 0..max_bits {
            let index = (self.start + i) % SIZE;
            if self.data[index] {
                // Map buffer index to u32 bit position, maintaining inverted order
                // Highest index (end-1) -> LSB (bit 0), lowest index (start) -> MSB
                let bit_pos = max_bits - 1 - i;
                result |= 1 << bit_pos;
            }
        }

        result
    }
}


#[derive(Clone, Copy)]
pub struct FastBitQueue {
    value: u32,
    len: u8,
}

impl FastBitQueue {
    pub fn new(value: u32, len: usize) -> Self {
        Self {
            value: value << (32 - len),
            len: len as u8
        }
    } 

    #[inline]
    pub fn pop(&mut self) -> bool {
        self.len -= 1;
        let res = (self.value & (1 << 31)) != 0;
        self.value = self.value << 1;
        res
    }

    #[inline]
    pub fn len(&self) -> u8 {
        self.len
    }
    
}


pub struct FastBitStack {
    value: u8,
}

impl FastBitStack {
    pub fn new() -> Self {
        Self {
            value: 0
        }
    }

    #[inline]
    pub fn push(&mut self, value: bool) {
        self.value = (self.value << 1) | (value as u8);
    }

    #[inline]
    pub fn value(&self) -> u8 {
        self.value
    }

    #[inline]
    pub fn clean(&mut self) {
        self.value = 0;
    }
}

#[derive(Clone, Copy)]
pub enum AttackCmd {
    Wait { bits: usize },
    Force { stream: FastBitQueue },
    Send { stream: FastBitQueue },
    Match { stream: FastBitQueue },
    Read { len: usize },
    WaitBuffered,
    None,
}
