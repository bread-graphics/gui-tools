// MIT/Apache2 License

use crate::event::Event;
use core::task::Waker;

#[cfg(feature = "alloc")]
pub mod alloc_delivery;
pub mod core_delivery;
#[cfg(feature = "std")]
pub mod threaded_delivery;

/// A mechanism for delivering events.
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait EventDelivery: Sync {
    /// Add one or more events to this delivery system.
    fn add_events<I: IntoIterator<Item = Event>>(&self, events: I);

    /// Pop an event from the bottom of this queue.
    fn pop_event(&self) -> Option<Event>;

    /// Is there an event pending in the queue?
    fn pending(&self) -> bool;

    /// Register a waker to this event delivery system, so it can
    /// wake up whenever it receives new events.
    fn set_waker(&self, waker: Waker);

    /// Create a new event delivery system.
    fn new() -> Self;
}

/// The default event delivery system, used in the runtime.
#[cfg(not(feature = "alloc"))]
pub type DefaultEventDelivery = core_delivery::CoreEventDelivery;

/// The default event delivery system, used in the runtime.
#[cfg(feature = "alloc")]
pub type DefaultEventDelivery = alloc_delivery::AllocEventDelivery;

///// The default event delivery system, used in the runtime.
//#[cfg(feature = "std")]
//pub type DefaultEventDelivery = threaded_delivery::ThreadedEventDelivery;
