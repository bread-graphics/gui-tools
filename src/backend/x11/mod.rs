// MIT/Apache2 License

pub mod visual;
pub(crate) mod x11displaymanager;
mod x11error;
mod x11event;
mod x11monitor;
mod x11runtime;
mod x11surface;

pub use x11error::*;
pub use x11event::*;
pub use x11monitor::*;
pub use x11runtime::*;
pub use x11surface::*;

use super::{Backend, BackendType, RuntimeInner, SurfaceInner};
use crate::{runtime::Runtime, surface::SurfaceInitialization};

fn open_function() -> crate::Result<(usize, RuntimeInner)> {
    let (default_monitor, rt) = X11Runtime::new()?;
    Ok((default_monitor, RuntimeInner::X11(rt)))
}

fn surface_function(
    runtime: &Runtime,
    props: &SurfaceInitialization,
) -> crate::Result<SurfaceInner> {
    Ok(SurfaceInner::X11(X11Surface::new(runtime, props)?))
}

pub const X11_BACKEND: Backend = Backend::new(BackendType::X11, &open_function, &surface_function);
