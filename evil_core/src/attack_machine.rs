use core::char::from_u32;

use defmt::info;

use crate::commands::{AttackCmd, BitStream};
use crate::attack_errors::AttackError;
use crate::tranceiver::Tranceiver;

const MAX_ATTACK_SIZE: usize = 32;


pub struct AttackMachine<Tr>
where
    Tr: Tranceiver,
{
    index: usize,
    attack: [AttackCmd; MAX_ATTACK_SIZE],
    pub tranceiver: Tr,
    matching: bool,
    buffer: BitStream<160>,

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
            matching: false,
            buffer: BitStream::new(),
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
                    Ok(state) => {
                        self.tranceiver.set_force(state);
                        Some(AttackMachine::<Tr>::QUANTA_PER_BIT)
                    },
                    Err(()) => {
                        self.index += 1;
                        self.tranceiver.set_force(false);

                        Some(0)
                    }
                }
            }

            AttackCmd::Send { ref mut stream } => {
                match stream.pop() {
                    Ok(state) => {
                        self.tranceiver.set_tx(!state);
                        Some(AttackMachine::<Tr>::QUANTA_PER_BIT)
                    },
                    Err(_) => {
                        self.index += 1;
                        self.tranceiver.set_tx(true);

                        Some(0)
                    }
                }
            }

            AttackCmd::Match { ref mut stream } => {
                if !self.matching {
                    self.matching = true;
                    // If we are startin, shift to the middle of the bit
                    return Some(AttackMachine::<Tr>::QUANTA_PER_BIT / 2)
                }

                // If we have no more bits, finish
                if stream.len() <= 0 {
                    self.matching = false;
                    self.index += 1;
                    return Some(0)
                }

                // Check the next bit with the RX
                let target_state = stream.pop().unwrap();

                // If it doesn't match, finish the attack
                if target_state != self.tranceiver.get_rx() {
                    self.matching = false;
                    return None
                }

                // If it was the last bit, shift forward to the start of the bit
                if stream.len() <= 0 {
                    return Some(AttackMachine::<Tr>::QUANTA_PER_BIT / 2)
                }

                // The bit has metched, wait for the next
                return Some(AttackMachine::<Tr>::QUANTA_PER_BIT)
            }

            AttackCmd::Read { ref mut len } => {
                if !self.matching {
                    self.matching = true;
                    self.buffer.clean();
                    // If we are startin, shift to the middle of the bit
                    return Some(AttackMachine::<Tr>::QUANTA_PER_BIT / 2)
                }

                // If we have no more bits to read, finish
                if *len <= 0 {
                    self.matching = false;
                    self.index += 1;
                    return Some(0)
                }

                // read
                self.buffer.push(self.tranceiver.get_rx()).unwrap();
                *len -= 1;

                // If it was the last bit, shift forward to the start of the bit
                if *len <= 0 {
                    return Some(AttackMachine::<Tr>::QUANTA_PER_BIT / 2)
                }

                // The bit has metched, wait for the next
                return Some(AttackMachine::<Tr>::QUANTA_PER_BIT)
            }

            AttackCmd::WaitBuffered => {
                let value = self.buffer.to_u32();
                self.buffer.clean();
                Some(AttackMachine::<Tr>::QUANTA_PER_BIT * value)
            }
        }
    }
}
