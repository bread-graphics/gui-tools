// MIT/Apache2 License

mod external;

use crate::{
    color::{colors, Rgba},
    graphics::GraphicsInternal,
    mutex::ShimMutex as Mutex,
};
use core::{
    cell::UnsafeCell,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};
use euclid::Angle;
use winapi::{
    ctypes::{c_int, c_void},
    shared::{
        minwindef::BYTE,
        ntdef::HANDLE,
        windef::{COLORREF, HDC__, HGDIOBJ, POINT},
    },
    um::wingdi::{self, LOGPEN},
};

struct GDIProps {
    color: Rgba,
    pen_width: u32,
    pen: external::GDIPPen,
    brush: external::GDIPBrush,
}

// structure to hold graphical information
pub(crate) struct Win32GDIInfo {
    hdc: NonNull<HDC__>,
    graphics: Mutex<external::GDIPGraphics>,
    color_props: Mutex<GDIProps>,
}

impl Drop for Win32GDIInfo {
    fn drop(&mut self) {
        unsafe {
            external::done_pen(self.color_props.get_mut().pen);
            external::done_brush(self.color_props.get_mut().brush);
            external::done_graphics(*self.graphics.get_mut());
        };
    }
}

impl Win32GDIInfo {
    #[inline]
    pub(crate) fn new(hdc: NonNull<HDC__>) -> crate::Result<Self> {
        const DEFAULT_COLOR: Rgba = colors::BLACK;
        const DEFAULT_PEN_WIDTH: u32 = 1;

        // create graphics, pen, and brush
        let mut graphics: MaybeUninit<external::GDIPGraphics> = MaybeUninit::uninit();
        let mut pen: MaybeUninit<external::GDIPPen> = MaybeUninit::uninit();
        let mut brush: MaybeUninit<external::GDIPBrush> = MaybeUninit::uninit();

        if unsafe { external::create_graphics(hdc.as_ptr(), graphics.as_mut_ptr()) } == 0 {
            return Err(crate::win32error("Graphics::Graphics"));
        }

        if unsafe {
            external::create_pen(DEFAULT_COLOR.into(), DEFAULT_PEN_WIDTH, pen.as_mut_ptr())
        } == 0
        {
            return Err(crate::win32error("Pen::Pen"));
        }

        if unsafe { external::create_brush(DEFAULT_COLOR.into(), brush.as_mut_ptr()) } == 0 {
            return Err(crate::win32error("Brush::Brush"));
        }

        Ok(Self {
            hdc,
            graphics: Mutex::new(unsafe { graphics.assume_init() }),
            color_props: Mutex::new(GDIProps {
                color: DEFAULT_COLOR,
                pen_width: DEFAULT_PEN_WIDTH,
                pen: unsafe { pen.assume_init() },
                brush: unsafe { brush.assume_init() },
            }),
        })
    }
}

impl GraphicsInternal for Win32GDIInfo {
    #[inline]
    fn set_color(&self, rgba: Rgba) -> crate::Result<()> {
        let mut props = self.color_props.lock();
        props.color = rgba;

        // create a new pen and brush
        let mut pen: MaybeUninit<external::GDIPPen> = MaybeUninit::uninit();
        let mut brush: MaybeUninit<external::GDIPBrush> = MaybeUninit::uninit();

        unsafe { external::create_pen(rgba.into(), props.pen_width, pen.as_mut_ptr()) };
        unsafe { external::create_brush(rgba.into(), brush.as_mut_ptr()) };

        let mut pen = unsafe { pen.assume_init() };
        let mut brush = unsafe { brush.assume_init() };

        // swap them in and dispose of the olds ones
        mem::swap(&mut pen, &mut props.pen);
        mem::swap(&mut brush, &mut props.brush);

        unsafe { external::done_pen(pen) };
        unsafe { external::done_brush(brush) };

        Ok(())
    }

    #[inline]
    fn set_line_width(&self, width: u32) -> crate::Result<()> {
        let mut props = self.color_props.lock();
        props.pen_width = width;

        let mut pen: MaybeUninit<external::GDIPPen> = MaybeUninit::uninit();
        unsafe { external::create_pen(props.color.into(), width, pen.as_mut_ptr()) };
        let mut pen = unsafe { pen.assume_init() };

        // swap out the other pen and dispose of it
        mem::swap(&mut pen, &mut props.pen);

        unsafe { external::done_pen(pen) };

        Ok(())
    }

    #[inline]
    fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result<()> {
        unsafe {
            external::draw_line(
                &mut self.graphics.lock(),
                &self.color_props.lock().pen,
                x1,
                y1,
                x2,
                y2,
            )
        };
        Ok(())
    }

    #[inline]
    fn draw_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        unsafe {
            external::draw_rectangle(
                &mut self.graphics.lock(),
                &self.color_props.lock().pen,
                x,
                y,
                width as _,
                height as _,
            )
        };
        Ok(())
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
        unsafe {
            external::draw_arc(
                &mut self.graphics.lock(),
                &self.color_props.lock().pen,
                rectleft,
                recttop,
                rectwidth,
                rectheight,
                start_angle.to_degrees(),
                end_angle.to_degrees(),
            )
        };
        Ok(())
    }

    #[inline]
    fn draw_ellipse(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        unsafe {
            external::draw_ellipse(
                &mut self.graphics.lock(),
                &self.color_props.lock().pen,
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
            )
        };
        Ok(())
    }

    #[inline]
    fn fill_rectangle(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        unsafe {
            external::fill_rectangle(
                &mut self.graphics.lock(),
                &self.color_props.lock().brush,
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
            )
        };
        Ok(())
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
        unsafe {
            external::fill_arc(
                &mut self.graphics.lock(),
                &self.color_props.lock().brush,
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
                start_angle.to_degrees(),
                end_angle.to_degrees(),
            )
        };
        Ok(())
    }

    #[inline]
    fn fill_ellipse(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        unsafe {
            external::fill_arc(
                &mut self.graphics.lock(),
                &self.color_props.lock().brush,
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
            )
        };
        Ok(())
    }
}
