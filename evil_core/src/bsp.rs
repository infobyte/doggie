use crate::clock::TicksClock;
use crate::tranceiver::Tranceiver;


pub struct EvilBsp<Clock, Tr>
where
    Clock: TicksClock,
    Tr: Tranceiver,
{
    pub clock: Clock,
    pub tr: Tr,
}

impl<Clock, Tr> EvilBsp<Clock, Tr>
where
    Clock: TicksClock,
    Tr: Tranceiver,
{
    pub fn new(clock: Clock, tr: Tr) -> Self {
        EvilBsp {
            clock,
            tr
        }
    }

    pub fn split(self) -> (Clock, Tr) {
        (self.clock, self. tr)
    }
}
