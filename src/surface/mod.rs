// MIT/Apache2 License

use crate::{
    backend::{x11::X11Surface, SurfaceInner},
    event::EventTypeMask,
    mutex::ShimRwLock,
    runtime::{Runtime, RuntimeBackend},
    string::CoolString,
};
use storagevec::StorageVec;

mod starting_point;

pub use starting_point::StartingPoint;

/// The properties that a window can hold.
#[derive(Debug, Default)]
pub struct SurfaceProperties {
    pub parent: Option<usize>,
    pub children: StorageVec<usize, 12>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub title: CoolString,
    pub event_mask: StorageVec<EventTypeMask, 20>,
}

/// The properties that a surface is initialized with.
#[derive(Debug, Default)]
pub struct SurfaceInitialization {
    pub parent: Option<usize>,
    pub children: StorageVec<usize, 12>,
    pub starting_point: StartingPoint,
    pub width: u32,
    pub height: u32,
    pub title: CoolString,
    pub event_mask: StorageVec<EventTypeMask, 20>,
}

impl SurfaceProperties {
    #[inline]
    pub fn new(
        parent: Option<usize>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: CoolString,
    ) -> Self {
        Self {
            parent,
            x,
            y,
            width,
            height,
            title,
            children: StorageVec::new(),
            event_mask: StorageVec::new(),
        }
    }
}

impl SurfaceInitialization {
    #[inline]
    pub fn new<Title: Into<CoolString>>(
        parent: Option<usize>,
        starting_point: StartingPoint,
        width: u32,
        height: u32,
        title: Title,
    ) -> Self {
        Self {
            parent,
            starting_point,
            width,
            height,
            title: title.into(),
            children: StorageVec::new(),
            event_mask: StorageVec::new(),
        }
    }

    #[inline]
    pub fn into_properties(self, parent_width: u32, parent_height: u32) -> SurfaceProperties {
        let (x, y) =
            self.starting_point
                .to_x_y(self.width, self.height, parent_width, parent_height);
        let SurfaceInitialization {
            parent,
            width,
            height,
            children,
            title,
            event_mask,
            ..
        } = self;
        let mut sp = SurfaceProperties::new(parent, x, y, width, height, title);
        sp.children = children;
        sp.event_mask = event_mask;
        sp
    }
}

/// A rectangle of pixels on the screen.
pub struct Surface {
    properties: ShimRwLock<SurfaceProperties>,
    internal: SurfaceInner,
}

impl Surface {
    pub(crate) fn new(runtime: &Runtime, properties: SurfaceInitialization) -> crate::Result<Self> {
        let (width, height) = match properties.parent {
            Some(parent) => runtime.surface_at(parent).unwrap().size(),
            None => runtime.default_monitor().size(),
        };

        Ok(Self {
            internal: runtime.backend().surface(runtime, &properties)?,
            properties: ShimRwLock::new(properties.into_properties(width, height)),
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

    /// The width and height of the window.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        let p = self.properties.read();
        (p.width, p.height)
    }

    /// Set the event mask.
    #[inline]
    pub fn set_event_mask(&self, em: &[EventTypeMask]) -> crate::Result<()> {
        self.properties.write().event_mask = em.iter().cloned().collect();
        self.internal.set_event_mask(em)
    }
}

pub trait SurfaceBackend {
    fn id(&self) -> usize;
    fn set_event_mask(&self, mask: &[EventTypeMask]) -> crate::Result<()>;
}
