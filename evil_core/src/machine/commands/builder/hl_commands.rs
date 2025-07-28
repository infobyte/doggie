use crate::machine::commands::{
    builder::{BuildError, Buildable},
    AttackCmd, FastBitQueue, FastBitStack,
};
use core::cmp::min;
use defmt::info;
use embedded_can::Id;
use heapless::Vec;

struct MsgBitQueue {
    data: [u8; 13],
    end: usize,
    start: usize,
    crc: u16,
}

impl MsgBitQueue {
    const CRC_POLY: u16 = 0xC599;
    const CRC_WIDTH: u16 = 15;

    fn crc_process_bit(&mut self, bit: u8) {
        // Get bit at the left
        let crc_high_bit = self.crc >> (Self::CRC_WIDTH - 1) & 1;

        // Append the new bit at right
        self.crc = (self.crc << 1) | (bit as u16);

        // If higher bit is 1, make a xor
        if crc_high_bit == 1 {
            self.crc ^= Self::CRC_POLY;
        }

        // Save only the 15 bits
        self.crc &= (1 << Self::CRC_WIDTH) - 1;
    }

    fn crc_calculate(&mut self) -> u16 {
        // Get the reminder of the divition
        for _ in 0..Self::CRC_WIDTH {
            self.crc_process_bit(0);
        }

        self.crc
    }

    fn new() -> Self {
        Self {
            data: [0; 13],
            end: 0,
            start: 0,
            crc: 0,
        }
    }

    fn append(&mut self, value: u32, num_bits: usize) {
        for i in (0..num_bits).rev() {
            let bit = (value >> i) as u8 & 1;
            let byte_idx = self.end / 8;
            let bit_offset = 7 - (self.end % 8);
            self.data[byte_idx] |= bit << bit_offset;
            self.end += 1;

            self.crc_process_bit(bit);
        }
    }

    fn pop(&mut self) -> u8 {
        let byte_idx = self.start / 8;
        let bit_offset = 7 - (self.start % 8);

        let res = (self.data[byte_idx] >> bit_offset) & 1;

        self.start += 1;

        res
    }

    fn as_ref(&self) -> &[u8; 13] {
        &self.data
    }

    fn len(&self) -> usize {
        let bits = self.end - self.start;
        bits / 8 + if bits % 8 != 0 { 1 } else { 0 }
    }

    fn pop_chunk(&mut self) -> Option<(u64, usize)> {
        if self.end == self.start {
            None
        } else {
            let size = min(64, self.end - self.start);

            let mut value: u64 = 0;
            for _ in 0..size {
                value = (value << 1) | self.pop() as u64;
            }

            Some((value, size))
        }
    }

    fn append_crc(&mut self) {
        let mut crc = self.crc_calculate();

        info!("CRC: {:X}", crc);

        self.append(crc as u32, 15);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HighLevelAttackCmd {
    MatchId {
        id: Id,
        rtr: bool,
    },
    MatchData {
        data: [u8; 8],
        match_size: u8,
    },
    SkipData,
    Wait {
        bits: usize,
    },
    SendError {
        count: usize,
    },
    SendRaw {
        bits: u64,
        bits_count: usize,
        force: bool,
    },
    WaitBusFree,
    WaitSof,
    SendMsg {
        id: Id,
        data: Option<[u8; 8]>,
        data_len: usize,
        rtr: bool,
        force: bool,
    },
}

impl<const OUT_SIZE: usize> Buildable<OUT_SIZE> for HighLevelAttackCmd {
    type Res = AttackCmd;

    fn build(&self, attack: &mut Vec<Self::Res, OUT_SIZE>) -> Result<usize, BuildError> {
        match self {
            Self::MatchId { id, rtr } => Self::build_match_id(attack, id, rtr),
            Self::SkipData => Self::build_skip_data(attack),
            Self::Wait { bits } => Self::build_wait(attack, bits),
            Self::MatchData { data, match_size } => {
                Self::build_match_data(attack, data, match_size)
            }
            Self::SendError { count } => Self::build_send_error(attack, count),
            Self::SendRaw {
                bits,
                bits_count,
                force,
            } => Self::build_send_raw(attack, bits, bits_count, force),
            Self::WaitBusFree => Self::build_wait_bus_free(attack),
            Self::WaitSof => Self::build_wait_sof(attack),
            Self::SendMsg {
                id,
                data,
                data_len,
                rtr,
                force,
            } => Self::build_send_msg(attack, id, data, data_len, rtr, force),
        }
    }
}

impl HighLevelAttackCmd {
    // pub fn build(self, attack: &mut [AttackCmd]) -> Result<usize, BuildError> {
    //     match self {
    //         Self::MatchId { id, rtr } => Self::build_match_id(attack, id, rtr),
    //         Self::SkipData => Self::build_skip_data(attack),
    //         Self::Wait { bits } => Self::build_wait(attack, bits),
    //         Self::MatchData { data, match_size } => {
    //             Self::build_match_data(attack, data, match_size)
    //         }
    //         Self::SendError { count } => Self::build_send_error(attack, count),
    //         Self::SendRaw {
    //             bits,
    //             bits_count,
    //             force,
    //         } => Self::build_send_raw(attack, bits, bits_count, force),
    //         Self::WaitEof => Self::build_wait_eof(attack),
    //         Self::WaitSof => Self::build_wait_sof(attack),
    //         Self::SendMsg {
    //             id,
    //             data,
    //             data_len,
    //             rtr,
    //             force,
    //         } => Self::build_send_msg(attack, id, data, data_len, rtr, force),
    //     }
    // }

    fn build_match_id<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
        id: &Id,
        rtr: &bool,
    ) -> Result<usize, BuildError> {
        // Validation: It has to be after a WaitSof command
        // Pre condition: We are at the second bit of the frame (after SOF)
        // Post condition: At the end we are in the beginning of the DLC

        let mut id_bits = FastBitStack::new();

        let len = match id {
            Id::Standard(id) => {
                // Push the ID
                id_bits.push_num(id.as_raw(), 11);

                info!("RAW ID: {}", id.as_raw());

                // Push RTR
                id_bits.push(*rtr);

                // Push IDE
                id_bits.push(false);

                // Push reserved
                id_bits.push(false);

                14
            }
            Id::Extended(id) => {
                // Push STD ID
                id_bits.push_num((id.as_raw() >> 7) as u32, 11);

                // Push SSR
                id_bits.push(true);

                // Push IDE
                id_bits.push(true);

                // Push EXT ID
                id_bits.push_num(id.as_raw() as u32, 18);

                // Push RTR
                id_bits.push(*rtr);

                // Push Reserved
                id_bits.push_num(0 as u32, 2);

                34
            }
        };

        attack.push(AttackCmd::WaitBusFree { count: 0 }).unwrap();
        attack.push(AttackCmd::WaitForSof).unwrap();
        attack.push(AttackCmd::Wait { bits: 1 }).unwrap();

        attack
            .push(AttackCmd::Match {
                stream: FastBitQueue::new(id_bits.value() as u64, len),
            })
            .unwrap();

        Ok(2)
    }

    fn build_skip_data<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
    ) -> Result<usize, BuildError> {
        // Validation: This has to be after a MatchId command
        // Pre condition: We are in the DLC position
        // Post condition: We are in the end of the data
        attack.push(AttackCmd::Read { len: 4 }).unwrap();
        attack.push(AttackCmd::MulBuffered { mult: 8 }).unwrap();
        attack.push(AttackCmd::WaitBuffered).unwrap();

        Ok(3)
    }

    fn build_wait<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
        bits: &usize,
    ) -> Result<usize, BuildError> {
        attack.push(AttackCmd::Wait { bits: *bits }).unwrap();
        Ok(1)
    }

    fn build_match_data<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
        data: &[u8; 8],
        match_size: &u8,
    ) -> Result<usize, BuildError> {
        // This will match the first {match_size} bytes of the data and wait
        // for the rest of the data.
        // Validation: This has to be after a MatchId command
        // Pre condition: We are at the DLC position
        // Post condition: We are at the end of the data or the match has faild

        // Read DLC
        attack.push(AttackCmd::Read { len: 4 }).unwrap();

        // Match data
        let mut raw_data: u64 = 0;
        for byte_index in 0..*match_size {
            raw_data |= (data[byte_index as usize] as u64) << (8 * byte_index);
        }

        attack
            .push(AttackCmd::Match {
                stream: FastBitQueue::new(raw_data, *match_size as usize * 8),
            })
            .unwrap();

        // Operate with the buffered value
        attack
            .push(AttackCmd::SubBuffered {
                sub: *match_size as u32,
            })
            .unwrap();
        attack.push(AttackCmd::MulBuffered { mult: 8 }).unwrap();

        // Wait the rest of the data
        attack.push(AttackCmd::WaitBuffered).unwrap();

        Ok(5)
    }

    fn build_send_error<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
        count: &usize,
    ) -> Result<usize, BuildError> {
        // The active error will be 6 dominant bits and 8 recessive bits: 14 bits
        // In a 64 bits queue, we could put 4.5 error messages.
        if *count == 0 {
            return Err(BuildError::BadArguments);
        }

        let mut attack_index = 0;

        let mut c = *count;
        while c > 0 {
            let error_cnt = min(4, *count);

            let mut bits: u64 = 0;
            for index in 0..error_cnt {
                bits = (bits << (14 * index)) | 0b00_0000_1111_1111;
            }

            attack
                .push(AttackCmd::Send {
                    stream: FastBitQueue::new(bits, error_cnt * 14),
                })
                .unwrap();

            attack_index += 1;
            c -= error_cnt;
        }

        Ok(attack_index + 1)
    }

    fn build_send_raw<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
        bits: &u64,
        bits_count: &usize,
        force: &bool,
    ) -> Result<usize, BuildError> {
        let queue = FastBitQueue::new(*bits, *bits_count);

        if *force {
            attack.push(AttackCmd::Force { stream: queue }).unwrap();
        } else {
            attack.push(AttackCmd::Send { stream: queue }).unwrap();
        }

        Ok(1)
    }

    fn build_wait_bus_free<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
    ) -> Result<usize, BuildError> {
        attack
            .push(AttackCmd::SetBitStuffing { state: false })
            .unwrap();
        attack.push(AttackCmd::WaitBusFree { count: 0 }).unwrap();
        attack
            .push(AttackCmd::SetBitStuffing { state: true })
            .unwrap();
        Ok(1)
    }

    fn build_wait_sof<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
    ) -> Result<usize, BuildError> {
        attack.push(AttackCmd::WaitForSof).unwrap();
        Ok(1)
    }

    fn build_send_msg<const OUT_SIZE: usize>(
        attack: &mut Vec<AttackCmd, OUT_SIZE>,
        id: &Id,
        data: &Option<[u8; 8]>,
        data_len: &usize,
        rtr: &bool,
        force: &bool,
    ) -> Result<usize, BuildError> {
        // Precondition: The bus is not bussy
        let mut msg_queue = MsgBitQueue::new();

        // SoF
        msg_queue.append(0, 1);

        // ID
        match id {
            Id::Standard(id_value) => {
                msg_queue.append(id_value.as_raw() as u32, 11);
            }
            Id::Extended(id_value) => {
                // STD id bits
                msg_queue.append(id_value.as_raw() >> 18, 11);

                // SRR and IDE
                msg_queue.append(0b11, 2);

                // EXT id bits
                msg_queue.append(id_value.as_raw(), 18);
            }
        }

        // RTR
        msg_queue.append(*rtr as u32, 1);

        // IDE for std and reserved bits
        msg_queue.append(0b00, 2);

        // DLC
        msg_queue.append(*data_len as u32, 4);

        // DATA
        if let Some(data_arr) = data {
            for byte in &data_arr[0..*data_len] {
                msg_queue.append((*byte).into(), 8);
            }
        }

        // msg_queue.append(crc.get_crc() as u32, 15);
        msg_queue.append_crc();

        defmt::info!("MSG: {}", msg_queue.as_ref());

        let mut attack_index = 0;

        while let Some((value, size)) = msg_queue.pop_chunk() {
            attack
                .push(if *force {
                    AttackCmd::Force {
                        stream: FastBitQueue::new(value, size),
                    }
                } else {
                    AttackCmd::Send {
                        stream: FastBitQueue::new(value, size),
                    }
                })
                .unwrap();

            attack_index += 1;
        }

        attack.push(AttackCmd::SetBitStuffing { state: false });

        let mut eof_queue = MsgBitQueue::new();

        // CRC Delimiter
        eof_queue.append(0b1, 1);

        // ACK
        if *force {
            eof_queue.append(0b0, 1);
        } else {
            eof_queue.append(0b1, 1);
        }

        //  ACK delimiter + EoF + IFS = 11
        eof_queue.append(0b11111111111, 1 + 7 + 3);

        while let Some((value, size)) = eof_queue.pop_chunk() {
            attack
                .push(if *force {
                    AttackCmd::Force {
                        stream: FastBitQueue::new(value, size),
                    }
                } else {
                    AttackCmd::Send {
                        stream: FastBitQueue::new(value, size),
                    }
                })
                .unwrap();

            attack_index += 1;
        }

        attack.push(AttackCmd::SetBitStuffing { state: true });

        Ok(attack_index + 2)
    }
}
