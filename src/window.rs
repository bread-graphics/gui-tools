// MIT/Apache2 License

use crate::{Color, FillRule};
use std::num::NonZeroUsize;

/// A logical window. This, in the barest terms, represents "a rectangle on the screen where the application has
/// control".
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Window {
    /// The inner value. Windows are either a pointer or a non-zero ID. We can therefore take advantage of zero
    /// value optimization using a `NonZeroUsize`.
    inner: NonZeroUsize,
}

impl Window {
    /// Get a `Window` from the raw value.
    #[inline]
    pub fn from_raw(inner: NonZeroUsize) -> Window {
        Window { inner }
    }

    /// Get the raw `NonZeroUsize` from this `Window`.
    #[inline]
    pub fn into_raw(self) -> NonZeroUsize {
        self.inner
    }
}

/// The properties that a window may have.
#[derive(Debug, Default)]
pub struct WindowProps {
    pub title: Option<String>,
    pub background: Option<FillRule>,
    pub border_color: Option<Color>,
    pub border_width: Option<usize>,
}
