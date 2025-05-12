use core::slice::Iter;

use crate::builder::{BuildError, HighLevelAttackCmd};
use evil_core::AttackCmd;
use heapless::Vec;

pub struct AttackBuilder<const SIZE: usize> {
    hl_cmd_vec: Vec<HighLevelAttackCmd, SIZE>,
}

impl<const SIZE: usize> AttackBuilder<SIZE> {
    pub fn new() -> Self {
        Self {
            hl_cmd_vec: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.hl_cmd_vec.clear()
    }

    pub fn push(&mut self, cmd: HighLevelAttackCmd) -> Result<(), BuildError> {
        match self.hl_cmd_vec.push(cmd) {
            Ok(()) => {
                // Check if it is valid
                match self.validate() {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        // If it is not valid, revert
                        self.pop();
                        Err(error)
                    }
                }
            }
            _ => Err(BuildError::BufferIsFull),
        }
    }

    pub fn pop(&mut self) -> Option<HighLevelAttackCmd> {
        // We don,t need to validate, the validation is constructive
        self.hl_cmd_vec.pop()
    }

    pub fn relocate(&mut self, from: usize, to: usize) -> Result<(), BuildError> {
        if self.hl_cmd_vec.is_full() {
            return Err(BuildError::BufferIsFull);
        }

        if from >= self.hl_cmd_vec.len() || to >= self.hl_cmd_vec.len() {
            return Err(BuildError::IndexOutOfBounds);
        }

        let from_cmd = self.hl_cmd_vec.remove(from);
        // Shouldn fail
        self.hl_cmd_vec.insert(to, from_cmd).unwrap();

        match self.validate() {
            Ok(_) => Ok(()),
            Err(error) => {
                // It must not fail, reverting
                self.relocate(to, from);
                Err(error)
            }
        }
    }

    pub fn remove(&mut self, index: usize) -> Result<(), BuildError> {
        if index >= self.hl_cmd_vec.len() {
            return Err(BuildError::IndexOutOfBounds);
        }

        let cmd = self.hl_cmd_vec.remove(index);

        // Valid and, if needed, revert
        match self.validate() {
            Ok(_) => Ok(()),
            Err(error) => {
                self.hl_cmd_vec.insert(index, cmd);
                Err(error)
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, HighLevelAttackCmd> {
        self.hl_cmd_vec.iter()
    }

    pub fn build(&mut self, attack: &mut [AttackCmd]) -> Result<usize, BuildError> {
        let mut index = 0;
        for cmd in self.hl_cmd_vec.iter() {
            match cmd.build(&mut attack[index..]) {
                Ok(written) => index += written,
                Err(error) => return Err(error),
            }
        }

        Ok(index)
    }

    fn validate(&self) -> Result<(), BuildError> {
        // TODO
        Ok(())
    }
}
