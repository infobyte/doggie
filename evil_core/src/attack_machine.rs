use crate::commands::AttackCmd;
use crate::attack_errors::AttackError;
use crate::tranceiver::Tranceiver;

const MAX_ATTACK_SIZE: usize = 32;


pub struct AttackMachine<Tr>
where
    Tr: Tranceiver,
{
    index: usize,
    attack: [AttackCmd; MAX_ATTACK_SIZE],
    pub tranceiver: Tr
}

impl <Tr> AttackMachine <Tr>
where
    Tr: Tranceiver,
{
    pub const QUANTA_PER_BIT: u32 = 8;

    pub fn new(tranceiver: Tr) -> Self {
        Self {
            index: 0,
            attack: [AttackCmd::None; MAX_ATTACK_SIZE],
            tranceiver,
        }
    }


    pub fn arm(&mut self, attack: &[AttackCmd]) -> Result<(), AttackError> {
        self.index = 0;

        if attack.len() > self.attack.len() {
            return Err(AttackError::AttackToLong);
        }

        for index in 0..(self.attack.len()) {
            if index < attack.len() {
                self.attack[index] = attack[index];
            } else {
                self.attack[index] = AttackCmd::None;
            }
        }
        
        Ok(())
    }

    #[inline]
    pub fn handle(&mut self) -> Option<u32> {
        if self.index >= self.attack.len() {
           return None;
        }

        match self.attack[self.index] {
            AttackCmd::None => {
                self.index += 1;
                None
            },

            AttackCmd::Wait { bits } => {
                self.index += 1;
                Some(AttackMachine::<Tr>::QUANTA_PER_BIT * bits as u32)
            },

            AttackCmd::Force { ref mut stream } => {
                match stream.pop() {
                    Some(state) => {
                        self.tranceiver.set_force(state);
                        Some(AttackMachine::<Tr>::QUANTA_PER_BIT)
                    },
                    None => {
                        self.index += 1;
                        self.tranceiver.set_force(false);

                        Some(0)
                    }
                }
            }
        }
    }
}
