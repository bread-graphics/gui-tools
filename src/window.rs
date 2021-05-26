// MIT/Apache2 License

use chalkboard::Color;
use std::num::NonZeroU64;

/// A window.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Window(NonZeroU64);

impl Window {
    #[inline]
    pub fn from_raw(raw: NonZeroU64) -> Self {
        Self(raw)
    }

    #[inline]
    pub fn into_raw(self) -> NonZeroU64 {
        self.0
    }
}

/// Associated properties of a window.
#[derive(Debug, Clone, PartialEq, Default, Eq, PartialOrd, Ord, Hash)]
pub struct WindowProps {
    pub title: Option<String>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    Visible,
    Hidden,
}
