use crate::commands::{AttackCmd, FastBitStack};
use crate::attack_errors::AttackError;
use crate::tranceiver::Tranceiver;

const MAX_ATTACK_SIZE: usize = 32;


enum State {
    START,
    MIDDLE,
    END,
}

pub struct AttackMachine<Tr>
where
    Tr: Tranceiver,
{
    index: usize,
    attack: [AttackCmd; MAX_ATTACK_SIZE],
    pub tranceiver: Tr,
    buffer: FastBitStack,
    bit_stuffing_cnt: u8,
    bit_stuffing_polarity: bool,
    state: State,
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
            buffer: FastBitStack::new(),
            bit_stuffing_cnt: 0,
            bit_stuffing_polarity: true,
            state: State::START,
        }
    }


    pub fn arm(&mut self, attack: &[AttackCmd]) -> Result<(), AttackError> {
        self.index = 0;
        self.state = State::START;
        self.bit_stuffing_polarity = true;
        self.bit_stuffing_cnt = 0;
        self.buffer.clean();

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
    fn next_cmd(&mut self) -> bool {
        self.index += 1;

        return self.index < self.attack.len()
    }

    #[inline]
    pub fn handle_start(&mut self) -> bool {
        match self.attack[self.index] {
            AttackCmd::Wait { ref mut bits } => {
                // Wait for one bit more
                if *bits > 0 {
                    *bits -= 1;
                }

                true
            },
            AttackCmd::Force { ref mut stream } => {
                self.tranceiver.set_force(stream.pop());
                true
            },
            AttackCmd::Send { ref mut stream } => {
                self.tranceiver.set_tx(!stream.pop());
                true
            },
            AttackCmd::WaitBuffered => {
                let value = self.buffer.value() * 8;
                self.buffer.clean();

                if value == 0 {
                    self.index += 1;
                    self.handle_start()
                } else {
                    self.attack[self.index] = AttackCmd::Wait { bits: (value - 1) as usize };
                    true
                }
            }
            _ => true
        }
    }

    #[inline]
    pub fn handle_middle(&mut self) -> bool {
        match self.attack[self.index] {
            AttackCmd::Match { ref mut stream } => {
                // Check the next bit with the RX
                // If it doesn't match, finish the attack
                stream.pop() == self.tranceiver.get_rx()
            },
            AttackCmd::Read { ref mut len } => {
                self.buffer.push(self.tranceiver.get_rx());
                *len -= 1;
                true
            },
            _ => true
        }
    }

    #[inline]
    pub fn handle_end(&mut self) -> bool {
        match self.attack[self.index] {
            AttackCmd::None => {
                false
            },
            AttackCmd::Wait { bits } => {
                if bits <= 0 {
                    self.next_cmd()
                } else {
                    true
                }

            }
            AttackCmd::Force { ref mut stream } => {
                if stream.len() <= 0 {
                    self.tranceiver.set_force(false);
                    self.next_cmd()
                } else {
                    true
                }
            },
            AttackCmd::Send { ref mut stream } => {
                if stream.len() <= 0 {
                    self.tranceiver.set_tx(true);
                    self.next_cmd()
                } else {
                    true
                }
            },
            AttackCmd::Match { ref mut stream } => {
                // Check the next bit with the RX
                if stream.len() <= 0 {
                    self.next_cmd()
                } else {
                    true
                }
            },
            AttackCmd::Read { ref mut len } => {
                if *len <= 0 {
                    self.next_cmd()
                } else {
                    true
                }
            },
            _ => true
        }
    }

    #[inline]
    pub fn handle(&mut self) -> Option<u32> {
        match self.state {
            State::START => {
                self.state = State::MIDDLE;

                if self.handle_start() {
                    Some(2)
                } else {
                    None
                }
            },
            State::MIDDLE => {
                let rx = self.tranceiver.get_rx();

                if rx == self.bit_stuffing_polarity {
                    self.bit_stuffing_cnt += 1;
                } else {
                    self.bit_stuffing_cnt = 1;
                    self.bit_stuffing_polarity = rx;
                }

                self.state = State::END;
                
                if self.handle_middle() {
                    Some(6)
                } else {
                    None
                }
            },
            State::END => {
                if !self.handle_end() {
                    return None
                }

                self.state = State::START;

                if self.bit_stuffing_cnt >= 5 {
                    self.bit_stuffing_cnt = 0;

                    Some(AttackMachine::<Tr>::QUANTA_PER_BIT as u32)
                } else {
                    Some(0)
                }
            }
        }
    }
}
