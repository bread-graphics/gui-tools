// MIT/Apache2 License

use super::EventDelivery;
use crate::{event::Event, mutex::ShimMutex as Mutex};
use core::task::Waker;
use crossbeam_queue::SegQueue;

/// A mechanism for delivering events that takes advantage of threaded code.
pub struct ThreadedEventDelivery {
    event_queue: SegQueue<Event>,
    waker: Mutex<Option<Waker>>,
}

impl EventDelivery for ThreadedEventDelivery {
    #[inline]
    fn new() -> Self {
        Self {
            event_queue: SegQueue::new(),
            waker: Mutex::new(None),
        }
    }

    #[inline]
    fn add_events<I: IntoIterator<Item = Event>>(&self, ev: I) {
        ev.into_iter().for_each(|e| self.event_queue.push(e));
        if let Some(w) = &*self.waker.lock() {
            w.wake_by_ref();
        }
    }

    #[inline]
    fn pop_event(&self) -> Option<Event> {
        match self.event_queue.pop() {
            Ok(x) => Some(x),
            Err(e) => {
                log::error!("{:?}", e);
                None
            }
        }
    }

    #[inline]
    fn pending(&self) -> bool {
        !self.event_queue.is_empty()
    }

    #[inline]
    fn set_waker(&self, waker: Waker) {
        *self.waker.lock() = Some(waker);
    }
}
