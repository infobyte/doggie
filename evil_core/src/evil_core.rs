use core::u32;

use defmt::info;

use crate::attack_errors::AttackError;
use crate::attack_machine::AttackMachine;
pub use crate::bsp::EvilBsp;
pub use crate::can::CanBitrates;
use crate::clock::TicksClock;
use crate::commands::AttackCmd;
use crate::tranceiver::Tranceiver;

pub type BoardSpecificAttackFn<C, T> = fn(core: &mut EvilCore<C, T>);

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

        info!("Ticks Per Quantum: {}", ticks_per_quantum);

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

    pub fn arm(&mut self, attack: &[AttackCmd]) -> Result<(), AttackError> {
        self.machine.arm(attack)
    }

    pub fn board_specific_attack(&mut self) {
        (self.board_specific_attack_fn)(self);
    }

    #[inline(always)]
    pub fn attack(&mut self) {
        self.machine.tranceiver.wait_for_sof();

        self.attack_on_sof()
    }

    #[inline(always)]
    pub fn attack_on_sof(&mut self) {
        let mut next_instant = self.clock.ticks() - self.sof_offset_ticks;

        loop {
            let wait_quantas_opt = self.machine.handle();

            match wait_quantas_opt {
                Some(wait_quantas) => {
                    if wait_quantas == 0 {
                        continue;
                    }

                    next_instant =
                        Clock::add_ticks(next_instant, wait_quantas * self.ticks_per_quantum);
                }
                None => return,
            };

            while next_instant > self.clock.ticks() {}
        }
    }
}
