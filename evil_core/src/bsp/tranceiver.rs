pub struct TranceiverState {
    pub tx: bool,
    pub force: bool,
}

impl TranceiverState {
    pub fn new() -> Self {
        Self {
            tx: true,
            force: false,
        }
    }

    pub fn set_tx(&mut self, state: bool) {
        self.tx = state;
    }

    pub fn set_force(&mut self, state: bool) {
        self.force = state;
    }
}

pub trait Tranceiver {
    fn set_tx(&mut self, state: bool);

    fn get_rx(&self) -> bool;

    fn set_force(&mut self, state: bool);

    fn set_debug(&mut self, state: bool);

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
