use super::{BuildError, HighLevelAttackCmd};
use heapless::Vec;

pub enum PredefAttacks {
    TestAttack,
}

impl PredefAttacks {
    pub fn build<const SIZE: usize>(
        self,
        hl_attack: &mut Vec<HighLevelAttackCmd, SIZE>,
    ) -> Result<usize, BuildError> {
        match self {
            PredefAttacks::TestAttack => self.build_test_attack(hl_attack),
        }
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
