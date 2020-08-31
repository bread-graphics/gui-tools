// MIT/Apache2 License

use crate::geometry::Pixel;
use core::{fmt, sync::atomic::{AtomicUsize, Ordering}};
use euclid::{Point2D, Size2D};
use storagevec::StorageVec;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "alloc")]
use core::any::Any;

/// The type of the event, but without the inlined data.
#[derive(Debug, Copy, Clone)]
pub enum EventTypeMask {
    Resized,
    Clicked,
    Str(&'static str),
}

/// The type of the event. This should be a unique value that can be used to identify a "class" of
/// events. It should also be able to contain some arguments associated with the event, in order
/// to ensure arguments are able to be kept on the stack.
#[derive(Debug)]
pub enum EventType {
    /// This widget has been resized.
    Resized {
        old: Size2D<u32, Pixel>,
        new: Size2D<u32, Pixel>,
    },
    /// This widget has been clicked.
    Clicked(Point2D<u32, Pixel>),
    /// A custom event, identified by a string.
    Str(&'static str),
}

impl EventType {
    #[inline]
    pub fn ty(&self) -> EventTypeMask {
        match self {
            Self::Resized { .. } => EventTypeMask::Resized,
            Self::Clicked(_) => EventTypeMask::Clicked,
            Self::Str(s) => EventTypeMask::Str(s),
        }
    }
}

// global event id
static EVENT_ID: AtomicUsize = AtomicUsize::new(0);

/// An event.
pub struct Event {
    ty: EventType,
    #[cfg(feature = "alloc")]
    data: Vec<Box<dyn Any + Send + Sync>>,
    // the ID of the window that sent this event, or None
    // if no window sent this event
    sender: Option<usize>,
    // unique ID of an event
    id: usize,
    // events that must be run first
    dependent_evs: StorageVec<usize, 2>,
    // is this a termination event?
    is_terminator: bool,
}

impl fmt::Debug for Event {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debugger = f.debug_struct("Event");
        debugger.field("ty", &self.ty);
        debugger.field("sender", &self.sender);
        debugger.field("id", &self.id);
        debugger.field("dependent_evs", &self.dependent_evs);
        debugger.field("is_terminator", &self.is_terminator);
        #[cfg(feature = "alloc")]
        debugger.field("data", &"..");
        debugger.finish()
    }
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
            is_terminator: false,
            #[cfg(feature = "alloc")]
            data: Vec::new(),
        }
    }

    ///	Is this event a termination event?
    #[inline]
    pub fn is_terminator(&self) -> bool {
        self.is_terminator
    }

    /// Set whether or not this event is a termination event.
    #[inline]
    pub fn set_is_terminator(&mut self, it: bool) {
        self.is_terminator = it;
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
