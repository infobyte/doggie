use crate::attack_errors::AttackError;
use crate::commands::{AttackCmd, FastBitStack};
use crate::tranceiver::Tranceiver;
use crate::TranceiverState;

const MAX_ATTACK_SIZE: usize = 32;

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
    on_start: bool,
    next_state: TranceiverState,
}

impl<Tr> AttackMachine<Tr>
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
            on_start: true,
            next_state: TranceiverState::new(),
        }
    }

    pub fn arm(&mut self, attack: &[AttackCmd]) -> Result<(), AttackError> {
        self.index = 0;
        self.on_start = true;
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

        self.pre_calculate();

        Ok(())
    }

    #[inline(always)]
    fn pre_calculate(&mut self) {
        /* Pre calculate the next state of the tranceiver */
        match self.attack[self.index] {
            AttackCmd::Wait { ref mut bits } => {
                // Wait for one bit more
                if *bits > 0 {
                    *bits -= 1;
                }
            }
            AttackCmd::Force { ref mut stream } => {
                self.next_state.set_force(stream.pop());
            }
            AttackCmd::Send { ref mut stream } => {
                self.next_state.set_tx(!stream.pop());
            }
            AttackCmd::WaitBuffered => {
                let value = self.buffer.value() * 8;
                self.buffer.clean();

                if value == 0 {
                    self.index += 1;
                    // We shouldn't have two WaitBuffered together
                    self.pre_calculate();
                } else {
                    self.attack[self.index] = AttackCmd::Wait {
                        bits: (value - 1) as usize,
                    };
                }
            }
            _ => {}
        }
    }

    #[inline(always)]
    pub fn handle_middle(&mut self, rx: bool) -> Result<bool, ()> {
        /* Handle the middle of the bit with the rx state
         * Returns
         *     Err(()) => if we need to stop the execution
         *     Ok(true) => if we need to pass to the next cmd
         *     Ok(false) => If we are in the same command
         * */
        match self.attack[self.index] {
            AttackCmd::Wait { bits } => Ok(bits <= 0),
            AttackCmd::Force { ref mut stream } => {
                if stream.len() <= 0 {
                    self.next_state.set_force(false);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            AttackCmd::Send { ref mut stream } => {
                if stream.len() <= 0 {
                    self.next_state.set_tx(true);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            AttackCmd::Match { ref mut stream } => {
                // Check the next bit with the RX
                // If it doesn't match, finish the attack
                let result = rx == stream.pop();

                if !result {
                    Err(())
                } else {
                    Ok(stream.len() <= 0)
                }
            }
            AttackCmd::Read { ref mut len } => {
                self.buffer.push(rx);
                *len -= 1;

                Ok(*len <= 0)
            }
            _ => Ok(false),
        }
    }

    #[inline(always)]
    fn next_cmd(&mut self) -> bool {
        self.index += 1;

        return self.index < self.attack.len();
    }

    #[inline(always)]
    pub fn handle(&mut self) -> Option<u32> {
        if self.on_start {
            self.tranceiver.apply(&self.next_state);
            self.on_start = false;

            Some(1)
        } else {
            let rx = self.tranceiver.get_rx();

            if rx == self.bit_stuffing_polarity {
                self.bit_stuffing_cnt += 1;
            } else {
                self.bit_stuffing_cnt = 1;
                self.bit_stuffing_polarity = rx;
            }

            // handle_middle return false if we finished the attack
            match self.handle_middle(rx) {
                Ok(true) => {
                    if !self.next_cmd() {
                        return None;
                    }
                }
                Err(_) => return None,
                Ok(_) => {}
            };

            // Bit stuffing
            if self.bit_stuffing_cnt >= 5 {
                self.bit_stuffing_cnt = 0;

                Some(8)
            } else {
                self.on_start = true;
                // pre_calculate
                self.pre_calculate();
                Some(7)
            }
        }
    }
}
