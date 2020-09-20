// MIT/Apache2 License

//! This module contains utilities used to deal with backends. The only type exported to the public
//! is the `BackendType` enum and the `Backend` type. This should allow for the user creation of
//! backends. The rest are only available internally.

use crate::{runtime::Runtime, surface::SurfaceInitialization};

pub mod noop;

mod selector;
pub(crate) use selector::*;

mod storage;
pub use storage::*;

#[cfg(target_os = "linux")]
pub mod x11;

#[cfg(not(target_os = "linux"))]
pub mod x11 {
    pub use super::noop::{
        noop_backend_selector as x11_backend_selector, NoOpRuntime as X11Runtime,
        NoOpSurface as X11Surface, NOOP_BACKEND as X11_BACKEND,
    };
}

#[cfg(windows)]
pub mod win32;

#[cfg(not(windows))]
pub mod win32 {
    pub use super::noop::{
        noop_backend_selector as win32_backend_selector, NoOpRuntime as Win32Runtime,
        NoOpSurface as Win32Surface,
    };
}

/// The backing library used by the backend.
#[derive(Copy, Clone)]
pub enum BackendType {
    X11,
    Win32,
    AppKit,
    DOM,
    NoOp,
    Other(usize),
    OtherStr(&'static str),
}

/// The backend library that `gui-tools` acts as an abstraction over. This is implemented as a series of
/// function pointers that call functions that use this library.
#[derive(Copy, Clone)]
pub struct Backend {
    ty: BackendType,
    open_function: &'static dyn Fn() -> crate::Result<(usize, RuntimeInner)>,
    register_function: &'static dyn Fn(&Runtime),
    surface_function:
        &'static dyn Fn(&Runtime, &SurfaceInitialization) -> crate::Result<SurfaceInner>,
    pub suppress_peeker_loop: bool,
}

impl Backend {
    /// Create a new runtime by some functions.
    #[inline]
    pub const fn new(
        ty: BackendType,
        open_function: &'static dyn Fn() -> crate::Result<(usize, RuntimeInner)>,
        register_function: &'static dyn Fn(&Runtime),
        surface_function: &'static dyn Fn(
            &Runtime,
            &SurfaceInitialization,
        ) -> crate::Result<SurfaceInner>,
    ) -> Self {
        Self {
            ty,
            open_function,
            register_function,
            surface_function,
            suppress_peeker_loop: false,
        }
    }

    /// The type associated with this backend.
    #[inline]
    pub fn ty(&self) -> BackendType {
        self.ty
    }

    /// Open the runtime.
    #[inline]
    pub fn open(&self) -> crate::Result<(usize, RuntimeInner)> {
        (self.open_function)()
    }

    /// Create a new surface.
    #[inline]
    pub fn surface(
        &self,
        runtime: &Runtime,
        props: &SurfaceInitialization,
    ) -> crate::Result<SurfaceInner> {
        (self.surface_function)(runtime, props)
    }

    /// Register the runtime.
    #[inline]
    pub fn register(&self, runtime: &Runtime) {
        (self.register_function)(runtime);
    }
}
