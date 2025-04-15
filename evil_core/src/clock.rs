pub trait TicksClock {
    const TICKS_PER_SEC: u32;

    fn ticks(&self) -> u32;

    fn add_ticks(t1: u32, t2: u32) -> u32;
}
