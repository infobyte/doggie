use crate::attack_errors::AttackError;
use crate::commands::{AttackCmd, FastBitStack};
use crate::tranceiver::Tranceiver;
use crate::TranceiverState;

pub enum HandleResult {
    Wait { quantas: u32 },
    Stop,
    WaitForSoF,
}

const MAX_ATTACK_SIZE: usize = 64;

pub fn new_attack_buf() -> [AttackCmd; MAX_ATTACK_SIZE] {
    [AttackCmd::None; MAX_ATTACK_SIZE]
}

pub struct AttackMachine<Tr>
where
    Tr: Tranceiver,
{
    index: usize,
    attack: [AttackCmd; MAX_ATTACK_SIZE],
    pub tranceiver: Tr,
    buffer: FastBitStack,
    buffer_value: u32,
    bit_stuffing_cnt: u8,
    bit_stuffing_polarity: bool,
    bit_stuffing_active: bool,
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
            attack: new_attack_buf(),
            tranceiver,
            buffer: FastBitStack::new(),
            buffer_value: 0,
            bit_stuffing_cnt: 0,
            bit_stuffing_polarity: true,
            bit_stuffing_active: false,
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
                self.next_state.set_tx(stream.pop());
            }
            AttackCmd::MulBuffered { mult } => {
                self.buffer_value *= mult as u32;

                self.next_cmd();
                self.pre_calculate();
            }
            AttackCmd::SubBuffered { sub } => {
                self.buffer_value -= sub;

                self.next_cmd();
                self.pre_calculate();
            }
            AttackCmd::WaitBuffered => {
                if self.buffer_value == 0 {
                    self.index += 1;
                    // We shouldn't have two WaitBuffered together
                    self.pre_calculate();
                } else {
                    self.attack[self.index] = AttackCmd::Wait {
                        bits: (self.buffer_value - 1) as usize,
                    };
                }
            }
            AttackCmd::WaitForSof => {
                // On WaifForSof we need to return control to the core
                // and it will come after start of frame, so we need to
                // prepare the state after the SoF
                self.index += 1;
                self.pre_calculate();
                self.index -= 1;
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

                let finished = *len <= 0;
                if finished {
                    self.buffer_value = self.buffer.value();
                    self.buffer.clean();
                }

                Ok(finished)
            }
            AttackCmd::None => Err(()),
            // AttackCmd::WaitForEof => {
            //     // Wait for EOF (7) + IFS (3) recessives
            //     if self.bit_stuffing_cnt >= 7 + 3 && self.bit_stuffing_polarity {
            //         self.bit_stuffing_cnt = 0;

            //         Ok(true)
            //     } else {
            //         Ok(false)
            //     }
            // }
            _ => Ok(false),
        }
    }

    #[inline(always)]
    fn next_cmd(&mut self) -> bool {
        self.index += 1;

        return self.index < self.attack.len();
    }

    #[inline(always)]
    fn pre_calculate_bs(&mut self) {
        match self.attack[self.index] {
            AttackCmd::Send { stream } => {
                self.next_state.set_tx(!self.bit_stuffing_polarity);
            }
            AttackCmd::Force { stream } => {
                self.next_state.set_force(!self.bit_stuffing_polarity);
            }
            AttackCmd::WaitForEof => {
                if self.bit_stuffing_cnt >= 7 + 3 && self.bit_stuffing_polarity {
                    self.bit_stuffing_cnt = 0;

                    self.next_cmd();
                } else {
                    return;
                }
            }
            _ => {}
        }

        self.bit_stuffing_cnt = 0;
        self.bit_stuffing_active = true;
    }

    #[inline(always)]
    pub fn handle(&mut self) -> HandleResult {
        // debug!("{:?}", defmt::Debug2Format(&self.attack[self.index]));
        // We have an special case for the WaitForSof as we already
        // have prepared the following state, we return the control
        // to the core until the Sof is found
        if let AttackCmd::WaitForSof = self.attack[self.index] {
            self.index += 1;
            self.bit_stuffing_cnt = 0;
            return HandleResult::WaitForSoF;
        }

        if self.on_start {
            self.tranceiver.apply(&self.next_state);
            self.on_start = false;

            HandleResult::Wait { quantas: 1 }
        } else {
            let rx = self.tranceiver.get_rx();

            if rx == self.bit_stuffing_polarity {
                self.bit_stuffing_cnt += 1;
            } else {
                self.bit_stuffing_cnt = 1;
                self.bit_stuffing_polarity = rx;
            }

            if !self.bit_stuffing_active {
                // handle_middle return false if we finished the attack
                match self.handle_middle(rx) {
                    Ok(true) => {
                        if !self.next_cmd() {
                            return HandleResult::Stop;
                        }
                    }
                    Err(_) => return HandleResult::Stop,
                    Ok(_) => {}
                }
            } else {
                self.bit_stuffing_active = false;
            }

            // Bit stuffing
            if self.bit_stuffing_cnt >= 5 {
                self.pre_calculate_bs();
            } else {
                // pre_calculate
                self.pre_calculate();
            }

            self.on_start = true;

            HandleResult::Wait { quantas: 7 }
        }
    }
}
