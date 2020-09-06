// MIT/Apache2 License

pub mod delivery;
mod event;
pub mod source;

pub use event::*;
pub use source::*;

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
