// MIT/Apache2 License

use super::EventDelivery;
use crate::{event::Event, mutex::ShimMutex as Mutex};
use core::task::Waker;
use storagevec::StorageVec;

/// A simple event delivery mechanism for the package running on libcore.
#[repr(transparent)]
pub struct CoreEventDelivery {
    members: Mutex<StorageVec<Event, 12>>,
}

impl EventDelivery for CoreEventDelivery {
    #[inline]
    fn new() -> Self {
        Self {
            members: Mutex::new(StorageVec::new()),
        }
    }

    #[inline]
    fn add_events<I: IntoIterator<Item = Event>>(&self, events: I) {
        self.members.lock().extend(events);
    }

    #[inline]
    fn pop_event(&self) -> Option<Event> {
        self.members.lock().remove(0)
    }

    #[inline]
    fn pending(&self) -> bool {
        !self.members.lock().is_empty()
    }

    #[inline]
    fn set_waker(&self, _waker: Waker) { /* core doesn't support async */
    }
}
