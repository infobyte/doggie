use super::{BuildError, Buildable, HighLevelAttackCmd};
use embedded_can::Id;
use heapless::Vec;

pub const MAX_HL_COMMANDS: usize = 32;

#[derive(Debug)]
pub enum PredefAttacks {
    TestAttack,
    SpoofingAttack {
        id: Id,
        spoof_data: Vec<u8, 8>,
        match_data: Vec<u8, 8>,
    },
    CustomAttack {
        commands: Vec<HighLevelAttackCmd, MAX_HL_COMMANDS>,
    },
}

impl<const OUT_SIZE: usize> Buildable<OUT_SIZE> for PredefAttacks {
    type Res = HighLevelAttackCmd;

    fn build(&self, out_vec: &mut Vec<Self::Res, OUT_SIZE>) -> Result<usize, BuildError> {
        match self {
            PredefAttacks::TestAttack => self.build_test_attack(out_vec),
            PredefAttacks::SpoofingAttack {
                id,
                ref spoof_data,
                ref match_data,
            } => self.build_spoofing_attack(out_vec, id, spoof_data, match_data),
            PredefAttacks::CustomAttack { ref commands } => {
                self.build_custom_attack(out_vec, &commands)
            }
        }
    }
}

impl PredefAttacks {
    fn build_custom_attack<const SIZE: usize>(
        &self,
        hl_attack: &mut Vec<HighLevelAttackCmd, SIZE>,
        commands: &Vec<HighLevelAttackCmd, MAX_HL_COMMANDS>,
    ) -> Result<usize, BuildError> {
        for item in commands {
            hl_attack.push(item.clone()).unwrap();
        }

        Ok(commands.len())
    }

    fn build_spoofing_attack<const SIZE: usize>(
        &self,
        hl_attack: &mut Vec<HighLevelAttackCmd, SIZE>,
        id: &Id,
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
            .push(HighLevelAttackCmd::MatchId {
                id: *id,
                rtr: false,
            })
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
        hl_attack.push(HighLevelAttackCmd::WaitBusFree).unwrap();
        // Send the actual message
        let mut data = [0; 8];
        for (index, byte) in spoof_data.iter().enumerate() {
            data[index] = *byte;
        }
        hl_attack
            .push(HighLevelAttackCmd::SendMsg {
                id: *id,
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
        // hl_attack
        //     .push(HighLevelAttackCmd::Wait { bits: 20 })
        //     .unwrap();
        hl_attack
            .push(HighLevelAttackCmd::SendRaw {
                bits: 0b0101_0101_0101_0101,
                bits_count: 16,
                force: true,
            })
            .unwrap();

        Ok(2)
    }
}
