// MIT/Apache2 License

use crate::{
    backend::{x11::X11Surface, SurfaceInner},
    mutex::ShimRwLock,
    runtime::{Runtime, RuntimeBackend},
};
use storagevec::StorageVec;

/// The properties that a window can hold.
pub struct SurfaceProperties {
    pub parent: Option<usize>,
    pub children: StorageVec<usize, 12>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl SurfaceProperties {
    pub fn new(parent: Option<usize>, x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            parent,
            x,
            y,
            width,
            height,
            children: StorageVec::new(),
        }
    }
}

/// A rectangle of pixels on the screen.
pub struct Surface {
    properties: ShimRwLock<SurfaceProperties>,
    internal: SurfaceInner,
}

impl Surface {
    pub(crate) fn new(runtime: &Runtime, properties: SurfaceProperties) -> crate::Result<Self> {
        Ok(Self {
            internal: runtime.backend().surface(runtime, &properties)?,
            properties: ShimRwLock::new(properties),
        })
    }

    #[inline]
    pub fn as_x11(&self) -> Option<&X11Surface> {
        self.internal.as_x11()
    }

    /// The universal ID of the window.
    #[inline]
    pub fn id(&self) -> usize {
        self.internal.id()
    }
}

pub trait SurfaceBackend {
    fn id(&self) -> usize;
}
