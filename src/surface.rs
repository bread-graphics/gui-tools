// MIT/Apache2 License

//! This module mostly re-exports items from `chalkboard`.

use chalkboard::Result as CResult;
pub use chalkboard::{
    color::Color,
    fill::FillRule,
    geometry::{Angle, BezierCurve, GeometricArc, Line, Point, Rectangle},
    path::{Path, PathSegment, PathSegmentType},
    surface::{Surface, SurfaceFeatures},
};

#[cfg(feature = "breadx")]
use breadx::display::{name::NameConnection, DisplayConnection};
#[cfg(feature = "breadx")]
use chalkboard::breadx::FallbackBreadxSurface;

#[cfg(all(feature = "breadx", feature = "xrender"))]
use chalkboard::breadx::RenderBreadxSurface;

/// An enum of types used by `gui-tools` that implement the `Surface` trait.
pub enum SurfaceSum<'dpy> {
    #[cfg(feature = "breadx")]
    FallbackBreadx(FallbackBreadxSurface<'dpy, NameConnection>),
    #[cfg(all(feature = "breadx", feature = "xrender"))]
    XrenderBreadx(RenderBreadxSurface<'dpy, DisplayConnection>),
    #[cfg(windows)]
    YawwGdi(chalkboard::yaww::YawwGdiSurface<'dpy>),
    Dynamic(Box<dyn Surface + Send + 'dpy>),
}

macro_rules! impl_fn_body {
    ($fname: ident, $self: expr, $($arg: expr),*) => {{
        match $self {
            #[cfg(feature = "breadx")]
            SurfaceSum::FallbackBreadx(b) => b.$fname($($arg),*),
            #[cfg(all(feature = "breadx", feature = "xrender"))]
            SurfaceSum::XrenderBreadx(x) => x.$fname($($arg),*),
            #[cfg(windows)]
            SurfaceSum::YawwGdi(g) => g.$fname($($arg),*),
            SurfaceSum::Dynamic(d) => d.$fname($($arg),*),
        }
    }}
}

impl<'dpy> Surface for SurfaceSum<'dpy> {
    #[inline]
    fn features(&self) -> SurfaceFeatures {
        impl_fn_body!(features, self,)
    }
    #[inline]
    fn set_stroke(&mut self, color: Color) -> CResult {
        impl_fn_body!(set_stroke, self, color)
    }
    #[inline]
    fn set_fill(&mut self, rule: FillRule) -> CResult {
        impl_fn_body!(set_fill, self, rule)
    }
    #[inline]
    fn set_line_width(&mut self, width: usize) -> CResult {
        impl_fn_body!(set_line_width, self, width)
    }
    #[inline]
    fn flush(&mut self) -> CResult {
        impl_fn_body!(flush, self,)
    }
    #[inline]
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> CResult {
        impl_fn_body!(draw_line, self, x1, y1, x2, y2)
    }
    #[inline]
    fn draw_lines(&mut self, lines: &[Line]) -> CResult {
        impl_fn_body!(draw_lines, self, lines)
    }
    #[inline]
    fn draw_path(&mut self, path: Path) -> CResult {
        impl_fn_body!(draw_path, self, path)
    }
    #[inline]
    fn draw_bezier_curve(&mut self, curve: BezierCurve) -> CResult {
        impl_fn_body!(draw_bezier_curve, self, curve)
    }
    #[inline]
    fn draw_bezier_curves(&mut self, curves: &[BezierCurve]) -> CResult {
        impl_fn_body!(draw_bezier_curves, self, curves)
    }
    #[inline]
    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> CResult {
        impl_fn_body!(draw_rectangle, self, x1, y1, x2, y2)
    }
    #[inline]
    fn draw_rectangles(&mut self, rects: &[Rectangle]) -> CResult {
        impl_fn_body!(draw_rectangles, self, rects)
    }
    #[inline]
    fn draw_arc(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> CResult {
        impl_fn_body!(draw_arc, self, x1, y1, x2, y2, start, end)
    }
    #[inline]
    fn draw_arcs(&mut self, arcs: &[GeometricArc]) -> CResult {
        impl_fn_body!(draw_arcs, self, arcs)
    }
    #[inline]
    fn draw_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> CResult {
        impl_fn_body!(draw_ellipse, self, x1, y1, x2, y2)
    }
    #[inline]
    fn draw_ellipses(&mut self, rects: &[Rectangle]) -> CResult {
        impl_fn_body!(draw_ellipses, self, rects)
    }
    #[inline]
    fn fill_polygon(&mut self, points: &[Point]) -> CResult {
        impl_fn_body!(fill_polygon, self, points)
    }
    #[inline]
    fn fill_path(&mut self, path: Path) -> CResult {
        impl_fn_body!(fill_path, self, path)
    }
    #[inline]
    fn fill_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> CResult {
        impl_fn_body!(fill_rectangle, self, x1, y1, x2, y2)
    }
    #[inline]
    fn fill_rectangles(&mut self, rects: &[Rectangle]) -> CResult {
        impl_fn_body!(fill_rectangles, self, rects)
    }
    #[inline]
    fn fill_arc(
        &mut self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        start: Angle,
        end: Angle,
    ) -> CResult {
        impl_fn_body!(fill_arc, self, x1, y1, x2, y2, start, end)
    }
    #[inline]
    fn fill_arcs(&mut self, arcs: &[GeometricArc]) -> CResult {
        impl_fn_body!(fill_arcs, self, arcs)
    }
    #[inline]
    fn fill_ellipse(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> CResult {
        impl_fn_body!(fill_ellipse, self, x1, y1, x2, y2)
    }
    #[inline]
    fn fill_ellipses(&mut self, rects: &[Rectangle]) -> CResult {
        impl_fn_body!(fill_ellipses, self, rects)
    }
}
