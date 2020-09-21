// MIT/Apache2 License

//! Basic geometric primitives.

use euclid::{point2, size2, Angle, Point2D, Size2D};

/// Unit for pixels or screen area. This is used as a type argument for the
/// `euclid` geometric types.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Pixel;

/// A rectangle consisting of a signed origin point and an unsigned width and height.
/// This is used for some operations within `gui-tools`.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Rectangle {
    pub origin: Point2D<i32, Pixel>,
    pub size: Size2D<u32, Pixel>,
}

impl Rectangle {
    #[inline]
    #[must_use]
    pub fn x(&self) -> i32 {
        self.origin.x
    }

    #[inline]
    #[must_use]
    pub fn y(&self) -> i32 {
        self.origin.y
    }

    #[inline]
    #[must_use]
    pub fn width(&self) -> u32 {
        self.size.width
    }

    #[inline]
    #[must_use]
    pub fn height(&self) -> u32 {
        self.size.height
    }

    #[inline]
    #[must_use]
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            origin: point2(x, y),
            size: size2(width, height),
        }
    }
}

/// A geometric arc, consisting of a bounding rectangle and two angles. It is named `GeometricArc`
/// to differentiate it from the standard library type.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct GeometricArc {
    pub bounding_rect: Rectangle,
    pub start_angle: Angle<f32>,
    pub end_angle: Angle<f32>,
}
