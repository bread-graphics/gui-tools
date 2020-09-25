// MIT/Apache2 License

#![cfg(target_os = "macos")]

mod appkitmonitor;
mod appkitruntime;

pub use appkitmonitor::*;
pub use appkitruntime::*;

use super::{Backend, BackendType, RuntimeInner, SurfaceInner};
use crate::{runtime::Runtime, surface::SurfaceInitialization};

fn open_function() -> crate::Result<(usize, RuntimeInner)> {
    let (default_monitor, runtime) = AppkitRuntime::new()?;
    Ok((default_monitor, RuntimeInner::Appkit(runtime)))
}

fn register_function(_runtime: &Runtime) {}

fn surface_function(
    runtime: &Runtime,
    surface: &SurfaceInitialization,
) -> crate::Result<SurfaceInner> {
    let surface = AppkitSurface::new(runtime, surface)?;
    Ok(SurfaceInner::Appkit(surface))
}

pub const APPKIT_BACKEND: Backend = Backend::new(
    BackendType::AppKit,
    &open_function,
    &register_function,
    &surface_function,
);

pub fn appkit_backend_selector() -> Option<Backend> {
    Some(APPKIT_BACKEND)
}

pub type CGFloat = f64;
