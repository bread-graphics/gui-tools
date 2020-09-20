// MIT/Apache2 License

use super::X11Surface;
use crate::{
    color::Rgba,
    error::x11_status_to_res,
    geometry::{GeometricArc, Pixel, Rectangle},
    graphics::GraphicsInternal,
    image::GenericImage,
    runtime::Runtime,
};
use core::{convert::TryInto, mem::MaybeUninit, num::TryFromIntError, ptr::NonNull};
use cty::{c_int, c_short};
use euclid::{Angle, Point2D, Size2D};
use storagevec::StorageVec;
use x11nas::xlib::{self, Display, XArc, XGCValues, XPoint, XRectangle, XID, _XGC};

/// Trait applied to X11 surfaces that can be drawed on. Windows and pixmaps.
pub trait X11Drawable {
    fn xid(&self) -> XID;
    fn display(&self) -> NonNull<Display>;
    fn gc(&self) -> NonNull<_XGC>;
    fn runtime(&self) -> &Runtime;
    fn monitor(&self) -> c_int;
}

impl X11Drawable for X11Surface {
    #[inline]
    fn xid(&self) -> XID {
        X11Surface::xid(self)
    }
    #[inline]
    fn display(&self) -> NonNull<Display> {
        X11Surface::display(self)
    }
    #[inline]
    fn gc(&self) -> NonNull<_XGC> {
        self.graphics_context()
    }
    #[inline]
    fn runtime(&self) -> &Runtime {
        X11Surface::runtime(self)
    }
    #[inline]
    fn monitor(&self) -> c_int {
        0
    }
}

const DEGREES_TO_XLIB_UNITS: f32 = 64.0;

#[inline]
fn rects_to_xrects(rects: &[Rectangle]) -> Result<StorageVec<XRectangle, 25>, TryFromIntError> {
    rects
        .iter()
        .map(|r| {
            Ok(XRectangle {
                x: r.x().try_into()?,
                y: r.y().try_into()?,
                width: r.width().try_into()?,
                height: r.height().try_into()?,
            })
        })
        .collect::<Result<StorageVec<XRectangle, 25>, TryFromIntError>>()
}

#[inline]
fn arcs_to_xarcs(arcs: &[GeometricArc]) -> Result<StorageVec<XArc, 25>, TryFromIntError> {
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
             }| {
                Ok(XArc {
                    x: x.try_into()?,
                    y: y.try_into()?,
                    width: width.try_into()?,
                    height: height.try_into()?,
                    angle1: (start_angle.to_degrees() * DEGREES_TO_XLIB_UNITS) as c_short,
                    angle2: (end_angle.to_degrees() * DEGREES_TO_XLIB_UNITS) as c_short,
                })
            },
        )
        .collect::<Result<StorageVec<XArc, 25>, TryFromIntError>>()
}

// macro to set the properties of a GC
macro_rules! set_gc_property {
    ($self: expr, $field: ident, $value: expr, $mask: expr) => {{
        let mut props = XGCValues {
            $field: $value,
            ..unsafe { MaybeUninit::uninit().assume_init() }
        };

        x11_status_to_res($self.display(), unsafe {
            xlib::XChangeGC(
                $self.display().as_ptr(),
                $self.gc().as_ptr(),
                ($mask).try_into().unwrap(),
                &mut props,
            )
        })
    }};
}

impl<T: X11Drawable> GraphicsInternal for T {
    #[inline]
    fn set_line_width(&self, lw: u32) -> crate::Result<()> {
        set_gc_property!(self, line_width, lw as c_int, xlib::GCLineWidth)
    }

    #[inline]
    fn set_color(&self, clr: Rgba) -> crate::Result<()> {
        let clr = self.runtime().as_x11().unwrap().color_id(clr)?;
        set_gc_property!(self, foreground, clr, xlib::GCForeground)
    }

    #[inline]
    fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result<()> {
        x11_status_to_res(self.display(), unsafe {
            xlib::XDrawLine(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                x1,
                y1,
                x2,
                y2,
            )
        })
    }

    #[inline]
    fn draw_lines(&self, points: &[Point2D<i32, Pixel>]) -> crate::Result<()> {
        let mut points = points
            .iter()
            .map(|p| {
                Ok(XPoint {
                    x: p.x.try_into()?,
                    y: p.y.try_into()?,
                })
            })
            .collect::<Result<StorageVec<XPoint, 25>, TryFromIntError>>()?;

        x11_status_to_res(self.display(), unsafe {
            xlib::XDrawLines(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                points.as_mut_ptr(),
                points.len() as c_int,
                xlib::CoordModeOrigin,
            )
        })
    }

    #[inline]
    fn draw_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        x11_status_to_res(self.display(), unsafe {
            xlib::XDrawRectangle(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                x,
                y,
                width,
                height,
            )
        })
    }

    #[inline]
    fn draw_rectangles(&self, rectangles: &[Rectangle]) -> crate::Result<()> {
        let mut rects = rects_to_xrects(rectangles)?;

        x11_status_to_res(self.display(), unsafe {
            xlib::XDrawRectangles(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                rects.as_mut_ptr(),
                rects.len() as c_int,
            )
        })
    }

    #[inline]
    fn draw_arc(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        x11_status_to_res(self.display(), unsafe {
            xlib::XDrawArc(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                rectleft,
                recttop,
                rectwidth,
                rectheight,
                (start_angle.to_degrees() * DEGREES_TO_XLIB_UNITS) as c_int,
                (end_angle.to_degrees() * DEGREES_TO_XLIB_UNITS) as c_int,
            )
        })
    }

    #[inline]
    fn draw_arcs(&self, arcs: &[GeometricArc]) -> crate::Result<()> {
        let mut arcs = arcs_to_xarcs(arcs)?;

        x11_status_to_res(self.display(), unsafe {
            xlib::XDrawArcs(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                arcs.as_mut_ptr(),
                arcs.len() as c_int,
            )
        })
    }

    #[inline]
    fn fill_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        x11_status_to_res(self.display(), unsafe {
            xlib::XFillRectangle(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                x,
                y,
                width,
                height,
            )
        })
    }

    #[inline]
    fn fill_rectangles(&self, rects: &[Rectangle]) -> crate::Result<()> {
        let mut rects = rects_to_xrects(rects)?;

        x11_status_to_res(self.display(), unsafe {
            xlib::XFillRectangles(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                rects.as_mut_ptr(),
                rects.len() as c_int,
            )
        })
    }

    #[inline]
    fn fill_arc(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        x11_status_to_res(self.display(), unsafe {
            xlib::XFillArc(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                rectleft,
                recttop,
                rectwidth,
                rectheight,
                (start_angle.to_degrees() * DEGREES_TO_XLIB_UNITS) as c_int,
                (end_angle.to_degrees() * DEGREES_TO_XLIB_UNITS) as c_int,
            )
        })
    }

    #[inline]
    fn fill_arcs(&self, arcs: &[GeometricArc]) -> crate::Result<()> {
        let mut arcs = arcs_to_xarcs(arcs)?;

        x11_status_to_res(self.display(), unsafe {
            xlib::XFillArcs(
                self.display().as_ptr(),
                self.xid(),
                self.gc().as_ptr(),
                arcs.as_mut_ptr(),
                arcs.len() as c_int,
            )
        })
    }

    #[inline]
    fn image(
        &self,
        image: &dyn GenericImage,
        x: i32,
        y: i32,
        origin_width: u32,
        origin_height: u32,
    ) -> crate::Result<()> {
        // create the pixmap
        let xpixmap = self
            .runtime()
            .as_x11()
            .unwrap()
            .pixmap_storage()
            .register_image(image, self.runtime(), self.monitor())?;

        x11_status_to_res(self.display(), unsafe {
            xlib::XCopyArea(
                self.display().as_ptr(),
                self.xid(),
                xpixmap.inner(),
                self.gc().as_ptr(),
                0,
                0,
                origin_width,
                origin_height,
                x,
                y,
            )
        })
    }
}
