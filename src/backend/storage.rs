// MIT/Apache2 License

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use super::x11::{X11Runtime, X11Surface};
use crate::{
    runtime::RuntimeBackend,
    surface::{SurfaceBackend, SurfaceProperties},
};

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
}

impl RuntimeBackend for RuntimeInner {}

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
}
