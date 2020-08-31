// MIT/Apache2 License

mod storage;
pub use storage::*;

use crate::{
    runtime::Runtime,
    surface::{Surface, SurfaceInitialization},
};

pub mod x11;

/// The backing library used by the backend.
#[derive(Copy, Clone)]
pub enum BackendType {
    X11,
    Win32,
    Other(usize),
}

#[derive(Copy, Clone)]
pub struct Backend {
    ty: BackendType,
    open_function: &'static dyn Fn() -> crate::Result<(usize, RuntimeInner)>,
    surface_function:
        &'static dyn Fn(&Runtime, &SurfaceInitialization) -> crate::Result<SurfaceInner>,
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
