// MIT/Apache2 License

use crate::{
    color::{colors, Rgba},
    graphics::GraphicsInternal,
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

#[cfg(not(feature = "std"))]
use conquer_once::spin::Lazy;
#[cfg(feature = "std")]
use conquer_once::Lazy;

// container for default objects
#[repr(transparent)]
struct StockObject(AtomicPtr<c_void>);

impl StockObject {
    #[inline]
    fn new(ty: c_int) -> crate::Result<Self> {
        let so = unsafe { wingdi::GetStockObject(ty) };
        if so.is_null() {
            Err(crate::win32error("GetStockObject"))
        } else {
            Ok(Self(AtomicPtr::new(so as *mut _)))
        }
    }

    #[inline]
    fn get(&self) -> HGDIOBJ {
        self.0.load(Ordering::Acquire)
    }
}

static NULL_PEN: Lazy<StockObject> = Lazy::new(|| StockObject::new(wingdi::NULL_PEN as _).unwrap());
static DEFAULT_BRUSH: Lazy<StockObject> =
    Lazy::new(|| StockObject::new(wingdi::DC_BRUSH as _).unwrap());
static NULL_BRUSH: Lazy<StockObject> =
    Lazy::new(|| StockObject::new(wingdi::NULL_BRUSH as _).unwrap());

#[inline]
fn get_pen_color(dc: &UnsafeCell<HDC__>) -> crate::Result<COLORREF> {
    let res = unsafe { wingdi::GetDCPenColor(dc.get()) };
    if res == wingdi::CLR_INVALID {
        Err(crate::win32error("GetDCPenColor"))
    } else {
        Ok(res)
    }
}

#[inline]
fn set_bg_mode(dc: &UnsafeCell<HDC__>, visible: bool) -> crate::Result<()> {
    // either use the default brush with the SetDCBrushColor function or the null brush
    let new_brush = if visible {
        DEFAULT_BRUSH.get()
    } else {
        NULL_BRUSH.get()
    };

    if unsafe { wingdi::SelectObject(dc.get(), new_brush as *mut _) }.is_null() {
        return Err(crate::win32error("SelectObject"));
    } else {
        Ok(())
    }
}

// copypasted from winapi
const fn const_rgb(r: BYTE, g: BYTE, b: BYTE) -> COLORREF {
    r as COLORREF | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}

#[inline]
fn set_pen_details<F>(dc: &UnsafeCell<HDC__>, mut f: F) -> crate::Result<()>
where
    F: FnMut(&mut LOGPEN),
{
    const DEFAULT_LOGPEN: LOGPEN = LOGPEN {
        lopnStyle: wingdi::PS_SOLID as _,
        lopnWidth: POINT { x: 1, y: 0 },
        lopnColor: const_rgb(0, 0, 0),
    };

    let previous_pen = unsafe { wingdi::SelectObject(dc.get(), NULL_PEN.get()) };
    let mut logpen = DEFAULT_LOGPEN;

    if !previous_pen.is_null() {
        if unsafe {
            wingdi::GetObjectA(
                previous_pen,
                mem::size_of::<LOGPEN>() as _,
                &mut logpen as *mut _ as *mut _,
            )
        } == 0
        {
            log::warn!("GetObjectA failed, but we don't truly need it.");
        }
    }

    // run the function we have on the logpen
    f(&mut logpen);

    // dealloc the old pen
    unsafe { wingdi::DeleteObject(previous_pen) };

    // create a new pen
    let new_pen = unsafe { wingdi::CreatePenIndirect(&logpen) };
    if new_pen.is_null() {
        return Err(crate::win32error("CreatePenIndirect"));
    }

    // select the object in
    unsafe { wingdi::SelectObject(dc.get(), new_pen as *mut _) };

    Ok(())
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
        };

        set_pen_details(self, move |l| l.lopnColor = clr)
    }

    #[inline]
    fn set_line_width(&self, lw: u32) -> crate::Result<()> {
        set_pen_details(self, move |l| l.lopnWidth.x = lw as _)
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
