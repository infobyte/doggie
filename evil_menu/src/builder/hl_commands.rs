use core::cmp::{max, min};

use crate::builder::BuildError;
use crc_any::CRC;
use embedded_can::Id;
use evil_core::{AttackCmd, FastBitQueue, FastBitStack};

fn append_bits(bitstream: &mut [u8], bit_pos: &mut usize, value: u32, num_bits: usize) {
    // [...,[9,10,11,12,13,14,15],[0, 1, 2, 3, 4, 5, 6, 7, 8]]
    for i in (0..num_bits).rev() {
        let bit = ((value >> i) & 1) as u8;
        let byte_idx = *bit_pos / 8;
        let bit_offset = 7 - (*bit_pos % 8);
        bitstream[byte_idx] |= bit << bit_offset;
        *bit_pos += 1;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HighLevelAttackCmd {
    MatchId {
        id: Id,
    },
    MatchData {
        data_len: usize,
        data: Option<[u8; 8]>,
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
    WaitEof,
    WaitSof,
    SendMsg {
        id: Id,
        data: Option<[u8; 8]>,
        data_len: usize,
        rtr: bool,
        force: bool,
    },
}

impl HighLevelAttackCmd {
    pub fn build(self, attack: &mut [AttackCmd]) -> Result<usize, BuildError> {
        match self {
            Self::MatchId { id } => Self::build_match_id(attack, id),
            Self::SkipData => Self::build_skip_data(attack),
            Self::Wait { bits } => Self::build_wait(attack, bits),
            Self::MatchData { data_len, data } => Self::build_match_data(attack, data_len, data),
            Self::SendError { count } => Self::build_send_error(attack, count),
            Self::SendRaw {
                bits,
                bits_count,
                force,
            } => Self::build_send_raw(attack, bits, bits_count, force),
            Self::WaitEof => Self::build_wait_eof(attack),
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

    fn build_match_id(attack: &mut [AttackCmd], id: Id) -> Result<usize, BuildError> {
        // Validation: It has to be after a WaitSof command
        // Pre condition: We are at the second bit of the frame (after SOF)
        // Post condition: At the end we are in the beginning of the DLC

        let mut id_bits = FastBitStack::new();

        let len = match id {
            Id::Standard(id) => {
                // Push the ID
                id_bits.push_num(id.as_raw(), 11);

                // Push RTR
                id_bits.push(false);

                // Push IDE
                id_bits.push(false);

                // Push reserved
                id_bits.push(false);

                14
            }
            Id::Extended(id) => {
                // Push STD ID
                id_bits.push_num((id.as_raw() >> 7) as usize, 11);

                // Push SSR
                id_bits.push(true);

                // Push IDE
                id_bits.push(true);

                // Push EXT ID
                id_bits.push_num(id.as_raw() as usize, 18);

                // Push RTR
                id_bits.push(false);

                // Push Reserved
                id_bits.push_num(0 as usize, 2);

                34
            }
        };

        attack[0] = AttackCmd::Match {
            stream: FastBitQueue::new(id_bits.value() as u64, len),
        };

        Ok(1)
    }

    fn build_skip_data(attack: &mut [AttackCmd]) -> Result<usize, BuildError> {
        // Validation: This has to be after a MatchId command
        // Pre condition: We are in the DLC position
        // Post condition: We are in the end of the data
        attack[0] = AttackCmd::Read { len: 4 };
        attack[1] = AttackCmd::WaitBuffered;

        Ok(2)
    }

    fn build_wait(attack: &mut [AttackCmd], bits: usize) -> Result<usize, BuildError> {
        attack[0] = AttackCmd::Wait { bits };
        Ok(1)
    }

    fn build_match_data(
        attack: &mut [AttackCmd],
        data_len: usize,
        data: Option<[u8; 8]>,
    ) -> Result<usize, BuildError> {
        // Validation: This has to be after a MatchId command
        // Pre condition: We are at the DLC position
        // Post condition: We are at the end of the data or the match has faild

        if data_len > 8 || (data.is_none() && data_len != 0) {
            return Err(BuildError::IndexOutOfBounds);
        }

        attack[0] = AttackCmd::Match {
            stream: FastBitQueue::new(data_len as u64, 4),
        };

        Ok(match data {
            Some(data_arr) => {
                let mut raw_data: u64 = 0;
                for byte_index in 0..data_len {
                    raw_data |= (data_arr[byte_index] as u64) << (8 * byte_index);
                }

                attack[0] = AttackCmd::Match {
                    stream: FastBitQueue::new(raw_data, data_len * 8),
                };
                2
            }
            None => 1,
        })
    }

    fn build_send_error(attack: &mut [AttackCmd], mut count: usize) -> Result<usize, BuildError> {
        // The active error will be 6 dominant bits and 8 recessive bits: 14 bits
        // In a 64 bits queue, we could put 4.5 error messages.
        if count == 0 {
            return Err(BuildError::BadArguments);
        }

        let mut attack_index = 0;

        while count > 0 {
            let error_cnt = min(4, count);

            let mut bits: u64 = 0;
            for index in 0..error_cnt {
                bits = (bits << (14 * index)) | 0b00_0000_1111_1111;
            }

            attack[attack_index] = AttackCmd::Send {
                stream: FastBitQueue::new(bits, error_cnt * 14),
            };

            attack_index += 1;
            count -= error_cnt;
        }

        Ok(attack_index + 1)
    }

    fn build_send_raw(
        attack: &mut [AttackCmd],
        bits: u64,
        bits_count: usize,
        force: bool,
    ) -> Result<usize, BuildError> {
        let queue = FastBitQueue::new(bits, bits_count);

        if force {
            attack[0] = AttackCmd::Force { stream: queue };
        } else {
            attack[0] = AttackCmd::Send { stream: queue };
        }

        Ok(1)
    }

    fn build_wait_eof(attack: &mut [AttackCmd]) -> Result<usize, BuildError> {
        attack[0] = AttackCmd::WaitForEof;
        Ok(1)
    }

    fn build_wait_sof(attack: &mut [AttackCmd]) -> Result<usize, BuildError> {
        attack[0] = AttackCmd::WaitForSof;
        Ok(1)
    }

    fn build_send_msg(
        attack: &mut [AttackCmd],
        id: Id,
        data: Option<[u8; 8]>,
        data_len: usize,
        rtr: bool,
        force: bool,
    ) -> Result<usize, BuildError> {
        // Precondition: The bus is not bussy
        let mut msg: [u8; 13] = [0; 13];
        let mut msg_pos = 0;

        // SoF
        append_bits(&mut msg, &mut msg_pos, 0, 1);

        // ID
        match id {
            Id::Standard(id_value) => {
                append_bits(&mut msg, &mut msg_pos, id_value.as_raw() as u32, 11);
            }
            Id::Extended(id_value) => {
                // STD id bits
                append_bits(&mut msg, &mut msg_pos, id_value.as_raw() >> 18, 11);

                // SRR and IDE
                append_bits(&mut msg, &mut msg_pos, 0b11, 2);

                // EXT id bits
                append_bits(&mut msg, &mut msg_pos, id_value.as_raw(), 18);
            }
        }

        // RTR
        append_bits(&mut msg, &mut msg_pos, rtr.into(), 1);

        // IDE for std and reserved bits
        append_bits(&mut msg, &mut msg_pos, 0b00, 2);

        // DLC
        append_bits(&mut msg, &mut msg_pos, data_len as u32, 4);

        // DATA
        if let Some(data_arr) = data {
            for byte in &data_arr[0..data_len] {
                append_bits(&mut msg, &mut msg_pos, (*byte).into(), 8);
            }
        }

        // CRC
        let mut crc = CRC::crc15can();
        msg.reverse();
        crc.digest(&msg);
        msg.reverse();

        append_bits(&mut msg, &mut msg_pos, crc.get_crc() as u32, 15);

        // ACK
        append_bits(&mut msg, &mut msg_pos, 0b11, 2);

        // EoF and IFS
        append_bits(&mut msg, &mut msg_pos, 0b0000000000, 7 + 3);

        // Pack the message in u64 chunks
        // 64 bits (max queue length)
        let mut attack_index = 0;
        let msg_len = msg_pos / 8 + if msg_pos % 8 != 0 { 1 } else { 0 };

        let mut chunk_offset = 0;
        while msg_len > 0 {
            let chunk_len = min(8, msg_len - chunk_offset);

            // Pack the value from chunk_offset to chunk_len
            let mut value: u64 = 0;
            for byte in &msg[chunk_offset..(chunk_offset + chunk_len)] {
                value = (value << 8) | *byte as u64;
            }

            let bits_left = msg_pos - (chunk_offset * 8);
            let bits_chunk = max(64, bits_left);
            chunk_offset += chunk_len;

            // Attach the attack
            attack[attack_index] = if force {
                AttackCmd::Force {
                    stream: FastBitQueue::new(value, bits_chunk),
                }
            } else {
                AttackCmd::Send {
                    stream: FastBitQueue::new(value, bits_chunk),
                }
            };

            attack_index += 1;
        }

        Ok(attack_index)
    }
}
