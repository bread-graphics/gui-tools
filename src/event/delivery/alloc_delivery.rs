// MIT/Apache2 License

use super::EventDelivery;
use crate::{
    event::Event,
    mutex::{MutexGuard, ShimMutex as Mutex},
};
use alloc::collections::VecDeque;
use core::{
    sync::atomic::{AtomicBool, Ordering},
    task::Waker,
};
use crossbeam_queue::{ArrayQueue, PushError};

#[cfg(feature = "std")]
use conquer_once::OnceCell;

/// A mechanism for delivering events that uses a deque structure internally.
pub struct AllocEventDelivery {
    event_queue: ArrayQueue<Event>,
    #[cfg(feature = "std")]
    waker: OnceCell<Waker>,
}

impl EventDelivery for AllocEventDelivery {
    #[inline]
    fn new() -> Self {
        Self {
            event_queue: ArrayQueue::new(30),
            #[cfg(feature = "std")]
            waker: OnceCell::uninit(),
        }
    }

    #[inline]
    fn add_events<I: IntoIterator<Item = Event>>(&self, events: I) {
        events.into_iter().for_each(|e| {
            self.event_queue.push(e).expect("Unexpected event overflow");
        });

        #[cfg(feature = "std")]
        {
            if let Some(waker) = self.waker.get() {
                waker.wake_by_ref()
            }
        }
    }

    #[inline]
    fn pop_event(&self) -> Option<Event> {
        self.event_queue.pop().ok()
    }

    #[inline]
    fn pending(&self) -> bool {
        !self.event_queue.is_empty()
    }

    #[inline]
    fn set_waker(&self, waker: Waker) {
        #[cfg(feature = "std")]
        self.waker.init_once(|| waker);
    }
}
