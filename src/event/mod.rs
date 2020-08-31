// MIT/Apache2 License

pub mod delivery;
mod event;
pub mod source;

pub use event::*;
pub use source::*;

/// Operations of the event loop
#[derive(Copy, Clone)]
pub enum EventLoopAction {
    Continue,
    Break,
}

impl EventLoopAction {
    #[inline]
    pub fn is_break(&self) -> bool {
        match self {
            Self::Continue => false,
            Self::Break => true,
        }
    }
}
