// MIT/Apache2 License

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use super::{
    win32::{Win32Runtime, Win32Surface},
    x11::{X11Runtime, X11Surface},
};
use crate::{
    color::Rgba,
    event::{Event, EventTypeMask},
    geometry::Rectangle,
    graphics::GraphicsInternal,
    monitor::Monitor,
    runtime::{Runtime, RuntimeBackend},
    surface::SurfaceBackend,
};
use core::ptr::NonNull;
use storagevec::StorageVec;

pub enum RuntimeInner {
    X11(X11Runtime),
    Win32(Win32Runtime),
    #[cfg(feature = "alloc")]
    Other(Box<dyn RuntimeBackend>),
}

impl RuntimeInner {
    #[inline]
    fn generic(&self) -> &dyn RuntimeBackend {
        match self {
            Self::X11(ref x) => x as _,
            Self::Win32(ref w) => w as _,
            #[cfg(feature = "alloc")]
            Self::Other(ref b) => &**b,
        }
    }

    #[inline]
    pub fn as_x11(&self) -> Option<&X11Runtime> {
        match self {
            Self::X11(ref x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_x11_mut(&mut self) -> Option<&mut X11Runtime> {
        match self {
            Self::X11(ref mut x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_win32(&self) -> Option<&Win32Runtime> {
        match self {
            Self::Win32(ref w) => Some(w),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl RuntimeBackend for RuntimeInner {
    #[inline]
    fn serve_event(&self, real: &Runtime) -> crate::Result<StorageVec<Event, 5>> {
        self.generic().serve_event(real)
    }

    #[inline]
    fn monitor_at(&self, monitor: usize) -> Option<&Monitor> {
        self.generic().monitor_at(monitor)
    }

    #[cfg(feature = "async")]
    async fn serve_event_async(&self) -> crate::Result<StorageVec<Event, 5>> {
        self.generic().serve_event_async().await
    }

    #[inline]
    fn dispatch_event(&self, ev: Event) -> crate::Result<()> {
        self.generic().dispatch_event(ev)
    }
}

pub enum SurfaceInner {
    X11(X11Surface),
    Win32(Win32Surface),
    #[cfg(feature = "alloc")]
    Other(Box<dyn SurfaceBackend>),
}

impl SurfaceInner {
    #[inline]
    fn generic(&self) -> &dyn SurfaceBackend {
        match self {
            Self::X11(ref x) => x,
            Self::Win32(ref w) => w,
            #[cfg(feature = "alloc")]
            Self::Other(ref b) => &**b,
        }
    }

    #[inline]
    pub fn as_x11(&self) -> Option<&X11Surface> {
        match self {
            Self::X11(ref x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_win32(&self) -> Option<&Win32Surface> {
        match self {
            Self::Win32(ref w) => Some(w),
            _ => None,
        }
    }
}

impl SurfaceBackend for SurfaceInner {
    #[inline]
    fn id(&self) -> usize {
        self.generic().id()
    }

    #[inline]
    fn set_event_mask(&self, mask: &[EventTypeMask]) -> crate::Result<()> {
        self.generic().set_event_mask(mask)
    }

    #[inline]
    fn set_size(&self, width: u32, height: u32) -> crate::Result<()> {
        self.generic().set_size(width, height)
    }

    #[inline]
    fn set_location(&self, x: i32, y: i32) -> crate::Result<()> {
        self.generic().set_location(x, y)
    }

    #[inline]
    fn set_background_color(&self, clr: Rgba) -> crate::Result<()> {
        self.generic().set_background_color(clr)
    }

    #[inline]
    fn set_border_color(&self, clr: Rgba) -> crate::Result<()> {
        self.generic().set_border_color(clr)
    }

    #[inline]
    fn set_border_width(&self, width: u32) -> crate::Result<()> {
        self.generic().set_border_width(width)
    }

    #[inline]
    fn graphics_internal(&self) -> crate::Result<NonNull<dyn GraphicsInternal>> {
        self.generic().graphics_internal()
    }

    #[inline]
    fn invalidate(&self, rect: Option<Rectangle>) -> crate::Result<()> {
        self.generic().invalidate(rect)
    }
}
