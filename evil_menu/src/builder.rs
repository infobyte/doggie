use embedded_can::Id;
use evil_core::{AttackCmd, FastBitQueue, MAX_ATTACK_SIZE};

#[derive(Clone, Copy, PartialEq)]
pub enum HighLevelAttackCmd {
    None,
    // Provisorio
    Match {
        id: Id,
        data_len: usize,
        data: Option<[u8; 8]>,
    },
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
        bits: u128,
        bits_count: usize,
        force: bool,
    },
    WaitEof,
    SendMsg {
        id: Id,
        data: Option<[u8; 8]>,
        data_len: usize,
        rtr: bool,
        force: bool,
    },
    SendOverloadFrame,
}

pub struct AttackBuilder {
    low_level_attack: [AttackCmd; MAX_ATTACK_SIZE],
    low_level_attack_index: usize,
    high_level_attack: [HighLevelAttackCmd; MAX_ATTACK_SIZE],
    high_level_attack_index: usize,
}

impl AttackBuilder {
    pub fn new() -> Self {
        let mut attack = [AttackCmd::None; MAX_ATTACK_SIZE];
        let high_level_attack = [HighLevelAttackCmd::None; MAX_ATTACK_SIZE];
        // Start of frame
        attack[0] = AttackCmd::Wait { bits: 1 };

        Self {
            low_level_attack: attack,
            low_level_attack_index: 1,
            high_level_attack,
            high_level_attack_index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.low_level_attack_index = 1;
        self.low_level_attack = [AttackCmd::None; MAX_ATTACK_SIZE];
        self.low_level_attack[0] = AttackCmd::Wait { bits: 1 };
        self.high_level_attack = [HighLevelAttackCmd::None; MAX_ATTACK_SIZE];
        self.high_level_attack_index = 0;
    }

    pub fn push_low_level_attack_cmd(&mut self, attack: AttackCmd) {
        self.low_level_attack[self.low_level_attack_index] = attack;
        self.low_level_attack_index += 1;
    }

    pub fn push_high_level_attack_cmd(&mut self, attack: HighLevelAttackCmd) {
        self.high_level_attack[self.high_level_attack_index] = attack;
        self.high_level_attack_index += 1;
    }

    pub fn build(&mut self) -> &[AttackCmd] {
        // Iterate over high level attack commands and build the atttack
        for i in 0..self.high_level_attack_index {
            let cmd = self.high_level_attack[i];
            match cmd {
                HighLevelAttackCmd::None => {}
                HighLevelAttackCmd::Match { id, data, data_len } => match id {
                    Id::Standard(id) => {
                        match data {
                            None => {
                                // Match {Id (11 bits), RTR bit = 0, Id Extension bit = 0, Reserved bit = 0, DLC (4 bits) = 0b0000 }
                                // Total 18 bits
                                let bits = (id.as_raw() as u64) << 7;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits.into(), 14),
                                });
                            }
                            Some(data) => {
                                // Match {Id (11 bits), RTR bit = 0, Id Extension bit = 0, Reserved bit = 0, DLC (4 bits) }
                                // Total 18 bits
                                let bits = ((id.as_raw() as u64) << 7) | data_len as u64;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, 18),
                                });

                                // Data
                                let bits = data[..data_len]
                                    .iter()
                                    .rev()
                                    .enumerate()
                                    .map(|(i, &b)| (b as u64) << (i * 8))
                                    .sum();
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, data_len),
                                });
                            }
                        }
                    }
                    Id::Extended(id) => {
                        match data {
                            None => {
                                // Match { Id (11 bits), SRR bit = 1, Id Extension bit = 1, Id (18 bits), RTR = 0, Reserved bit = 0, Reserved bit = 0, DLC (4 bits) = 0b0000 }
                                // Total 38 bits
                                let bits_high = (id.as_raw() as u64 & 0x1ffc0000) >> 18;
                                let bits_low = id.as_raw() as u64 & 0x3ffff;
                                let bits = bits_high << 27 | 1 << 26 | 1 << 25 | bits_low << 7;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, 38),
                                });
                            }
                            Some(data) => {
                                // Match { Id (11 bits), SRR bit = 1, Id Extension bit = 1, Id (18 bits), RTR = 0, Reserved bit = 0, Reserved bit = 0, DLC (4 bits) }
                                // Total 38 bits
                                let bits_high = (id.as_raw() as u64 & 0x1ffc0000) >> 18;
                                let bits_low = id.as_raw() as u64 & 0x3ffff;
                                let bits = bits_high << 27
                                    | 1 << 26
                                    | 1 << 25
                                    | bits_low << 7
                                    | data_len as u64;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, 38),
                                });

                                // Data
                                let bits = data[..data_len]
                                    .iter()
                                    .rev()
                                    .enumerate()
                                    .map(|(i, &b)| (b as u64) << (i * 8))
                                    .sum();
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, data_len),
                                });
                            }
                        }
                    }
                },
                _ => {
                    todo!()
                }
            }
        }

        &self.low_level_attack[..self.low_level_attack_index]
    }

    pub fn set_test_attack(&mut self) {
        self.low_level_attack = [AttackCmd::None; MAX_ATTACK_SIZE];
        self.low_level_attack[0] = AttackCmd::Wait { bits: 1 };
        self.low_level_attack[1] = AttackCmd::Force {
            stream: FastBitQueue::new(0b1010_101, 7),
        };
    }
}
