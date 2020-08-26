// MIT/Apache2 License

use crate::geometry::Pixel;
use core::sync::atomic::{AtomicUsize, Ordering};
use euclid::Point2D;
use storagevec::StorageVec;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "alloc")]
use core::any::Any;

/// The type of the event. This should be a unique value that can be used to identify a "class" of
/// events. It should also be able to contain some arguments associated with the event, in order
/// to ensure arguments are able to be kept on the stack.
pub enum EventType {
    /// This widget has been clicked.
    Clicked(Point2D<u32, Pixel>),
    /// A custom event, identified by a string.
    Str(&'static str),
}

// global event id
static EVENT_ID: AtomicUsize = AtomicUsize::new(0);

/// An event.
pub struct Event {
    ty: EventType,
    #[cfg(feature = "alloc")]
    data: Vec<Box<dyn Any>>,
    // the ID of the window that sent this event, or None
    // if no window sent this event
    sender: Option<usize>,
    // unique ID of an event
    id: usize,
    // events that must be run first
    dependent_evs: StorageVec<usize, 2>,
}

impl Event {
    /// Create a new event.
    #[inline]
    pub fn new(ty: EventType, sender: Option<usize>) -> Self {
        Self {
            ty,
            sender,
            id: EVENT_ID.fetch_add(1, Ordering::Acquire),
            dependent_evs: StorageVec::new(),
            #[cfg(feature = "alloc")]
            data: Vec::new(),
        }
    }

    /// Get the type of this event.
    #[inline]
    pub fn ty(&self) -> &EventType {
        &self.ty
    }

    /// Get the sender of this event.
    #[inline]
    pub fn sender(&self) -> Option<usize> {
        self.sender
    }

    /// Get the ID of this event.
    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    /// Make this event dependent on other events.
    #[inline]
    pub fn make_dependent_on<I: IntoIterator<Item = usize>>(&mut self, dep: I) {
        self.dependent_evs.extend(dep);
    }
}
