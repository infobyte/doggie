use core::u32;

use defmt::{debug, info, println};

use super::bsp::{CanBitrates, EvilBsp, TicksClock, Tranceiver};
use super::machine::{commands::AttackCmd, AttackError, AttackMachine, HandleResult};

pub type BoardSpecificAttackFn<C, T> = fn(core: &mut EvilCore<C, T>) -> bool;

pub struct EvilCore<Clock, Tr>
where
    Clock: TicksClock,
    Tr: Tranceiver,
{
    pub clock: Clock,
    ticks_per_quantum: u32,
    sof_offset_ticks: u32,
    machine: AttackMachine<Tr>,
    board_specific_attack_fn: BoardSpecificAttackFn<Clock, Tr>,
}

impl<Clock, Tr> EvilCore<Clock, Tr>
where
    Clock: TicksClock,
    Tr: Tranceiver,
{
    /// Create a new instance of EvilCore
    ///
    /// # Arguments
    ///
    /// * `board_specific_attack_fn` - Board-specific attack function.
    /// Should disable interrupts. and call core.attack()
    pub fn new(
        bsp: EvilBsp<Clock, Tr>,
        baudrate: CanBitrates,
        sof_offset_ns: u32,
        board_specific_attack_fn: BoardSpecificAttackFn<Clock, Tr>,
    ) -> Self {
        let (clock, tr) = bsp.split();
        let machine = AttackMachine::new(tr);
        let ticks_per_quantum = ((baudrate.to_period_ns() / 1_000)
            * (Clock::TICKS_PER_SEC / 1_000_000))
            / AttackMachine::<Tr>::QUANTA_PER_BIT;
        let sof_offset_ticks = (Clock::TICKS_PER_SEC / 1_000_000 * sof_offset_ns) / 1_000;

        info!("Evil core initialization:");
        println!("\tQuantas per bit: {}", AttackMachine::<Tr>::QUANTA_PER_BIT);
        println!("\tTicks per second: {}", Clock::TICKS_PER_SEC);
        println!("\tTicks Per Quantum: {}", ticks_per_quantum);
        println!(
            "\tSoF offset: {}ns = {}ticks",
            sof_offset_ns, sof_offset_ticks
        );

        EvilCore {
            clock,
            ticks_per_quantum,
            sof_offset_ticks,
            machine,
            board_specific_attack_fn,
        }
    }

    pub fn set_baudrate(&mut self, baudrate: CanBitrates) {
        let ticks_per_quantum = ((baudrate.to_period_ns() / 1_000)
            * (Clock::TICKS_PER_SEC / 1_000_000))
            / AttackMachine::<Tr>::QUANTA_PER_BIT;

        info!("Ticks Per Quantum: {}", ticks_per_quantum);

        self.ticks_per_quantum = ticks_per_quantum;
    }

    pub fn get_baudrate(&self) -> CanBitrates {
        let baudrate = (self.ticks_per_quantum * AttackMachine::<Tr>::QUANTA_PER_BIT) * 1_000
            / (Clock::TICKS_PER_SEC / 1_000_000);

        CanBitrates::from_period_ns(baudrate)
    }

    #[link_section = ".rwtext"]
    pub fn board_specific_attack(
        &mut self,
        attack: &[AttackCmd],
        mut successes: usize,
        retries: Option<usize>,
    ) -> bool {
        if successes <= 0 {
            return false;
        }

        let mut retries_cnt: usize = 0;

        while retries.is_none_or(|retries_value| retries_cnt < retries_value) && successes > 0 {
            // debug!(
            //     "Attempting attack with {} retries left and {} successes left",
            //     retries_cnt, successes
            // );
            self.machine.arm(attack);
            if (self.board_specific_attack_fn)(self) {
                successes -= 1;
                retries_cnt = 0;
            } else {
                retries_cnt += 1;
            }
        }
        // debug!("Attack finished. Success {}", successes <= 0);
        successes <= 0
    }

    #[inline(always)]
    pub fn attack(&mut self) -> bool {
        // Set inital time
        let mut next_instant = self.clock.ticks();

        loop {
            // Handle the command
            let handle_result = self.machine.handle();

            // Wait for quantas, wait for start of frame or exit
            match handle_result {
                HandleResult::Wait { quantas } => {
                    if quantas == 0 {
                        continue;
                    }

                    let interval = quantas * self.ticks_per_quantum;

                    // Wait for to the next interval
                    loop {
                        if Clock::sub_ticks(self.clock.ticks(), next_instant) >= interval {
                            break;
                        }
                    }

                    next_instant = Clock::add_ticks(next_instant, interval);
                }
                HandleResult::Stop => {
                    // debug!("[core] Attack finished");
                    let res = self.machine.has_finished();
                    self.machine.reset();
                    return res;
                }
                HandleResult::WaitForSoF => {
                    // Wait for SoF and restart the counter
                    self.machine.tranceiver.wait_for_sof();
                    next_instant = Clock::sub_ticks(self.clock.ticks(), self.sof_offset_ticks);
                }
            };
        }
    }
}
