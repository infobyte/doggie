use crate::TranceiverState;

pub trait Tranceiver {
    fn set_tx(&mut self, state: bool);

    fn get_rx(&self) -> bool;

    fn set_force(&mut self, state: bool);

    #[inline(always)]
    fn wait_for_sof(&self) {
        while self.get_rx() {}
    }

    #[inline(always)]
    fn apply(&mut self, state: &TranceiverState) {
        self.set_force(state.force);
        self.set_tx(state.tx);
    }
}
