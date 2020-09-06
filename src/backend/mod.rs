// MIT/Apache2 License

//! This module contains utilities used to deal with backends. The only type exported to the public
//! is the `BackendType` enum. The rest is currently only available internally.

mod selector;
pub(crate) use selector::*;

mod storage;
pub(crate) use storage::*;

use crate::{runtime::Runtime, surface::SurfaceInitialization};

pub(crate) mod x11;

/// The backing library used by the backend.
#[derive(Copy, Clone)]
pub enum BackendType {
    X11,
    Win32,
    Other(usize),
}

#[derive(Copy, Clone)]
pub(crate) struct Backend {
    ty: BackendType,
    open_function: &'static dyn Fn() -> crate::Result<(usize, RuntimeInner)>,
    surface_function:
        &'static dyn Fn(&Runtime, &SurfaceInitialization) -> crate::Result<SurfaceInner>,
    pub suppress_peeker_loop: bool,
}

impl Backend {
    #[inline]
    pub const fn new(
        ty: BackendType,
        open_function: &'static dyn Fn() -> crate::Result<(usize, RuntimeInner)>,
        surface_function: &'static dyn Fn(
            &Runtime,
            &SurfaceInitialization,
        ) -> crate::Result<SurfaceInner>,
    ) -> Self {
        Self {
            ty,
            open_function,
            surface_function,
            suppress_peeker_loop: false,
        }
    }

    #[inline]
    pub fn ty(&self) -> BackendType {
        self.ty
    }

    #[inline]
    pub fn open(&self) -> crate::Result<(usize, RuntimeInner)> {
        (self.open_function)()
    }

    #[inline]
    pub fn surface(
        &self,
        runtime: &Runtime,
        props: &SurfaceInitialization,
    ) -> crate::Result<SurfaceInner> {
        (self.surface_function)(runtime, props)
    }
}
