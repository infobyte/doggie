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
pub enum AttackCmd {
    Wait { bits: usize },
    Force { stream: BitStream<160> },
    Send { stream: BitStream<160> },
    Match { stream: BitStream<160> },
    Read { len: usize },
    WaitBuffered,
    None,
}
