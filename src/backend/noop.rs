// MIT/Apache2 License

use super::{Backend, BackendType, RuntimeInner, SurfaceInner};
use crate::{
    color::Rgba,
    event::{Event, EventTypeMask},
    geometry::Rectangle,
    graphics::GraphicsInternal,
    monitor::Monitor,
    runtime::{Runtime, RuntimeBackend},
    surface::{SurfaceBackend, SurfaceInitialization},
};
use core::ptr::NonNull;
use storagevec::StorageVec;

fn open_function() -> crate::Result<(usize, RuntimeInner)> {
    Err(crate::Error::NoOpFunctionCalled)
}

fn surface_function(
    runtime: &Runtime,
    props: &SurfaceInitialization,
) -> crate::Result<SurfaceInner> {
    Err(crate::Error::NoOpFunctionCalled)
}

fn register_function(runtime: &Runtime) {
    Err::<(), crate::Error>(crate::Error::NoOpFunctionCalled).unwrap();
}

pub const NOOP_BACKEND: Backend = Backend::new(
    BackendType::NoOp,
    &open_function,
    &register_function,
    &surface_function,
);

pub fn noop_backend_selector() -> Option<Backend> {
    Some(NOOP_BACKEND)
}

pub struct NoOpRuntime;

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl RuntimeBackend for NoOpRuntime {
    #[inline]
    fn serve_event(&self, _real: &Runtime) -> crate::Result<StorageVec<Event, 5>> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn monitor_at(&self, _index: usize) -> Option<&Monitor> {
        None
    }

    #[cfg(feature = "async")]
    #[inline]
    async fn serve_event_async(&self) -> crate::Result<StorageVec<Event, 5>> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn dispatch_event(&self, _ev: Event) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }
}

pub struct NoOpSurface;

impl SurfaceBackend for NoOpSurface {
    #[inline]
    fn id(&self) -> usize {
        panic!("Requested ID for No-Op Surface")
    }

    #[inline]
    fn set_event_mask(&self, _mask: &[EventTypeMask]) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn set_size(&self, _width: u32, _height: u32) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn set_location(&self, _x: i32, _y: i32) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn set_background_color(&self, _clr: Rgba) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn set_border_color(&self, _clr: Rgba) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn set_border_width(&self, _width: u32) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn graphics_internal(&self) -> crate::Result<NonNull<dyn GraphicsInternal>> {
        Err(crate::Error::NoOpFunctionCalled)
    }

    #[inline]
    fn invalidate(&self, _rectangle: Option<Rectangle>) -> crate::Result<()> {
        Err(crate::Error::NoOpFunctionCalled)
    }
}
