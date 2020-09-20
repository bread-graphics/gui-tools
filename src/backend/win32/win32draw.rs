// MIT/Apache2 License

use crate::{
    color::{colors, AtomicRgba, Rgba},
    graphics::GraphicsInternal,
    image::GenericImage,
    runtime::Runtime,
};
use core::{
    mem,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};
use euclid::Angle;
use winapi::{
    shared::windef::HDC__,
    um::{
        gdiplusenums::UnitPixel,
        gdiplusflat,
        gdiplusgpstubs::{GpBrush, GpGraphics, GpPen},
        gdipluspixelformats::ARGB,
        gdiplustypes::Ok as StatusOk,
    },
};

pub(crate) struct Win32Drawer {
    hdc: NonNull<HDC__>,
    gp_graphics: NonNull<GpGraphics>,
    gp_pen: AtomicPtr<GpPen>,
    gp_brush: AtomicPtr<GpBrush>,
    color: AtomicRgba,
    pen_width: AtomicU32,
    runtime: Runtime,
}

impl Drop for Win32Drawer {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gdiplusflat::GdipDeletePen(*self.gp_pen.get_mut());
            gdiplusflat::GdipDeleteBrush(*self.gp_brush.get_mut());
            gdiplusflat::GdipDeleteGraphics(self.gp_graphics.as_ptr());
        };
    }
}

// helper function to create an ARGB item
#[inline]
fn argb(clr: Rgba) -> ARGB {
    let (r, g, b, a) = clr.convert_elements::<u8>();
    let res = b as ARGB | ((g as ARGB) << 8) | ((r as ARGB) << 16) | ((a as ARGB) << 24);
    res
}

// helper functions for creating pens and brushes
#[inline]
fn create_gp_pen(clr: Rgba, width: u32) -> crate::Result<NonNull<GpPen>> {
    let mut gp_pen: *mut GpPen = ptr::null_mut();
    if unsafe { gdiplusflat::GdipCreatePen1(argb(clr), width as _, UnitPixel, &mut gp_pen) }
        != StatusOk
    {
        Err(crate::win32error("GdipCreatePen1"))
    } else {
        match NonNull::new(gp_pen) {
            Some(p) => Ok(p),
            None => Err(crate::win32error("GdipCreatePen1")),
        }
    }
}

#[inline]
fn create_gp_brush(clr: Rgba) -> crate::Result<NonNull<GpBrush>> {
    let mut gp_brush: *mut GpBrush = ptr::null_mut();
    if unsafe { gdiplusflat::GdipCreateSolidFill(argb(clr), &mut gp_brush as *mut _ as *mut _) }
        != StatusOk
    {
        Err(crate::win32error("GdipCreateSolidFill"))
    } else {
        match NonNull::new(gp_brush) {
            Some(p) => Ok(p),
            None => Err(crate::win32error("GdipCreateSolidFill")),
        }
    }
}

impl Win32Drawer {
    #[inline]
    pub(crate) fn new(runtime: Runtime, hdc: NonNull<HDC__>) -> crate::Result<Self> {
        const DEFAULT_COLOR: Rgba = colors::BLACK;
        const DEFAULT_PEN_WIDTH: u32 = 1;

        assert_eq!(mem::size_of::<Self>(), mem::size_of::<Option<Self>>());

        let mut gp_ptr: *mut GpGraphics = ptr::null_mut();
        if unsafe { gdiplusflat::GdipCreateFromHDC(hdc.as_ptr(), &mut gp_ptr) } != StatusOk {
            return Err(crate::win32error("GdipCreateFromHDC"));
        }
        let gp_graphics = match NonNull::new(gp_ptr) {
            Some(graphics) => graphics,
            None => return Err(crate::win32error("GdipCreateFromHDC")),
        };

        // create an appropriate pen and brush
        let gp_pen = create_gp_pen(DEFAULT_COLOR, DEFAULT_PEN_WIDTH)?;
        let gp_brush = create_gp_brush(DEFAULT_COLOR)?;

        Ok(Self {
            hdc,
            gp_graphics,
            gp_pen: AtomicPtr::new(gp_pen.as_ptr()),
            gp_brush: AtomicPtr::new(gp_brush.as_ptr()),
            color: AtomicRgba::new(DEFAULT_COLOR),
            pen_width: AtomicU32::new(DEFAULT_PEN_WIDTH),
            runtime,
        })
    }
}

macro_rules! graphics {
    ($self: expr) => {{
        $self.gp_graphics.as_ptr()
    }};
}

macro_rules! pen {
    ($self: expr) => {{
        $self.gp_pen.load(Ordering::Acquire)
    }};
}

macro_rules! brush {
    ($self: expr) => {{
        $self.gp_brush.load(Ordering::Acquire)
    }};
}

impl GraphicsInternal for Win32Drawer {
    // TODO: use SetPenColor instead
    #[inline]
    fn set_color(&self, clr: Rgba) -> crate::Result<()> {
        self.color.store(clr, Ordering::Release);

        let new_pen = create_gp_pen(clr, self.pen_width.load(Ordering::Acquire))?;
        let new_brush = create_gp_brush(clr)?;

        // swap out the old pen and brush
        let old_pen = self.gp_pen.swap(new_pen.as_ptr(), Ordering::Acquire);
        let old_brush = self.gp_brush.swap(new_brush.as_ptr(), Ordering::Acquire);

        // delete the old stuff
        unsafe {
            gdiplusflat::GdipDeletePen(old_pen);
            gdiplusflat::GdipDeleteBrush(old_brush);
        };

        Ok(())
    }

    #[inline]
    fn set_line_width(&self, lw: u32) -> crate::Result<()> {
        self.pen_width.store(lw, Ordering::Release);

        let mut new_pen = create_gp_pen(self.color.load(Ordering::Acquire), lw)?;
        let old_pen = self.gp_pen.swap(new_pen.as_ptr(), Ordering::Acquire);
        unsafe { gdiplusflat::GdipDeletePen(old_pen) };
        Ok(())
    }

    #[inline]
    fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result<()> {
        if unsafe {
            gdiplusflat::GdipDrawLineI(
                graphics!(self),
                pen!(self),
                x1 as _,
                y1 as _,
                x2 as _,
                y2 as _,
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipDrawLineI"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn draw_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        if unsafe {
            gdiplusflat::GdipDrawRectangleI(
                graphics!(self),
                pen!(self),
                x as _,
                y as _,
                width as _,
                height as _,
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipDrawRectangleI"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn draw_arc(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
        mut start_angle: Angle<f32>,
        mut end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        let ninety_degrees = Angle::degrees(90.0);
        start_angle += ninety_degrees;
        end_angle += ninety_degrees;

        if unsafe {
            gdiplusflat::GdipDrawArcI(
                graphics!(self),
                pen!(self),
                rectleft as _,
                recttop as _,
                rectwidth as _,
                rectheight as _,
                start_angle.to_degrees(),
                (end_angle - start_angle).to_degrees(),
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipDrawArcI"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn draw_ellipse(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        if unsafe {
            gdiplusflat::GdipDrawEllipseI(
                graphics!(self),
                pen!(self),
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipDrawEllipseI"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn fill_rectangle(&self, x: i32, y: i32, rectwidth: u32, rectheight: u32) -> crate::Result<()> {
        if unsafe {
            gdiplusflat::GdipFillRectangleI(
                graphics!(self),
                brush!(self),
                x,
                y,
                rectwidth as _,
                rectheight as _,
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipFillRectangleI"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn fill_arc(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
        mut start_angle: Angle<f32>,
        mut end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        let ninety_degrees = Angle::degrees(90.0);
        start_angle += ninety_degrees;
        end_angle += ninety_degrees;

        if unsafe {
            gdiplusflat::GdipFillPieI(
                graphics!(self),
                brush!(self),
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
                start_angle.to_degrees(),
                (end_angle - start_angle).to_degrees(),
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipFillPieI"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn fill_ellipse(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        if unsafe {
            gdiplusflat::GdipFillEllipseI(
                graphics!(self),
                brush!(self),
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipFillEllipseI"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn image(
        &self,
        image: &dyn GenericImage,
        origin_x: i32,
        origin_y: i32,
        clip_width: u32,
        clip_height: u32,
    ) -> crate::Result<()> {
        if unsafe {
            gdiplusflat::GdipDrawImagePointRectI(
                graphics!(self),
                self.runtime
                    .as_win32()
                    .unwrap()
                    .image_storage()
                    .register_image(image)?
                    .img_ptr()
                    .as_ptr(),
                origin_x as _,
                origin_y as _,
                0,
                0,
                clip_width as _,
                clip_height as _,
                UnitPixel,
            )
        } != StatusOk
        {
            Err(crate::win32error("GdipDrawImagePointRectI"))
        } else {
            Ok(())
        }
    }
}
