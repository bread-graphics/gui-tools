// MIT/Apache2 License

//! Geometry primitives.

#[doc(inline)]
pub use chalkboard::geometry::*;

/// The dimensions of a window.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dimensions {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
