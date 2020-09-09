// MIT/Apache2 License

#![cfg(target_os = "linux")]

pub mod visual;
pub(crate) mod x11displaymanager;
pub mod x11drawable;
mod x11error;
mod x11event;
mod x11monitor;
mod x11runtime;
mod x11surface;

pub use x11error::*;
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

fn register_function(runtime: &Runtime) {
    x11displaymanager::set_runtime(runtime.as_x11().unwrap().display().clone(), runtime.clone());
}

pub(crate) const X11_BACKEND: Backend = Backend::new(
    BackendType::X11,
    &open_function,
    &register_function,
    &surface_function,
);

pub(crate) fn x11_backend_selector() -> Option<Backend> {
    Some(X11_BACKEND)
}
