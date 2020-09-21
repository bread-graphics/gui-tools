// MIT/Apache2 License

use crate::{geometry::Pixel, graphics::Graphics, keyboard::KeyInfo, mouse::MouseButton};
use core::{
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};
use euclid::{Point2D, Size2D};
use storagevec::StorageVec;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};
#[cfg(feature = "alloc")]
use core::any::Any;

/// The type of the event, but without the inlined data.
///
/// This is used when the type of the event is needed, but not the data associated with
/// the event. For instance, setting the events received uses this type. The EventTypeMask
/// of an EventType can be acquired via the `ty()` method.
#[derive(Debug, Copy, Clone)]
pub enum EventTypeMask {
    NoOp,
    Resized,
    Moved,
    MouseDown,
    MouseUp,
    MouseEnterWindow,
    MouseExitWindow,
    MouseMove,
    KeyDown,
    KeyUp,
    Focus,
    Unfocus,
    Quit,
    Paint,
    Str(&'static str),
}

impl Default for EventTypeMask {
    #[inline]
    fn default() -> Self {
        Self::NoOp
    }
}

/// The type of the event. This should be a unique value that can be used to identify a "class" of
/// events. It should also be able to contain some arguments associated with the event, in order
/// to ensure arguments are able to be kept on the stack.
#[derive(Debug)]
pub enum EventType {
    /// No-op event.
    NoOp,
    /// This widget has been resized.
    Resized {
        old: Size2D<u32, Pixel>,
        new: Size2D<u32, Pixel>,
    },
    /// This widget has been moved.
    Moved {
        old: Point2D<i32, Pixel>,
        new: Point2D<i32, Pixel>,
    },
    /// This widget has been clicked.
    MouseDown(Point2D<i32, Pixel>, MouseButton),
    /// The mouse button has been released.
    MouseUp(Point2D<i32, Pixel>, MouseButton),
    /// The mouse pointer has entered the window.
    MouseEnterWindow(Point2D<i32, Pixel>),
    /// The mouse pointer has exited the window.
    MouseExitWindow(Point2D<i32, Pixel>),
    /// The mouse pointer has moved within the window.
    MouseMove(Point2D<i32, Pixel>),
    /// A key has been depressed.
    KeyDown(KeyInfo, Point2D<i32, Pixel>),
    /// A key has been released.
    KeyUp(KeyInfo, Point2D<i32, Pixel>),
    /// Focus in on this window.
    Focus,
    /// Unfocus from this window.
    Unfocus,
    /// Quit.
    Quit,
    /// The window is being painted.
    Paint(Graphics),
    /// A custom event, identified by a string.
    Str(&'static str),
}

impl Default for EventType {
    #[inline]
    fn default() -> Self {
        Self::NoOp
    }
}

unsafe impl Send for EventType {}
unsafe impl Sync for EventType {}

impl EventType {
    /// Get the `EventTypeMask` associated with an `EventType`.
    #[inline]
    pub fn ty(&self) -> EventTypeMask {
        match self {
            Self::NoOp => EventTypeMask::NoOp,
            Self::Resized { .. } => EventTypeMask::Resized,
            Self::Moved { .. } => EventTypeMask::Moved,
            Self::MouseUp(_, _) => EventTypeMask::MouseUp,
            Self::MouseDown(_, _) => EventTypeMask::MouseDown,
            Self::MouseEnterWindow(_) => EventTypeMask::MouseEnterWindow,
            Self::MouseExitWindow(_) => EventTypeMask::MouseExitWindow,
            Self::MouseMove(_) => EventTypeMask::MouseMove,
            Self::KeyDown(_, _) => EventTypeMask::KeyDown,
            Self::KeyUp(_, _) => EventTypeMask::KeyUp,
            Self::Focus => EventTypeMask::Focus,
            Self::Unfocus => EventTypeMask::Unfocus,
            Self::Quit => EventTypeMask::Quit,
            Self::Paint(_) => EventTypeMask::Paint,
            Self::Str(s) => EventTypeMask::Str(s),
        }
    }
}

// global event id
static EVENT_ID: AtomicUsize = AtomicUsize::new(0);

/// A signal used as a way of telling those subscribed to it that something has happened. Most native
/// libraries are built around using Events as a way of telling when something has happened. The `Event`
/// type acts as a combined format for all of these types of events, and is what the `gui-tools` event
/// runtime deals with.
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
    // skip the peeker loop for this event?
    skip_peeker: bool,
}

impl Default for Event {
    #[inline]
    fn default() -> Self {
        Self::new(EventType::NoOp, None)
    }
}

impl fmt::Debug for Event {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct NoItems;
        impl fmt::Debug for NoItems {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("..")
            }
        }

        let mut debugger = f.debug_struct("Event");
        debugger.field("ty", &self.ty);
        debugger.field("sender", &self.sender);
        debugger.field("id", &self.id);
        debugger.field("dependent_evs", &self.dependent_evs);
        debugger.field("is_terminator", &self.is_terminator);
        #[cfg(feature = "alloc")]
        debugger.field("data", &NoItems);
        debugger.finish()
    }
}

impl Event {
    /// Create a new event.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gui_tools::event::{Event, EventType};
    ///
    /// let father_grigori = Event::new(EventType::Str("Father Grigori"), None);
    /// ```
    #[inline]
    pub fn new(ty: EventType, sender: Option<usize>) -> Self {
        Self {
            ty,
            sender,
            id: EVENT_ID.fetch_add(1, Ordering::Acquire),
            dependent_evs: StorageVec::new(),
            is_terminator: false,
            skip_peeker: false,
            #[cfg(feature = "alloc")]
            data: Vec::new(),
        }
    }

    ///	Is this event a termination event? If the event loop receives a termination event,
    /// it will stop as soon as possible.
    #[inline]
    pub fn is_terminator(&self) -> bool {
        self.is_terminator
    }

    /// Set whether or not this event is a termination event.
    #[inline]
    pub fn set_is_terminator(&mut self, it: bool) {
        self.is_terminator = it;
    }

    /// Should this event skip the peeker loop?
    #[inline]
    pub fn skip_peeker_loop(&self) -> bool {
        self.skip_peeker
    }

    /// Set whether or not this event should skip the peeker loop.
    #[inline]
    pub fn set_skip_peeker_loop(&mut self, val: bool) {
        self.skip_peeker = val;
    }

    /// Get the type of this event.
    #[inline]
    pub fn ty(&self) -> &EventType {
        &self.ty
    }

    /// Get the sender of this event. This is often the surface associated with the event.
    #[inline]
    pub fn sender(&self) -> Option<usize> {
        self.sender
    }

    /// Get the ID of this event. The ID is unique to this event, as far as possible.
    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    /// Make this event dependent on other events.
    #[inline]
    pub fn make_dependent_on<I: IntoIterator<Item = usize>>(&mut self, dep: I) {
        self.dependent_evs.extend(dep);
    }

    /// Get the internal data of this event.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn internal_data(&self) -> &[Box<dyn Any + Send + Sync>] {
        &self.data
    }

    /// Add some internal data to this event.
    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn add_internal_data<T: Any + Send + Sync + 'static>(&mut self, data: T) {
        self.data.push(Box::new(data));
    }
}
