// MIT/Apache2 License

use super::EventDelivery;
use crate::{event::Event, mutex::ShimMutex as Mutex};
use alloc::collections::VecDeque;
use core::task::Waker;

/// A mechanism for delivering events that uses a deque structure internally.
#[repr(transparent)]
pub struct AllocEventDelivery {
    event_queue: Mutex<VecDeque<Event>>,
}

impl EventDelivery for AllocEventDelivery {
    #[inline]
    fn new() -> Self {
        Self {
            event_queue: Mutex::new(VecDeque::new()),
        }
    }

    #[inline]
    fn add_events<I: IntoIterator<Item = Event>>(&self, events: I) {
        let mut eq = self.event_queue.lock();
        events.into_iter().for_each(|e| eq.push_back(e));
    }

    #[inline]
    fn pop_event(&self) -> Option<Event> {
        self.event_queue.lock().pop_front()
    }

    #[inline]
    fn pending(&self) -> bool {
        !self.event_queue.lock().is_empty()
    }

    #[inline]
    fn set_waker(&self, _waker: Waker) { /* this isn't async, we don't care */
    }
}
