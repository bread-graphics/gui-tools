// MIT/Apache2 License

use super::{Backend, BackendType, RuntimeInner, SurfaceInner};
use crate::{runtime::Runtime, surface::SurfaceInitialization};

mod win32event;
mod win32monitor;
mod win32runtime;
mod win32surface;
pub(crate) mod window_proc;

pub(crate) use win32event::*;
pub use win32monitor::*;
pub use win32runtime::*;
pub use win32surface::*;

fn open_function_commctrl() -> crate::Result<(usize, RuntimeInner)> {
    let (default_monitor, runtime) = Win32Runtime::new(true)?;
    Ok((default_monitor, RuntimeInner::Win32(runtime)))
}

fn open_function_no_commctrl() -> crate::Result<(usize, RuntimeInner)> {
    let (default_monitor, runtime) = Win32Runtime::new(false)?;
    Ok((default_monitor, RuntimeInner::Win32(runtime)))
}

fn register_function(_runtime: &Runtime) {
    /* do nothing */
}

fn surface_function(
    runtime: &Runtime,
    surface: &SurfaceInitialization,
) -> crate::Result<SurfaceInner> {
    let surface = Win32Surface::new(runtime, surface)?;
    Ok(SurfaceInner::Win32(surface))
}

#[inline]
pub(crate) fn win32_backend_selector(commctrl: bool) -> Option<Backend> {
    Some(Backend::new(
        BackendType::Win32,
        if commctrl {
            &open_function_commctrl
        } else {
            &open_function_no_commctrl
        },
        &register_function,
        &surface_function,
    ))
}

pub(crate) fn win32_backend_selector_commctrl() -> Option<Backend> {
    win32_backend_selector(true)
}
pub(crate) fn win32_backend_selector_no_commctrl() -> Option<Backend> {
    win32_backend_selector(false)
}
