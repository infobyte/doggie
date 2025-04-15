#[derive(Clone, Copy)]
pub struct BitStream<const SIZE: usize> {
    data: [bool; SIZE],
    len: usize,
    index: usize,
}

impl<const SIZE: usize> BitStream<SIZE> {

    pub fn from_u8(data: u8, len: usize) -> Self {
        let mut bs = BitStream {
            data: [false; SIZE],
            len,
            index: 0
        };

        for index in 0..8 {
            bs.data[index] = (data >> index) & 1 != 0;
        }

        bs
    }

    pub fn pop(&mut self) -> Option<bool> {
        if self.len <= self.index {
            return None;
        }

        let res = self.data[self.index];

        self.index += 1;

        Some(res)
    }
}

#[derive(Clone, Copy)]
pub enum AttackCmd {
    Wait { bits: usize },
    Force { stream: BitStream<160> },
    // Match { len: usize, data:  },
    // Send { len, data },
    // Read { len }
    // SkipData {},
    None,
}
