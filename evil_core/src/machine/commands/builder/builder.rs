use super::BuildError;
use core::slice::Iter;
use heapless::Vec;

pub trait Buildable<const OUT_SIZE: usize>: core::fmt::Debug {
    type Res;

    fn build(&self, out_vec: &mut Vec<Self::Res, OUT_SIZE>) -> Result<usize, BuildError>;
}

pub struct AttackBuilder<const IN_SIZE: usize, const OUT_SIZE: usize, B: Buildable<OUT_SIZE>> {
    attack_vec: Vec<B, IN_SIZE>,
}

impl<const IN_SIZE: usize, const OUT_SIZE: usize, B: Buildable<OUT_SIZE>>
    AttackBuilder<IN_SIZE, OUT_SIZE, B>
{
    pub fn new() -> Self {
        Self {
            attack_vec: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.attack_vec.clear()
    }

    pub fn pop(&mut self) -> Option<B> {
        // We don,t need to validate, the validation is constructive
        self.attack_vec.pop()
    }

    pub fn relocate(&mut self, from: usize, to: usize) -> Result<(), BuildError> {
        if from >= self.attack_vec.len() || to >= self.attack_vec.len() {
            return Err(BuildError::IndexOutOfBounds);
        }

        let from_cmd = self.attack_vec.remove(from);
        // Shouldn fail
        self.attack_vec.insert(to, from_cmd).unwrap();

        match self.validate() {
            Ok(_) => Ok(()),
            Err(error) => {
                // It must not fail, reverting
                self.relocate(to, from).unwrap();
                Err(error)
            }
        }
    }

    pub fn remove(&mut self, index: usize) -> Result<(), BuildError> {
        if index >= self.attack_vec.len() {
            return Err(BuildError::IndexOutOfBounds);
        }

        let cmd = self.attack_vec.remove(index);

        // Valid and, if needed, revert
        match self.validate() {
            Ok(_) => Ok(()),
            Err(error) => {
                self.attack_vec.insert(index, cmd).unwrap();
                Err(error)
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, B> {
        self.attack_vec.iter()
    }

    pub fn build(&mut self, out_vec: &mut Vec<B::Res, OUT_SIZE>) -> Result<usize, BuildError> {
        for attack in self.attack_vec.iter() {
            match attack.build(out_vec) {
                Ok(_) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(self.attack_vec.len())
    }

    pub fn push(&mut self, attack: B) -> Result<(), BuildError> {
        match self.attack_vec.push(attack) {
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

    fn validate(&self) -> Result<(), BuildError> {
        // TODO
        Ok(())
    }
}
