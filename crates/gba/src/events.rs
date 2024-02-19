use std::{cell::RefCell, rc::Rc};

use arm::emu::Cycles;
use arrayvec::ArrayVec;

#[derive(Default, Clone)]
pub(crate) struct SharedGbaScheduler {
    inner: Rc<RefCell<GbaScheduler>>,
}

impl SharedGbaScheduler {
    pub fn schedule(&mut self, event: GbaEvent, cycles: Cycles) {
        self.inner.borrow_mut().schedule(event, cycles)
    }

    pub fn tick(&mut self, cycles: &mut Cycles) -> Option<GbaEvent> {
        self.inner.borrow_mut().tick(cycles)
    }

    pub fn clear(&mut self) {
        self.inner.borrow_mut().clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GbaEvent {
    HDraw,
    HBlank,

    // FIXME replace this with something else once we have
    //       another event. Right now it's only used in tests.
    #[allow(dead_code)]
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    cycles: Cycles,
    event: GbaEvent,
}

#[derive(Default)]
pub struct GbaScheduler {
    entries: ArrayVec<Entry, 64>,
}

impl GbaScheduler {
    pub fn schedule(&mut self, event: GbaEvent, cycles: Cycles) {
        let mut new_entry = Entry { cycles, event };
        if self.entries.is_empty() {
            self.entries.push(new_entry);
            return;
        }

        let mut slot = self.entries.len();

        for (idx, entry) in self.entries.iter_mut().enumerate().rev() {
            if entry.cycles <= new_entry.cycles {
                new_entry.cycles -= entry.cycles;
                slot = idx;
            } else {
                entry.cycles -= new_entry.cycles;
                break;
            }
        }

        self.entries.insert(slot, new_entry);
    }

    pub fn tick(&mut self, cycles: &mut Cycles) -> Option<GbaEvent> {
        if let Some(entry) = self.entries.last_mut() {
            if entry.cycles <= *cycles {
                *cycles -= entry.cycles;
                return self.entries.pop().map(|entry| entry.event);
            } else {
                entry.cycles -= *cycles;
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod test {
    use arm::emu::Cycles;

    use crate::events::Entry;

    use super::{GbaEvent, GbaScheduler};

    #[test]
    fn test_scheduling_empty() {
        let mut scheduler = GbaScheduler::default();
        scheduler.schedule(GbaEvent::HDraw, Cycles::from(12));
        assert_eq!(
            scheduler.entries.last(),
            Some(&Entry {
                event: GbaEvent::HDraw,
                cycles: Cycles::from(12)
            })
        );
    }

    #[test]
    fn test_scheduling_after() {
        let mut scheduler = GbaScheduler::default();
        scheduler.schedule(GbaEvent::HDraw, Cycles::from(12));
        scheduler.schedule(GbaEvent::HBlank, Cycles::from(16));

        assert_eq!(
            scheduler.entries.get(1),
            Some(&Entry {
                event: GbaEvent::HDraw,
                cycles: Cycles::from(12)
            })
        );

        assert_eq!(
            scheduler.entries.first(),
            Some(&Entry {
                event: GbaEvent::HBlank,
                cycles: Cycles::from(4)
            })
        );
    }

    #[test]
    fn test_scheduling_before() {
        let mut scheduler = GbaScheduler::default();
        scheduler.schedule(GbaEvent::HBlank, Cycles::from(16));
        scheduler.schedule(GbaEvent::HDraw, Cycles::from(12));

        assert_eq!(
            scheduler.entries.get(1),
            Some(&Entry {
                event: GbaEvent::HDraw,
                cycles: Cycles::from(12)
            })
        );

        assert_eq!(
            scheduler.entries.first(),
            Some(&Entry {
                event: GbaEvent::HBlank,
                cycles: Cycles::from(4)
            })
        );
    }

    #[test]
    fn test_scheduling_between() {
        let mut scheduler = GbaScheduler::default();
        scheduler.schedule(GbaEvent::HBlank, Cycles::from(16));
        scheduler.schedule(GbaEvent::HDraw, Cycles::from(12));
        scheduler.schedule(GbaEvent::Test, Cycles::from(14));

        assert_eq!(
            scheduler.entries.get(2),
            Some(&Entry {
                event: GbaEvent::HDraw,
                cycles: Cycles::from(12)
            })
        );

        assert_eq!(
            scheduler.entries.get(1),
            Some(&Entry {
                event: GbaEvent::Test,
                cycles: Cycles::from(2)
            })
        );

        assert_eq!(
            scheduler.entries.first(),
            Some(&Entry {
                event: GbaEvent::HBlank,
                cycles: Cycles::from(2)
            })
        );
    }

    #[test]
    fn test_scheduling_tick() {
        let mut scheduler = GbaScheduler::default();
        scheduler.schedule(GbaEvent::HBlank, Cycles::from(16));
        scheduler.schedule(GbaEvent::HDraw, Cycles::from(12));
        scheduler.schedule(GbaEvent::Test, Cycles::from(14));

        let mut cycles = Cycles::from(1);
        assert_eq!(scheduler.tick(&mut cycles), None);

        let mut cycles = Cycles::from(11);
        assert_eq!(scheduler.tick(&mut cycles), Some(GbaEvent::HDraw));
        assert_eq!(cycles, Cycles::zero());

        let mut cycles = Cycles::from(4);
        assert_eq!(scheduler.tick(&mut cycles), Some(GbaEvent::Test));
        assert_eq!(cycles, Cycles::from(2));
        assert_eq!(scheduler.tick(&mut cycles), Some(GbaEvent::HBlank));
        assert_eq!(cycles, Cycles::zero());
    }
}
