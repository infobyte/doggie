pub trait Tranceiver {
    fn set_tx(&mut self, state: bool);

    fn get_rx(&self) -> bool;

    fn set_force(&mut self, state: bool);

    fn wait_for_sof(&self) {
        while self.get_rx() {}
    }
}
