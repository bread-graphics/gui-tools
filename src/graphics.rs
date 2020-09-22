// MIT/Apache2 License

//! This module provides the API for drawing things. For more information, see the documentation for the
//! [`Graphics`](struct.Graphics.html) object.

use crate::{
    color::Rgba,
    geometry::{GeometricArc, Pixel, Rectangle},
    image::GenericImage,
};
use core::{fmt, ptr::NonNull};
use euclid::{point2, Angle, Point2D, Size2D};
use storagevec::StorageVec;

/// The internal API for the graphics object.
pub trait GraphicsInternal {
    fn set_color(&self, clr: Rgba) -> crate::Result<()>;
    fn set_line_width(&self, lw: u32) -> crate::Result<()>;

    fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result<()>;

    #[inline]
    fn draw_lines(&self, points: &[Point2D<i32, Pixel>]) -> crate::Result<()> {
        points
            .iter()
            .step_by(2)
            .zip(points.iter().skip(1).step_by(2))
            .map(|(p1, p2)| self.draw_line(p1.x, p1.y, p2.x, p2.y))
            .collect::<crate::Result<()>>()
    }

    #[inline]
    fn draw_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        let w: i32 = width as i32;
        let h: i32 = height as i32;
        let points = [
            // top line
            point2(x, y),
            point2(x + w, y),
            // right line
            point2(x + w, y),
            point2(x + w, y + h),
            // bottom line
            point2(x + w, y + h),
            point2(x, y + h),
            // left line
            point2(x, y + h),
            point2(x, y),
        ];
        self.draw_lines(&points)
    }

    #[inline]
    fn draw_rectangles(&self, rectangles: &[Rectangle]) -> crate::Result<()> {
        rectangles
            .iter()
            .copied()
            .map(
                |Rectangle {
                     origin: Point2D { x, y, .. },
                     size: Size2D { width, height, .. },
                 }| { self.draw_rectangle(x, y, width, height) },
            )
            .collect::<crate::Result<()>>()
    }

    fn draw_arc(
        &self,
        rectleft: i32,
        rectop: i32,
        rectwidth: u32,
        rectheight: u32,
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()>;

    #[inline]
    fn draw_arcs(&self, arcs: &[GeometricArc]) -> crate::Result<()> {
        arcs.iter()
            .copied()
            .map(
                |GeometricArc {
                     bounding_rect:
                         Rectangle {
                             origin: Point2D { x, y, .. },
                             size: Size2D { width, height, .. },
                         },
                     start_angle,
                     end_angle,
                 }| self.draw_arc(x, y, width, height, start_angle, end_angle),
            )
            .collect::<crate::Result<()>>()
    }

    #[inline]
    fn draw_ellipse(
        &self,
        rectleft: i32,
        rectop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        self.draw_arc(
            rectleft,
            rectop,
            rectwidth,
            rectheight,
            Angle::radians(0.0),
            Angle::degrees(360.0),
        )
    }

    #[inline]
    fn draw_ellipses(&self, rects: &[Rectangle]) -> crate::Result<()> {
        let arcs = rects
            .iter()
            .copied()
            .map(|bounding_rect| GeometricArc {
                bounding_rect,
                start_angle: Angle::radians(0.0),
                end_angle: Angle::degrees(360.0),
            })
            .collect::<StorageVec<GeometricArc, 100>>();
        self.draw_arcs(&arcs)
    }

    fn fill_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()>;

    #[inline]
    fn fill_rectangles(&self, rects: &[Rectangle]) -> crate::Result<()> {
        rects
            .iter()
            .copied()
            .map(
                |Rectangle {
                     origin: Point2D { x, y, .. },
                     size: Size2D { width, height, .. },
                 }| { self.fill_rectangle(x, y, width, height) },
            )
            .collect::<crate::Result<()>>()
    }

    fn fill_arc(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()>;

    #[inline]
    fn fill_arcs(&self, arcs: &[GeometricArc]) -> crate::Result<()> {
        arcs.iter()
            .copied()
            .map(
                |GeometricArc {
                     bounding_rect:
                         Rectangle {
                             origin: Point2D { x, y, .. },
                             size: Size2D { width, height, .. },
                         },
                     start_angle,
                     end_angle,
                 }| { self.fill_arc(x, y, width, height, start_angle, end_angle) },
            )
            .collect::<crate::Result<()>>()
    }

    #[inline]
    fn fill_ellipse(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        self.fill_arc(
            x,
            y,
            width,
            height,
            Angle::radians(0.0),
            Angle::degrees(360.0),
        )
    }

    #[inline]
    fn fill_ellipses(&self, rects: &[Rectangle]) -> crate::Result<()> {
        let arcs = rects
            .iter()
            .copied()
            .map(|bounding_rect| GeometricArc {
                bounding_rect,
                start_angle: Angle::radians(0.0),
                end_angle: Angle::degrees(360.0),
            })
            .collect::<StorageVec<GeometricArc, 100>>();
        self.fill_arcs(&arcs)
    }

    fn image(
        &self,
        image: &dyn GenericImage,
        origin_x: i32,
        origin_y: i32,
        clip_width: u32,
        clip_height: u32,
    ) -> crate::Result<()>;
}

/// The API for drawing 2D graphics.
///
/// This object acts as a common interface between the user and the various graphics APIs
/// of the native backend libraries. It allows one to draw basic shapes, draw images, and render
/// strings of text.
///
/// This object should not be instantiated directly. The preferred source of `Graphics` structs is the
/// [`EventType::Paint`](../event/enum.EventType.html) variant provided during the runtime. If a repaint is
/// needed, it is recommended to call [`Surface::invalidate`](../surface/index.html) and then use a peeker
/// on the `EventType::Paint` listener.
#[repr(transparent)]
pub struct Graphics {
    internal: NonNull<dyn GraphicsInternal>,
}

impl Graphics {
    /// Create a new `Graphics` struct based upon a pointer to an internal graphics object. It is not
    /// recommended to use this; see above for preferred instantiation of `Graphics`.
    #[inline]
    pub fn new(internal: NonNull<dyn GraphicsInternal>) -> Self {
        Self { internal }
    }

    #[inline]
    fn internal(&self) -> &dyn GraphicsInternal {
        unsafe { self.internal.as_ref() }
    }

    /// Set the color used to draw items.
    #[inline]
    pub fn set_color<R: Into<Rgba>>(&self, clr: R) -> crate::Result<()> {
        self.internal().set_color(clr.into())
    }

    /// Set the line width used to draw lines.
    #[inline]
    pub fn set_line_width(&self, lw: u32) -> crate::Result<()> {
        self.internal().set_line_width(lw)
    }

    /// Draw a line.
    #[inline]
    pub fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result<()> {
        self.internal().draw_line(x1, y1, x2, y2)
    }

    /// Draw several lines.
    #[inline]
    pub fn draw_lines(&self, points: &[Point2D<i32, Pixel>]) -> crate::Result<()> {
        self.internal().draw_lines(points)
    }

    /// Draw a rectangle.
    #[inline]
    pub fn draw_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        self.internal().draw_rectangle(x, y, width, height)
    }

    /// Draw several rectangles.
    #[inline]
    pub fn draw_rectangles(&self, rects: &[Rectangle]) -> crate::Result<()> {
        self.internal().draw_rectangles(rects)
    }

    /// Draw an arc.
    #[inline]
    pub fn draw_arc(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        self.internal().draw_arc(
            rectleft,
            recttop,
            rectwidth,
            rectheight,
            start_angle,
            end_angle,
        )
    }

    /// Draw several arcs.
    #[inline]
    pub fn draw_arcs(&self, arcs: &[GeometricArc]) -> crate::Result<()> {
        self.internal().draw_arcs(arcs)
    }

    /// Draw an ellipse.
    #[inline]
    pub fn draw_ellipse(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        self.internal().draw_ellipse(x, y, width, height)
    }

    /// Draw several ellipses.
    #[inline]
    pub fn draw_ellipses(&self, bounding_rects: &[Rectangle]) -> crate::Result<()> {
        self.internal().draw_ellipses(bounding_rects)
    }

    /// Fill a rectangle.
    #[inline]
    pub fn fill_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        self.internal().fill_rectangle(x, y, width, height)
    }

    /// Fill several rectangles.
    #[inline]
    pub fn fill_rectangles(&self, rects: &[Rectangle]) -> crate::Result<()> {
        self.internal().fill_rectangles(rects)
    }

    /// Fill in an arc.
    #[inline]
    pub fn fill_arc(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        self.internal().fill_arc(
            rectleft,
            recttop,
            rectwidth,
            rectheight,
            start_angle,
            end_angle,
        )
    }

    /// Fill in several arcs.
    #[inline]
    pub fn fill_arcs(&self, arcs: &[GeometricArc]) -> crate::Result<()> {
        self.internal().fill_arcs(arcs)
    }

    /// Fill an ellipse.
    #[inline]
    pub fn fill_ellipse(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        self.internal().fill_ellipse(x, y, width, height)
    }

    /// Fill several ellipsi.
    #[inline]
    pub fn fill_ellipses(&self, rects: &[Rectangle]) -> crate::Result<()> {
        self.internal().fill_ellipses(rects)
    }

    /// Draw an image.
    #[inline]
    pub fn image<Backend: GenericImage, Source: AsRef<Backend>>(
        &self,
        source: &Source,
        origin_x: i32,
        origin_y: i32,
        clip_width: Option<u32>,
        clip_height: Option<u32>,
    ) -> crate::Result<()> {
        let img = source.as_ref();
        self.internal().image(
            img,
            origin_x,
            origin_y,
            clip_width.unwrap_or_else(|| img.width() as _),
            clip_height.unwrap_or_else(|| img.height() as _),
        )
    }
}

impl fmt::Debug for Graphics {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Graphics({:p})", self.internal.as_ptr())
    }
}
