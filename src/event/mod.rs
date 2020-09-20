// MIT/Apache2 License

//! Events are things that can happen over the course of a `gui-tools` program.
//!
//! If surfaces are the building blocks of `gui-tools` programs, events are the electricity that powers this
//! metaphorical building. Things like key presses, clicks, and window moves are processed into events. These
//! events are usually passed along by the runtime; see documentation in the runtime module for more
//! information on this.

pub mod delivery;
mod event;

pub use event::*;

/// Operations of the event loop. This is used by peekers to give direction
/// to the event runtime while it is running.
#[derive(Copy, Clone)]
pub enum EventLoopAction {
    /// Continue operating the event loop.
    Continue,
    /// Stop the event loop as soon as possible.
    Break,
}

impl EventLoopAction {
    #[inline]
    pub(crate) fn is_break(&self) -> bool {
        match self {
            Self::Continue => false,
            Self::Break => true,
        }
    }
}
