// MIT/Apache2 License

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use super::x11::{X11Runtime, X11Surface};
use crate::{
    event::{Event, EventTypeMask},
    monitor::Monitor,
    runtime::{Runtime, RuntimeBackend},
    surface::{SurfaceBackend, SurfaceProperties},
};
use storagevec::StorageVec;

pub enum RuntimeInner {
    X11(X11Runtime),
    #[cfg(feature = "alloc")]
    Other(Box<dyn RuntimeBackend>),
}

impl RuntimeInner {
    #[inline]
    fn generic(&self) -> &dyn RuntimeBackend {
        match self {
            Self::X11(ref x) => x as _,
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
    #[cfg(feature = "alloc")]
    Other(Box<dyn SurfaceBackend>),
}

impl SurfaceInner {
    #[inline]
    fn generic(&self) -> &dyn SurfaceBackend {
        match self {
            Self::X11(ref x) => x,
            #[cfg(feature = "alloc")]
            Self::Other(ref b) => &**b,
        }
    }

    #[inline]
    fn generic_mut(&mut self) -> &mut dyn SurfaceBackend {
        match self {
            Self::X11(ref mut x) => x,
            #[cfg(feature = "alloc")]
            Self::Other(ref mut b) => &mut **b,
        }
    }

    #[inline]
    pub fn as_x11(&self) -> Option<&X11Surface> {
        match self {
            Self::X11(ref x) => Some(x),
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
}
