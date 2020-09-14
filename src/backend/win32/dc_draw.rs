// MIT/Apache2 License

use crate::{
    color::{colors, Rgba},
    graphics::GraphicsInternal,
};
use core::{
    cell::UnsafeCell,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
};
use euclid::Angle;
use winapi::{
    ctypes::c_void,
    shared::{ntdef::HANDLE, windef::HDC__},
    um::wingdi::{self, LOGPEN},
};

#[inline]
fn set_bg_mode(dc: &UnsafeCell<HDC__>, visible: bool) -> crate::Result<()> {
    if unsafe {
        wingdi::SetBkMode(
            dc.get(),
            if visible {
                wingdi::OPAQUE as _
            } else {
                wingdi::TRANSPARENT as _
            },
        )
    } == 0
    {
        Err(crate::win32error("SetBkMode"))
    } else {
        Ok(())
    }
}

#[inline]
fn draw_arc(
    dc: &UnsafeCell<HDC__>,
    rectleft: i32,
    recttop: i32,
    rectwidth: u32,
    rectheight: u32,
    start_angle: Angle<f32>,
    end_angle: Angle<f32>,
    visible: bool,
) -> crate::Result<()> {
    set_bg_mode(dc, visible)?;

    let (rw2, rh2) = (rectwidth as i32 / 2, rectheight as i32 / 2);
    let (center_x, center_y) = (rectleft as i32 + rw2, recttop as i32 + rh2);
    let (real_start_angle, real_end_angle) = (
        start_angle + Angle::degrees(90.0),
        end_angle + Angle::degrees(90.0),
    );
    let (rx1, ry1) = (
        center_x + (rw2 * real_start_angle.get().cos() as i32),
        center_y + (rh2 * real_start_angle.get().sin() as i32),
    );
    let (rx2, ry2) = (
        center_x + (rw2 * real_end_angle.get().cos() as i32),
        center_y + (rh2 * real_end_angle.get().sin() as i32),
    );

    if unsafe {
        wingdi::Pie(
            dc.get(),
            rectleft,
            recttop,
            rectwidth as _,
            rectheight as _,
            rx1,
            ry1,
            rx2,
            ry2,
        )
    } == 0
    {
        Err(crate::win32error("Pie"))
    } else {
        Ok(())
    }
}

impl GraphicsInternal for UnsafeCell<HDC__> {
    #[inline]
    fn set_color(&self, clr: Rgba) -> crate::Result<()> {
        let (r, g, b, _) = clr.convert_elements::<u8>();
        let clr = wingdi::RGB(r, g, b);
        unsafe {
            wingdi::SetDCBrushColor(self.get(), clr);
            wingdi::SetDCPenColor(self.get(), clr);
        };
        Ok(())
    }

    #[inline]
    fn set_line_width(&self, lw: u32) -> crate::Result<()> {
        // create a new pen
        let our_clr = unsafe { wingdi::GetDCPenColor(self.get()) };
        if our_clr == wingdi::CLR_INVALID {
            return Err(crate::win32error("GetDCPenColor"));
        }

        let hpen = unsafe { wingdi::CreatePen(wingdi::PS_SOLID as _, lw as _, our_clr) };
        if hpen.is_null() {
            return Err(crate::win32error("CreatePen"));
        }

        // put it into the DC
        let res = unsafe { wingdi::SelectObject(self.get(), hpen as *mut _) };
        if res.is_null() {
            Err(crate::win32error("SelectObject"))
        } else {
            unsafe { wingdi::DeleteObject(res as *mut _) };
            Ok(())
        }
    }

    #[inline]
    fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> crate::Result<()> {
        unsafe { wingdi::MoveToEx(self.get(), x1, y1, ptr::null_mut()) };
        unsafe { wingdi::LineTo(self.get(), x2, y2) };
        Ok(())
    }

    #[inline]
    fn draw_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        set_bg_mode(self, false)?;

        if unsafe { wingdi::Rectangle(self.get(), x, y, width as _, height as _) } == 0 {
            Err(crate::win32error("Rectangle"))
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
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        draw_arc(
            self,
            rectleft,
            recttop,
            rectwidth,
            rectheight,
            start_angle,
            end_angle,
            false,
        )
    }

    #[inline]
    fn draw_ellipse(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        set_bg_mode(self, false)?;

        if unsafe {
            wingdi::Ellipse(
                self.get(),
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
            )
        } == 0
        {
            Err(crate::win32error("Ellipse"))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn fill_rectangle(&self, x: i32, y: i32, width: u32, height: u32) -> crate::Result<()> {
        set_bg_mode(self, true)?;

        if unsafe { wingdi::Rectangle(self.get(), x, y, width as _, height as _) } == 0 {
            Err(crate::win32error("Rectangle"))
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
        start_angle: Angle<f32>,
        end_angle: Angle<f32>,
    ) -> crate::Result<()> {
        draw_arc(
            self,
            rectleft,
            recttop,
            rectwidth,
            rectheight,
            start_angle,
            end_angle,
            true,
        )
    }

    #[inline]
    fn fill_ellipse(
        &self,
        rectleft: i32,
        recttop: i32,
        rectwidth: u32,
        rectheight: u32,
    ) -> crate::Result<()> {
        set_bg_mode(self, true)?;

        if unsafe {
            wingdi::Ellipse(
                self.get(),
                rectleft,
                recttop,
                rectwidth as _,
                rectheight as _,
            )
        } == 0
        {
            Err(crate::win32error("Ellipse"))
        } else {
            Ok(())
        }
    }
}
